use std::{
    fmt::Display,
    fs,
    io::Read,
    path::PathBuf,
    str::FromStr,
    sync::{atomic::{Ordering, AtomicU32, AtomicBool}, Arc},
    time, borrow::Borrow, rc::Rc, cell::RefCell,
};

use anyhow::Context;
use mongodb::bson::Document;
use tempfile::NamedTempFile;
use wasm_shrink::{IsInteresting, WasmShrink};

use crate::{
    errors::{AResult, CliError},
    meta::{self, Meta},
    Hasheable, Printable, State, NO_WORKERS,
};

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

pub fn reduce_single_binary(state: RefCell<State>, chunk: Vec<PathBuf>) -> AResult<()> {
    log::debug!("reducing {} binaries", chunk.len());

    let outfolder = state.borrow().out_folder.as_ref().unwrap().clone();
    let dbclient = state.borrow().dbclient.as_ref().unwrap().clone();
    let dbname = state.borrow().dbname.clone();
    let collection_name = state.borrow().collection_name.clone();

    'iter: for f in chunk.iter() {

        log::debug!("{:?}", state.borrow().finish.load(Ordering::Relaxed));
        if state.borrow().finish.load(Ordering::Relaxed) {
            break;
        }
        let mut file = fs::File::open(f)?;

        let name = f.file_name().unwrap().to_str().unwrap().to_string();

        // Check if it in the DB, continue if so
        let db = dbclient.database(&dbname);
        let collection = db.collection::<Meta>(&collection_name);
        let mut filter = Document::new();
        filter.insert("parent", name.clone());

        let entry = collection.find_one(filter, None);
        
        match entry {
            Err(e) => {
                log::error!("{}", e);
            }
            Ok(d) => {
                match d {
                    Some(_) => {
                        log::debug!("\rSkipping {}", name);
                        if state.borrow()
                            .process
                            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
                            % 99
                            == 0
                        {
                            log::debug!("\n{} processed", state.borrow().process.load(Ordering::Relaxed));
                        }
                        continue 'iter;
                    }
                    None => {
                        log::debug!("\nReducing {} ", name);
                    }
                }
                
            }
        }

        // Filter first the header to check for Wasm
        let mut buf = [0; 4];
        let r = &file.read_exact(&mut buf);

        match r {
            Err(e) => {
                log::error!("{}", e);
                continue 'iter;
            },
            Ok(_) => {

            }
        }

        match &buf {
            // Filter first the header to check for Wasm
            b"\0asm" => {
                let mut meta = meta::Meta::new();
                meta.id = name.clone();
                // Get size of the file
                let fileinfo = fs::metadata(f)?;
                meta.size = fileinfo.len() as usize;

                // Parse Wasm to get more info
                let bindata = fs::read(f)?;
                let cp = bindata.clone();

                let mut reducer = WasmShrink::default();
                let reducer = reducer.attempts(10000);
                //let reducer = reducer.allow_empty(true);

                let output =
                    PathBuf::from_str(&format!("{}/{}.shrunken.wasm", outfolder, name)).unwrap();
                let logs =
                    PathBuf::from_str(&format!("{}/{}.shrunken.logs", outfolder, name)).unwrap();

                // copy the original in the folder to get it as the new shrunked binary

                std::fs::write(output.clone(), bindata.clone()).unwrap();

                log::debug!("Start =========== {}", name);

                let initial_size = cp.len();
                let r = reducer
                    .on_new_smallest(Some(Box::new({
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

                            Ok(())
                        }
                    })))
                    .run(cp, move |wasm| Ok(Interesting(wasm.len() > 8)));

                match r {
                    Err(e) => {
                        log::debug!("Error {}", e);
                        if state.borrow().save_logs {

                            log::debug!("Saving logs");
                            let name = std::thread::current();
                            let name  = name.name().unwrap();
                            let log_file = format!("output{}.log", name);
                            let r = std::fs::rename(log_file, logs.clone());

                            match r {
                                Err(e) => log::error!("{}", e),
                                Ok(_) => {

                                }
                            }
                        }
                        if state.borrow()
                            .error
                            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
                            % 9
                            == 0
                        {
                            log::error!("{} errors!", state.borrow().error.load(Ordering::Relaxed));
                        }
                        continue 'iter;
                    }
                    Ok(_i) => {
                        // Save logs if flag is set
                        if state.borrow().save_logs {

                            log::debug!("Saving logs");
                            let name = std::thread::current();
                            let name  = name.name().unwrap();
                            let log_file = format!("output{}.log", name);
                            let r = std::fs::rename(log_file, logs.clone());

                            match r {
                                Err(e) => log::error!("{}", e),
                                Ok(_) => {

                                }
                            }
                        }
                    }
                };

                let mut meta = meta::Meta::new();
                meta.id = output.display().to_string();

                let bindata = loop {
                    let bindata = fs::read(output.clone());

                    match bindata {
                        Err(e) => {
                            log::error!("{}", e);
                            continue 'iter;
                        }
                        Ok(r) => break r,
                    }
                };

                // Get size of the file
                meta.tpe = "canonical".to_string();
                meta.hash = bindata.to_vec().hash256().fmt1();
                meta.parent = Some(name.clone());
                meta.size = bindata.len();
                meta.logs = logs.display().to_string();
                meta.description = format!(
                    "seed: {}, fuel: {}, ratio {}",
                    0,
                    1000,
                    bindata.len() as f32 / initial_size as f32
                );

                let db = dbclient.database(&dbname);
                let collection = db.collection::<Meta>(&collection_name);

                let docs = vec![meta];

                match collection.insert_many(docs, None) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("{}", e);
                        std::panic::panic_any(e)
                    }
                }
            }
            _ => {
                log::error!("\nJust discard {:?}\n", f);
            }
        }

        if state.borrow()
            .process
            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
            % 99
            == 0
        {
            log::debug!("{} processed", state.borrow().process.load(Ordering::Relaxed));
        }
    }

    Ok(())
}

pub fn reduce_binaries(state: RefCell<State>, files: &Vec<PathBuf>) -> Result<(), CliError> {
    let mut workers = vec![vec![]; NO_WORKERS];

    for (idx, file) in files.iter().enumerate() {
        workers[idx % NO_WORKERS].push(file.clone());
    }

    let jobs = workers
        .into_iter()
        .enumerate()
        .map(|(i, x)| {
            let br = state.borrow();

            let t = State {
                dbclient: br.dbclient.clone(),
                collection_name: br.collection_name.clone(),
                mutation_cl_name: br.mutation_cl_name.clone(),
                dbname: br.dbname.clone(),
                process: AtomicU32::new(0),
                error: AtomicU32::new(0),
                parsing_error: AtomicU32::new(0),
                out_folder: br.out_folder.clone(),
                save_logs: br.save_logs.clone(),
                finish: AtomicBool::new(false),
                depth: br.depth.clone(),
                seed: br.seed.clone(),
                sample_ratio: br.sample_ratio.clone(),
            };

            std::thread::Builder::new()
                .name(format!("t{}", i))
                .stack_size(32 * 1024 * 1024 * 1024) // 320 MB
                .spawn(move || reduce_single_binary(RefCell::new(t), x))
                .unwrap()
        })
        .collect::<Vec<_>>();


    // Capture ctrl-c signal
    /* 
    ctrlc::set_handler(move|| {
        println!("received Ctrl+C! Finishing up");
        t.borrow().finish.store(true,Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");*/

    for j in jobs {
        let _ = j.join().map_err(|x| CliError::Any(format!("{:#?}", x)))?;
    }

    log::debug!("{} processed", state.borrow().process.load(Ordering::Relaxed));
    log::error!(
        "{} parsing errors!",
        state.borrow().parsing_error.load(Ordering::Relaxed)
    );
    log::error!("{} errors!", state.borrow().error.load(Ordering::Relaxed));

    Ok(())
}

pub fn reduce(state: RefCell<State>, path: String) -> AResult<()> {
    log::debug!("Creating folder if it doesn't exist");

    let outf = &state.borrow().out_folder;
    let outf = outf.as_ref().unwrap();


    std::fs::create_dir(outf.clone()); // Ignore if error since it's already created

    let mut files = vec![];

    let mut count = 0;
    let mut start = time::Instant::now();

    let meta = fs::metadata(path.clone())?;

    if meta.is_file() {
        files.push(PathBuf::from(path.clone()));
        count += 1;
    } else {
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

                log::debug!("Files count {} in {}ms", count, elapsed.as_millis());
                start = time::Instant::now();
            }

            count += 1;
        }
    }

    log::debug!("Final files count {}", count);
    // Filter files if they are not Wasm binaries
    // Do so in parallel
    let br = state.borrow();

    let t = State {
        dbclient: br.dbclient.clone(),
        collection_name: br.collection_name.clone(),
        mutation_cl_name: br.mutation_cl_name.clone(),
        dbname: br.dbname.clone(),
        process: AtomicU32::new(0),
        error: AtomicU32::new(0),
        parsing_error: AtomicU32::new(0),
        out_folder: br.out_folder.clone(),
        save_logs: br.save_logs.clone(),
        finish: AtomicBool::new(false),
        depth: br.depth.clone(),
        seed: br.seed.clone(),
        sample_ratio: br.sample_ratio.clone(),
    };
    reduce_binaries(RefCell::new(t), &files)?;
    Ok(())
}
