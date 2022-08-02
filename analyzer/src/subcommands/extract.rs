// Extract subcommand logic

use std::{
    fs,
    io::Read,
    path::PathBuf,
    sync::{atomic::{Ordering, AtomicU32, AtomicBool}, Arc},
    thread::spawn,
    time, cell::RefCell, collections::HashMap, borrow::Borrow,
};

use mongodb::bson::{Document, Bson, doc};
use wasm_mutate::WasmMutate;

use crate::{
    errors::{AResult, CliError},
    info::InfoExtractor,
    meta::{self, Meta},
    State, NO_WORKERS
};


pub fn get_wasm_info(state: RefCell<State>, chunk: Vec<PathBuf>, print_meta: bool) -> AResult<Vec<PathBuf>> {
    if chunk.is_empty() {
        return Ok(vec![]);
    }

    let dbclient = state.borrow().dbclient.as_ref().unwrap().clone();
    let dbname = state.borrow().dbname.clone();
    let collection_name = state.borrow().collection_name.clone();
    let depth = state.borrow().depth.clone();
    let outfolder = state.borrow().out_folder.clone().unwrap_or("metas".into());
    let br = state.borrow();
    let patch = state.borrow().patch_metadata;

    'iter: for f in chunk.iter() {
        let mut file = fs::File::open(f)?;

        let name = f.file_name().unwrap().to_str().unwrap().to_string();

        // Check if it in the DB, continue if Some
        let db = dbclient.database(&dbname);
        let collection = db.collection::<Document>(&collection_name);
        let mut filter = Document::new();
        filter.insert("id", name.clone());

        let entry = collection.find_one(filter.clone(), None);

        match entry {
            Err(e) => {
                log::error!("{}", e);
                if patch {
                    continue 'iter;
                }
            }
            Ok(d) => {
                match d {
                    Some(m) => {
                        log::debug!("Patching {:?}", patch);
                        if patch {
                            // Get the static info
                            // Filter first the header to check for Wasm
                            let mut buf = [0; 4];
                            let r = file.read_exact(&mut buf);

                            match r {
                                Err(e) => {
                                    log::error!("{}", e);
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
                                            log::error!("{:#?}               Parsing error {:?}", f, e);

                                            // Patch metadata
                                        }
                                        Ok(metadata) => {
                                            // continue
                                            //let mut patch = Document::new();
                                            let collection = db.collection::<Document>(&collection_name);
                                            

                                            match metadata {
                                                Err(e) => log::error!("{}", e),
                                                Ok(metadata) => {
                                                    let patch  = doc!{"$set" : 
                                                        {
                                                            "num_tags": metadata.num_tags,
                                                            "num_globals": metadata.num_globals,
                                                            "num_types": metadata.num_tpes,
                                                            "num_tables": metadata.num_tables,
                                                            "num_elements": metadata.num_elements,
                                                            "num_data": metadata.num_data,
                                                            "num_data_segments": metadata.num_data_segments,
                                                            "num_imports": metadata.num_imports,
                                                            "num_exports": metadata.num_exports
                                                        } 
                                                    };
                                                    //patch.insert("num_tags", metadata.num_tags);
                                                    let updater = collection.update_one(
                                                        m.clone() , patch, None);

                                                    match updater {
                                                        Err(e) => log::error!("{} m {:?}", e, m),
                                                        Ok(_) => {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                _ => {
                                 
                                }
                            }
                        }
                        continue 'iter;
                    }
                    None => {
                        log::debug!("\nExtracting {} ", name);
                    }
                }
                
            }
        }
        // Filter first the header to check for Wasm
        let mut buf = [0; 4];
        let r = file.read_exact(&mut buf);

        match r {
            Err(e) => {
                log::error!("{}", e);
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
                        log::error!("{:#?}               Parsing error {:?}", f, e);

                        if state.borrow()
                            .parsing_error
                            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
                            % 9
                            == 0
                        {
                            log::error!(
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
                        log::error!("{:#?}               Error {:?}", f, e);

                        if state
                        .borrow().error
                            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
                            % 9
                            == 0
                        {
                            log::error!("{} errors!", state.borrow().error.load(Ordering::Relaxed));
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

                let info = InfoExtractor::get_mutable_info(&mut cp, config, br.depth, br.seed, br.sample_ratio);

                match info {
                    Ok((mut info, mut mutations)) => {
                        // Save meta to_string mongodb
                        if let Some(client) = &state.borrow().dbclient {
                            let db = client.database(&state.borrow().dbname);
                            let collection = db.collection::<Meta>(&state.borrow().collection_name);
                            // Add mutations but send mutations map to a file :)
                            /*if ! print_meta {
                                for (m, map) in mutations.iter_mut() {

                                    let dirname = format!("{}", outfolder);
                                    std::fs::create_dir(dirname.clone());
                                    let filename = format!("{}/{}.{}.meta.json", dirname, info.id, m.class_name);
                                    std::fs::write(
                                        filename.clone(),
                                        serde_json::to_vec_pretty(map)?
                                    ).unwrap();

                                    m.map = (map.len(), filename.into());

                                    info.mutations.push(
                                        m.clone()
                                    );
                                    
                                }
                            } */


                            for (m, map) in mutations.iter_mut() {
                                
                                
                                if map.len() > 0 {
                                    m.generic_map = Some(map.clone());
                                    
                                    info.mutations.push(
                                        m.clone()
                                    );
                                }
                            }

                            let docs = vec![info.clone()];

                            match collection.insert_many(docs, None) {
                                Ok(_) => {
                                }
                                Err(e) => {
                                    log::error!("{:?}", e)
                                }
                            }
                        } else {
                            log::error!("Where is the client")
                        }
                    }
                    Err(e) => {
                        log::error!("{:?}", e)
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

    Ok(vec![])
}

pub fn get_only_wasm(state: RefCell<State>, files: &Vec<PathBuf>, print_meta: bool) -> Result<Vec<PathBuf>, CliError> {
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
                mutation_cl_name: br.mutation_cl_name.clone(),
                seed: br.seed.clone(),
                patch_metadata: br.patch_metadata.clone(),
                sample_ratio: br.sample_ratio.clone(),
            };

            spawn(move || get_wasm_info(RefCell::new(t), x, print_meta))
        })
        .collect::<Vec<_>>();

    for j in jobs {
        let _ = j.join().map_err(|x| CliError::Any(format!("{:#?}", x)))?;
    }

    log::debug!("{} processed", state.borrow().process.load(Ordering::Relaxed));
    log::error!(
        "{} parsing errors!",
        state.borrow().parsing_error.load(Ordering::Relaxed)
    );
    log::error!("{} errors!", state.borrow().error.load(Ordering::Relaxed));

    Ok(vec![])
}

pub fn extract(state: RefCell<State>, path: String) -> Result<Vec<PathBuf>, CliError> {
    let mut files = vec![];

    let mut count = 0;
    let mut start = time::Instant::now();
    let meta = fs::metadata(path.clone())?;
    let mut print_meta = false;

    if meta.is_file() {
        files.push(PathBuf::from(path.clone()));
        print_meta = true;
        count += 1;
    } 
    else {

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
    let filtered = get_only_wasm(state, &files, print_meta)?;
    Ok(filtered)
}
