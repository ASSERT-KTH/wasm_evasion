#![feature(internal_output_capture)]

use clap::{load_yaml, value_t, App};
use env_logger::{Builder, Target, Env};
use errors::{AResult, CliError};
use info::InfoExtractor;
use log::Record;
use sha2::{Sha256, Digest};
use std::{
    borrow::{Borrow, BorrowMut},
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    process,
    rc::Rc,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    thread::spawn,
    time::{self, Duration, UNIX_EPOCH, SystemTime}, str::FromStr, fmt::Display,
};
use wasm_mutate::WasmMutate;

use crate::meta::Meta;
use mongodb::options::{ClientOptions, Credential, Predicate};
use mongodb::sync::Client;
use wasm_shrink::{WasmShrink, IsInteresting};

#[macro_use]
extern crate log;

mod errors;
pub mod info;
mod meta;
use anyhow::Context;
use tempfile::NamedTempFile;

pub const NO_WORKERS: usize = 16;

pub trait Hasheable {
    fn hash256(self: &Self) -> Vec<u8>;
}

impl Hasheable for Vec<u8> {
    fn hash256(self: &Self) -> Vec<u8> {

        let mut encoder = Sha256::new();
        encoder.update(self);
        let hash_bytes = encoder.finalize();
        hash_bytes.to_vec()
    }
}


pub trait Printable {
    fn fmt1(self: &Self) -> String;
}

impl Printable for Vec<u8> {
    fn fmt1(self: &Self) -> String {
        self.iter()
            .map(|x| format!("{:02x}", x))
            .collect::<Vec<_>>()
            .join("")
    }
}

#[derive(Debug)]
pub struct State {
    dbclient: Option<Client>,
    collection_name: String,
    dbname: String,
    process: AtomicU32,
    error: AtomicU32,
    parsing_error: AtomicU32,
    out_folder: Option<String>,
}

pub fn get_wasm_info(state: Arc<State>, chunk: Vec<PathBuf>) -> AResult<Vec<PathBuf>> {
    if chunk.len() == 0 {
        return Ok(vec![]);
    }

    for f in chunk.iter() {
        let mut file = fs::File::open(f)?;

        // Filter first the header to check for Wasm
        let mut buf = [0; 4];
        &file.read_exact(&mut buf).unwrap();

        match &buf {
            b"\0asm" => {
                //println!("Wasm !");

                let mut meta = meta::Meta::new();
                meta.id = f.file_name().unwrap().to_str().unwrap().to_string();
                // Get size of the file
                let fileinfo = fs::metadata(f)?;
                meta.size = fileinfo.len() as usize;

                // Parse Wasm to get more info
                let bindata = fs::read(f)?;
                let cp = bindata.clone();

                let info =
                    std::panic::catch_unwind(move || InfoExtractor::get_info(&cp, &mut meta));

                match info {
                    Err(e) => {
                        println!("{:#?}               Parsing error {:?}", f, e);

                        if state
                            .parsing_error
                            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
                            % 9
                            == 0
                        {
                            println!(
                                "{} parsing errors!",
                                state.parsing_error.load(Ordering::Relaxed)
                            );
                        }
                        continue;
                    }
                    _ => {
                        // continue
                    }
                }

                let info = info.map_err(|x| CliError::Any(format!("{:#?}", x)))?;

                match info {
                    Err(e) => {
                        println!("{:#?}               Error {:?}", f, e);

                        if state
                            .error
                            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
                            % 9
                            == 0
                        {
                            println!("{} errors!", state.error.load(Ordering::Relaxed));
                        }
                        continue;
                    }
                    _ => {
                        // continue
                    }
                }

                // Get mutation info, TODO

                let mut config = WasmMutate::default();
                let stinfo = config
                    .setup(&bindata)
                    .map_err(|x| CliError::Any(format!("{:#?}", x)));

                match stinfo {
                    Err(e) => {
                        if state
                            .error
                            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
                            % 9
                            == 0
                        {
                            println!("{} errors!", state.error.load(Ordering::Relaxed));
                        }
                        continue
                    }
                    Ok(_) => {}
                }

                config.preserve_semantics(true);

                let mut cp = info?.clone();

                let info = InfoExtractor::get_mutable_info(&mut cp, config);

                match info {
                    Ok(info) => {
                        // Save meta to_string mongodb
                        if let Some(client) = &state.dbclient {
                            let db = client.database(&state.dbname);
                            let collection = db.collection::<Meta>(&state.collection_name);

                            let docs = vec![info];

                            match collection.insert_many(docs, None) {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("{:?}", e)
                                }
                            }
                        } else {
                            println!("Where is the client")
                        }
                    }
                    Err(e) => {
                        println!("{:?}", e)
                    }
                }
            }
            _ => {
                eprintln!("\nJust discard {:?}\n", f);
            }
        }

        if state
            .process
            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
            % 99
            == 0
        {
            println!("{} processed", state.process.load(Ordering::Relaxed));
        }
    }

    Ok(vec![])
}

pub fn get_only_wasm(state: Arc<State>, files: &Vec<PathBuf>) -> Result<Vec<PathBuf>, CliError> {
    let mut workers = vec![vec![]; NO_WORKERS];

    for (idx, file) in files.iter().enumerate() {
        workers[idx % NO_WORKERS].push(file.clone());
    }

    let jobs = workers
        .into_iter()
        .enumerate()
        .map(|(_, x)| {
            let t = state.clone();

            spawn(move || get_wasm_info(t, x))
        })
        .collect::<Vec<_>>();

    for j in jobs {
        let _ = j.join().map_err(|x| CliError::Any(format!("{:#?}", x)))?;
    }

    println!("");
    println!("{} processed", state.process.load(Ordering::Relaxed));
    println!(
        "{} parsing errors!",
        state.parsing_error.load(Ordering::Relaxed)
    );
    println!("{} errors!", state.error.load(Ordering::Relaxed));

    Ok(vec![])
}

macro_rules! arge {
    ($str: literal) => {
        CliError::Arg($str.to_string())
    };
}

macro_rules! arg_or_error {
    ($matches: ident,$arg: literal) => {
        $matches.value_of($arg).ok_or(arge!($arg))?.to_string()
    };
}

#[derive(Debug)]
pub struct Interesting(bool);

impl Display for Interesting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

impl IsInteresting for Interesting {
    fn is_interesting(&self) -> bool {
        self.0
    }
}


pub fn reduce_single_binary(state: Arc<State>, chunk: Vec<PathBuf>) -> AResult<()> {
    println!("reducing {} binaries", chunk.len());

    let outfolder = state.out_folder.as_ref().unwrap().clone();
    let dbclient = state.dbclient.as_ref().unwrap().clone();
    let dbname = state.dbname.clone();
    let collection_name = state.collection_name.clone();

    'iter: for f in chunk.iter() {
        let mut file = fs::File::open(f)?;

        // Filter first the header to check for Wasm
        let mut buf = [0; 4];
        let _ = &file.read_exact(&mut buf)?;

        match &buf {
            b"\0asm" => {

                let mut meta = meta::Meta::new();
                meta.id = f.file_name().unwrap().to_str().unwrap().to_string();
                let name = f.file_name().unwrap().to_str().unwrap().to_string();
                // Get size of the file
                let fileinfo = fs::metadata(f)?;
                meta.size = fileinfo.len() as usize;

                // Parse Wasm to get more info
                let bindata = fs::read(f)?;
                let cp = bindata.clone();

                let reducer = WasmShrink::default();
                //let reducer = reducer.allow_empty(true);

                let output = PathBuf::from_str(&format!("{}/{}.shrunken.wasm", outfolder, name)).unwrap();
                let logs = PathBuf::from_str(&format!("{}/{}.shrunken.logs", outfolder, name)).unwrap();
                
                // copy the original in the folder to get it as the new shrunked binary

                std::fs::write(output.clone(), bindata.clone()).unwrap();

                //let predicate = PathBuf::from_str(&format!("{}/{}.shrunken.predicate", outfolder, name)).unwrap();
                //let wat_path = PathBuf::from_str(&format!("{}/{}.shrunken.wat", outfolder, name)).unwrap();
                

                log::debug!("Start =========== {}", name);

                let initial_size = cp.len();
                let r = reducer.on_new_smallest(Some(Box::new({
                    let output = output.clone();
                    move |new_smallest: &[u8]| {
                        
                        let tmp = match output.parent() {
                            Some(parent) => NamedTempFile::new_in(parent),
                            None => NamedTempFile::new(),
                        };
                        let tmp = tmp.context("Failed to create a temporary file")?;
                        std::fs::write(tmp.path(), new_smallest).with_context(|| {
                            format!("Failed to write to file: {}", tmp.path().display())
                        })?;
                        std::fs::rename(tmp.path(), &output).with_context(|| {
                            format!(
                                "Failed to rename {} to {}",
                                tmp.path().display(),
                                output.display()
                            )
                        })?;

                        // TODO, there should be a way to collect which mutations were aplied here, getting the stderr logs maybe

    
                        /*println!(
                            "{} bytes ({:.02}% smaller)",
                            new_smallest.len(),
                            (100.0 - (new_smallest.len() as f64 / initial_size as f64 * 100.0))
                        );*/
    
                        // Now write the WAT disassembly as well.
                        /*match wasmprinter::print_bytes(new_smallest) {
                            Err(e) => {
                                println!("{}", e);
                                // Ignore disassembly errors, since this isn't critical for
                                // shrinking.
                                //log::warn!("Error disassembling the shrunken Wasm into WAT: {}", e);
                            }
                            Ok(wat) => {
                                let wat_path = output.with_extension("wat");
                                //log::info!("Writing WAT disassembly to {}", wat_path.display());
                                std::fs::write(&wat_path, wat).with_context(|| {
                                    format!("Failed to write WAT disassembly to {}", wat_path.display())
                                })?;
                            }
                        }*/
    
                        Ok(())
                    }
                })))
                .run(cp, move |wasm| {
                    
                    //println!("l {}", wasm.len());
                    Ok(Interesting(wasm.len() > 8))
                });

                let logs = match r {
                    Err(e) => {
                        eprintln!("\t\t Error {e}");
                        if state
                        .error
                        .fetch_add(1, std::sync::atomic::Ordering::Acquire)
                        % 9
                        == 0
                        {
                            println!("{} errors!", state.error.load(Ordering::Relaxed));
                        }
                        continue 'iter;
                    },
                    Ok(_i) => {
                        // TODO, get the stderr and process it as the canonization path
                        // read the logs file
                        let name = std::thread::current();
                        let name = name.name().unwrap();
                        let name = format!("output{}.log", name);
                        let content = std::fs::read(name).unwrap();

                        String::from_utf8(content).unwrap()
                    }
                };

                // Filter first the header to check for Wasm

                let mut meta = meta::Meta::new();
                meta.id = output.display().to_string();


                let bindata = loop {
                    let bindata = fs::read(output.clone());

                    match bindata {
                        Err(e) => {
                            println!("{}", e);
                            continue 'iter;
                        }
                        Ok(r) => {
                            break r
                        }
                    }
                };
                
                // Get size of the file
                meta.tpe = "canonical".to_string();
                meta.hash = bindata.to_vec().hash256().fmt1();
                meta.parent = Some(name.clone());
                meta.size = bindata.len();
                meta.description = format!("seed: {}, fuel: {}, ratio {}", 0, 1000, bindata.len() as f32/initial_size as f32);
                meta.logs = logs;

                let db = dbclient.database(&dbname);
                let collection = db.collection::<Meta>(&collection_name);

                let docs = vec![meta];

                match collection.insert_many(docs, None) {
                    Ok(_) => {
                    }
                    Err(e) => {
                        panic!(e)
                    }
                }

            }
            _ => {
                eprintln!("\nJust discard {:?}\n", f);
            }
        }

        if state
            .process
            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
            % 99
            == 0
        {
            println!("{} processed", state.process.load(Ordering::Relaxed));
        }
    }

    Ok(())
}

pub fn reduce_binaries(state: Arc<State>, files: &Vec<PathBuf>) -> Result<(), CliError> {
    let mut workers = vec![vec![]; NO_WORKERS];

    for (idx, file) in files.iter().enumerate() {
        workers[idx % NO_WORKERS].push(file.clone());
    }

    let jobs = workers
        .into_iter()
        .enumerate()
        .map(|(i, x)| {
            let t = state.clone();
            std::thread::Builder::new()
                .name(format!("t{}", i))
                .spawn(move || reduce_single_binary(t, x))
                .unwrap()
        })
        .collect::<Vec<_>>();

    for j in jobs {
        let _ = j.join().map_err(|x| CliError::Any(format!("{:#?}", x)))?;
    }

    println!("");
    println!("{} processed", state.process.load(Ordering::Relaxed));
    println!(
        "{} parsing errors!",
        state.parsing_error.load(Ordering::Relaxed)
    );
    println!("{} errors!", state.error.load(Ordering::Relaxed));

    Ok(())
}

pub fn reduce(state: Arc<State>, path: String) -> AResult<()> {
    println!("Creating folder if it doesn't exist");

    let outf = &state.out_folder;
    let outf = outf.as_ref().unwrap();

    std::fs::create_dir(outf.clone()); // Ignore if error since it's already created

    let mut files = vec![];

    let mut count = 0;
    let mut start = time::Instant::now();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        let metadata = entry.metadata()?;

        if !metadata.is_dir() {
            // get files only
            files.push(path);
        }

        if count % 999 == 0 {
            let elapsed = start.elapsed();

            println!("Files count {} in {}ms", count, elapsed.as_millis());
            start = time::Instant::now();
        }

        count += 1;
    }

    println!("Final files count {}", count);
    // Filter files if they are not Wasm binaries
    // Do so in parallel
    let filtered = reduce_binaries(state, &files)?;
    Ok(())
}

pub fn extract(state: Arc<State>, path: String) -> Result<Vec<PathBuf>, CliError> {
    let mut files = vec![];

    let mut count = 0;
    let mut start = time::Instant::now();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        let metadata = entry.metadata()?;

        if !metadata.is_dir() {
            // get files only
            files.push(path);
        }

        if count % 999 == 0 {
            let elapsed = start.elapsed();

            println!("Files count {} in {}ms", count, elapsed.as_millis());
            start = time::Instant::now();
        }

        count += 1;
    }

    println!("Final files count {}", count);
    // Filter files if they are not Wasm binaries
    // Do so in parallel
    let filtered = get_only_wasm(state, &files)?;
    Ok(filtered)
}

pub fn main() -> Result<(), errors::CliError> {

    let env = Env::default()
        //.filter_or("LOG_LEVEL", "trace")
        .filter("RUST_LOG")
        .write_style_or("LOG_STYLE", "never");



    Builder::from_env(env)
        .format(move |buf, record: &Record|  { 
            // Send to a diff file, depending on thread
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!("output{}.log", std::thread::current().name().unwrap()))
            .unwrap();
            let _ = file.write(&format!("[{}] [{}] <<<{}>>>\n", 
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
                record.module_path().unwrap_or(""),
                record.args()).into_bytes());
            Ok(())
        })
        .init();
      

    let yaml = load_yaml!("config.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let mut dbclientoptions = ClientOptions::parse(arg_or_error!(matches, "dbconn"))?;

    dbclientoptions.app_name = Some(arg_or_error!(matches, "dbname"));

    let mut credentials = Credential::default();
    credentials.password = Some(arg_or_error!(matches, "dbpass"));
    credentials.username = Some(arg_or_error!(matches, "dbuser"));

    dbclientoptions.credential = Some(credentials);

    let dbclient = Client::with_options(dbclientoptions)?;
    let mut state = State {
        dbclient: Some(dbclient.clone()),
        process: AtomicU32::new(0),
        error: AtomicU32::new(0),
        parsing_error: AtomicU32::new(0),
        collection_name: arg_or_error!(matches, "collection_name"),
        dbname: arg_or_error!(matches, "dbname"),
        out_folder: None,
    };

    match matches.subcommand() {
        ("extract", Some(args)) => {
            let reset = args.is_present("reset");
            if reset {
                println!("Reseting ");
                dbclient
                    .database(&arg_or_error!(matches, "dbname"))
                    .collection::<Meta>(&arg_or_error!(matches, "collection_name"))
                    .drop(None)?;
            }
            println!("Extracting...");
            extract(Arc::new(state), arg_or_error!(args, "folder"))?;
        }
        ("reduce", Some(args)) => {
            let reset = args.is_present("reset");
            if reset {
                println!("Reseting ");
                dbclient
                    .database(&arg_or_error!(matches, "dbname"))
                    .collection::<Meta>(&arg_or_error!(matches, "collection_name"))
                    .drop(None)?;
            }
            println!("Reducing...");
            state.out_folder = Some(arg_or_error!(args, "out"));
            reduce(Arc::new(state), arg_or_error!(args, "folder"))?;
        }
        ("export", Some(args)) => {
            println!("Exporting");
            let collection = dbclient
                .database(&arg_or_error!(matches, "dbname"))
                .collection::<Meta>(&arg_or_error!(matches, "collection_name"));

            let records = collection.find(None, None).unwrap();
            let mut outfile = std::fs::File::create(arg_or_error!(args, "out")).unwrap();

            if args.is_present("csv") {
                let mut writer = csv::Writer::from_writer(outfile);

                for record in records {
                    writer.serialize(record.unwrap()).unwrap();
                }

                writer.flush().unwrap()
            } else {
                let mut all = vec![];

                for record in records {
                    all.push(record.unwrap());
                }

                outfile
                    .write_all(serde_json::to_string_pretty(&all).unwrap().as_bytes())
                    .unwrap();
            }
        }
        ("clean", Some(_)) => {
            println!("Reseting ");
            dbclient
                .database(&arg_or_error!(matches, "dbname"))
                .collection::<Meta>(&arg_or_error!(matches, "collection_name"))
                .drop(None)?;
        }
        (c, _) => {
            todo!("Command {}", c);
        }
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use std::sync::atomic::AtomicU32;
    use std::sync::Arc;
    use std::time::Duration;

    use mongodb::options::{ClientOptions, Credential};
    use mongodb::sync::Client;

    use crate::meta::Meta;
    use crate::{extract, State};

    //#[test]
    pub fn test_extract() {
        let state = State {
            dbclient: None,
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            collection_name: "wasms".to_string(),
            dbname: "obfuscator".to_string(),
            out_folder: None,
        };
        extract(
            Arc::new(state),
            "../RQ1/all-binaries-metadata/all".to_string(),
        )
        .unwrap();
    }

    #[test]
    pub fn test_extract2() {
        let state = State {
            dbclient: None,
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            collection_name: "wasms".to_string(),
            dbname: "obfuscator".to_string(),
            out_folder: None,
        };
        extract(Arc::new(state), "./".to_string()).unwrap();
    }

    pub fn test_db() {
        let mut dbclientoptions = ClientOptions::parse("mongodb://localhost:27017").unwrap();

        dbclientoptions.app_name = Some("obfuscator".to_string());
        dbclientoptions.connect_timeout = Some(Duration::from_millis(500));
        dbclientoptions.server_selection_timeout = Some(Duration::from_millis(500));

        let mut credentials = Credential::default();
        credentials.password = Some("admin".to_string());
        credentials.username = Some("admin".to_string());

        dbclientoptions.credential = Some(credentials);

        let client = Client::with_options(dbclientoptions).unwrap();

        // List the names of the databases in that deployment.
        for db_name in client.list_database_names(None, None).unwrap() {
            println!("{}", db_name);
        }
    }

    #[test]
    pub fn test_csv() {
        let mut writer = csv::Writer::from_writer(std::io::stdout());
        let m = Meta::new();

        // writer.write_record(&["a"]).unwrap();

        writer.serialize(m).unwrap();
        writer.flush().unwrap();
    }
}
