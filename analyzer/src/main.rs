use clap::{load_yaml, value_t, App};
use errors::{AResult, CliError};
use info::InfoExtractor;
use std::{
    borrow::{Borrow, BorrowMut},
    fs,
    io::Read,
    path::PathBuf,
    process,
    rc::Rc,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    thread::spawn, time,
};

use mongodb::options::{ClientOptions, Credential};
use mongodb::sync::Client;

use crate::meta::Meta;

mod errors;
pub mod info;
mod meta;

#[derive(Debug)]
pub struct State {
    dbclient: Option<Client>,
    collection_name: String,
    dbname: String,
    process: AtomicU32,
    error: AtomicU32,
    parsing_error: AtomicU32,
}

pub fn get_wasm_info(state: Arc<State>, chunk: Vec<PathBuf>) -> AResult<Vec<PathBuf>> {
    if chunk.len() == 0 {
        return Ok(vec![]);
    }

    for f in chunk.iter() {
        let mut file = fs::File::open(f)?;

        // Filter first the header to check for Wasm
        let mut buf = [0; 4];
        file.by_ref().read_exact(&mut buf)?;

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

                let info =
                    std::panic::catch_unwind(move || InfoExtractor::get_info(&bindata, &mut meta));

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

                // Save meta to mongodb

                if let Some(client) = &state.dbclient {
                    let db = client.database(&state.dbname);
                    let collection = db.collection::<Meta>(&state.collection_name);

                    let docs = vec![info?];

                    collection.insert_many(docs, None)?;
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
    let no_workers = 10;
    let mut workers = vec![vec![]; no_workers];

    for (idx, file) in files.iter().enumerate() {
        workers[idx % no_workers].push(file.clone());
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

pub fn extract(state: Arc<State>, path: String) -> Result<Vec<PathBuf>, CliError> {
    // TODO traverse the folder recursively

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
    let yaml = load_yaml!("config.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let mut dbclientoptions = ClientOptions::parse(arg_or_error!(matches, "dbconn"))?;

    dbclientoptions.app_name = Some(arg_or_error!(matches, "dbname"));

    let mut credentials = Credential::default();
    credentials.password = Some(arg_or_error!(matches, "dbpass"));
    credentials.username = Some(arg_or_error!(matches, "dbuser"));

    dbclientoptions.credential = Some(credentials);

    let dbclient = Client::with_options(dbclientoptions)?;
    let state = State {
        dbclient: Some(dbclient.clone()),
        process: AtomicU32::new(0),
        error: AtomicU32::new(0),
        parsing_error: AtomicU32::new(0),
        collection_name: arg_or_error!(matches, "collection_name"),
        dbname: arg_or_error!(matches, "dbname"),
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
        ("export", Some(args)) => {
            println!("Exporting")
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
}
