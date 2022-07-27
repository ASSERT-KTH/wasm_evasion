// Extract subcommand logic

use std::{
    fs,
    io::Read,
    path::PathBuf,
    sync::{atomic::Ordering, Arc},
    thread::spawn,
    time,
};

use wasm_mutate::WasmMutate;

use crate::{
    errors::{AResult, CliError},
    info::InfoExtractor,
    meta::{self, Meta},
    State, NO_WORKERS,
};

pub fn get_wasm_info(state: Arc<State>, chunk: Vec<PathBuf>) -> AResult<Vec<PathBuf>> {
    if chunk.is_empty() {
        return Ok(vec![]);
    }

    for f in chunk.iter() {
        let mut file = fs::File::open(f)?;

        // Filter first the header to check for Wasm
        let mut buf = [0; 4];
        file.read_exact(&mut buf).unwrap();

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
                    Err(_e) => {
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

    println!();
    println!("{} processed", state.process.load(Ordering::Relaxed));
    println!(
        "{} parsing errors!",
        state.parsing_error.load(Ordering::Relaxed)
    );
    println!("{} errors!", state.error.load(Ordering::Relaxed));

    Ok(vec![])
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
