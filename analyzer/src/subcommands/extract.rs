// Extract subcommand logic

use std::{
    fs,
    io::Read,
    path::PathBuf,
    sync::{atomic::{Ordering, AtomicU32, AtomicBool}, Arc},
    thread::spawn,
    time, cell::RefCell, collections::HashMap, borrow::Borrow,
};

use mongodb::bson::Document;
use wasm_mutate::WasmMutate;

use crate::{
    errors::{AResult, CliError},
    info::InfoExtractor,
    meta::{self, Meta},
    State, NO_WORKERS,
};

pub fn get_wasm_info(state: RefCell<State>, chunk: Vec<PathBuf>) -> AResult<Vec<PathBuf>> {
    if chunk.is_empty() {
        return Ok(vec![]);
    }

    let dbclient = state.borrow().dbclient.as_ref().unwrap().clone();
    let dbname = state.borrow().dbname.clone();
    let collection_name = state.borrow().collection_name.clone();
    let outfolder = state.borrow().out_folder.clone().unwrap_or("metas".into());
    let br = state.borrow();

    'iter: for f in chunk.iter() {
        let mut file = fs::File::open(f)?;

        let name = f.file_name().unwrap().to_str().unwrap().to_string();

        // Check if it in the DB, continue if Some
        let db = dbclient.database(&dbname);
        let collection = db.collection::<Meta>(&collection_name);
        let mut filter = Document::new();
        filter.insert("parent", name.clone());

        let entry = collection.find_one(filter, None);

        match entry {
            Err(e) => {
                println!("{}", e);
            }
            Ok(d) => {
                match d {
                    Some(_) => {
                        continue 'iter;
                    }
                    None => {
                        print!("\nExtracting {} ", name);
                    }
                }
                
            }
        }
        // Filter first the header to check for Wasm
        let mut buf = [0; 4];
        let r = file.read_exact(&mut buf);

        match r {
            Err(e) => {
                println!("{}", e);
                continue 'iter;
            },
            Ok(_) => {

            }
        }

        match &buf {
            b"\0asm" => {
                //println!("Wasm !");

                let mut meta = meta::Meta::new();
                meta.id = name;
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

                        if state.borrow()
                            .parsing_error
                            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
                            % 9
                            == 0
                        {
                            println!(
                                "{} parsing errors!",
                                state.borrow().parsing_error.load(Ordering::Relaxed)
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
                        .borrow().error
                            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
                            % 9
                            == 0
                        {
                            println!("{} errors!", state.borrow().error.load(Ordering::Relaxed));
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
                    Err(_e) => {
                        
                        continue;
                    }
                    Ok(_) => {}
                }

                config.preserve_semantics(true);

                let mut cp = info?.clone();

                let info = InfoExtractor::get_mutable_info(&mut cp, config, br.depth);

                match info {
                    Ok((mut info, mut mutations)) => {
                        // Save meta to_string mongodb
                        if let Some(client) = &state.borrow().dbclient {
                            let db = client.database(&state.borrow().dbname);
                            let collection = db.collection::<Meta>(&state.borrow().collection_name);
                            // Add mutations but send mutations map to a file :)
                            for (m, map) in mutations.iter_mut() {

                                let dirname = format!("{}", outfolder);
                                std::fs::create_dir(dirname.clone());


                                let filename = format!("{}/{}.{}.meta.json", dirname, info.id, m.class_name);
                                std::fs::write(
                                    filename.clone(),
                                    serde_json::to_vec_pretty(map).unwrap()
                                ).unwrap();

                                m.map = filename.into();

                                info.mutations.push(
                                    m.clone()
                                );
                            }

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

        if state.borrow()
            .process
            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
            % 99
            == 0
        {
            println!("{} processed", state.borrow().process.load(Ordering::Relaxed));
        }
    }

    Ok(vec![])
}

pub fn get_only_wasm(state: RefCell<State>, files: &Vec<PathBuf>) -> Result<Vec<PathBuf>, CliError> {
    let mut workers = vec![vec![]; NO_WORKERS];

    for (idx, file) in files.iter().enumerate() {
        workers[idx % NO_WORKERS].push(file.clone());
    }

    let jobs = workers
        .into_iter()
        .enumerate()
        .map(|(_, x)| {
            let br = state.borrow();

            let t = State {
                dbclient: br.dbclient.clone(),
                collection_name: br.collection_name.clone(),
                dbname: br.dbname.clone(),
                process: AtomicU32::new(0),
                error: AtomicU32::new(0),
                parsing_error: AtomicU32::new(0),
                out_folder: br.out_folder.clone(),
                save_logs: br.save_logs.clone(),
                finish: AtomicBool::new(false),
                depth: br.depth.clone(),
            };

            spawn(move || get_wasm_info(RefCell::new(t), x))
        })
        .collect::<Vec<_>>();

    for j in jobs {
        let _ = j.join().map_err(|x| CliError::Any(format!("{:#?}", x)))?;
    }

    println!();
    println!("{} processed", state.borrow().process.load(Ordering::Relaxed));
    println!(
        "{} parsing errors!",
        state.borrow().parsing_error.load(Ordering::Relaxed)
    );
    println!("{} errors!", state.borrow().error.load(Ordering::Relaxed));

    Ok(vec![])
}

pub fn extract(state: RefCell<State>, path: String) -> Result<Vec<PathBuf>, CliError> {
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
