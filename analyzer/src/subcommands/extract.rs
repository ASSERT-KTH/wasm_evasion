// Extract subcommand logic

use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashMap,
    fs,
    io::Read,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
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

pub fn get_wasm_info(
    state: Arc<State>,
    chunk: Vec<PathBuf>,
    print_meta: bool,
) -> AResult<Vec<PathBuf>> {
    if chunk.is_empty() {
        return Ok(vec![]);
    }

    let dbclient = state.dbclient.as_ref().unwrap().clone();
    let depth = state.depth.clone();
    let outfolder = state.out_folder.clone().unwrap_or("metas".into());
    let patch = state.patch_metadata;

    let mut time = time::Instant::now();
    'iter: for f in chunk.iter() {
        let mut file = fs::File::open(f)?;

        let name = f.file_name().unwrap().to_str().unwrap().to_string();

        let entry: AResult<Meta> = dbclient.get(&name.clone());

        match entry {
            Err(e) => {
                log::trace!(
                    "Extracting {} {}",
                    name.clone(),
                    state.process.load(Ordering::Relaxed)
                );
            }
            Ok(d) => {
                state.process.fetch_add(1, Ordering::SeqCst);
                log::trace!(
                    "{} already processed {}",
                    state.process.load(Ordering::Relaxed),
                    dbclient.f
                );
                continue 'iter;
            }
        }

        // Filter first the header to check for Wasm
        let mut buf = [0; 4];
        let r = file.read_exact(&mut buf);

        match r {
            Err(e) => {
                log::error!("{}", e);
                continue 'iter;
            }
            Ok(_) => {}
        }

        match &buf {
            b"\0asm" => {
                //println!("Wasm !");

                let mut meta = meta::Meta::new();
                meta.id = name.clone();
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

                        if state
                            .parsing_error
                            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
                            % 10
                            == 9
                        {
                            log::error!(
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
                        log::error!("{:#?}               Error {:?}", f, e);

                        if state
                            .error
                            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
                            % 10
                            == 9
                        {
                            log::error!("{} errors!", state.error.load(Ordering::Relaxed));
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

                let info = InfoExtractor::get_mutable_info(
                    &mut cp,
                    config,
                    state.depth,
                    state.seed,
                    state.sample_ratio,
                );
                match info {
                    Ok((mut info, mut mutations)) => {
                        // Save meta to_string mongodb
                        if let Some(client) = &state.dbclient {
                            for (m, map) in mutations.iter_mut() {
                                if map.len() > 0 {
                                    m.generic_map = Some(map.clone());

                                    info.mutations.push(m.clone());
                                }
                            }

                            log::debug!(
                                "Saving record for {} {}",
                                name.clone(),
                                state.process.load(Ordering::Relaxed)
                            );
                            match dbclient.set(&info.id.clone(), info) {
                                Ok(_) => {}
                                Err(e) => {
                                    log::error!("{:?}", e);
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

        if state
            .process
            .fetch_add(1, std::sync::atomic::Ordering::Acquire)
            % 100
            == 99
        {
            log::debug!(
                "{} processed {} in {}ms",
                state.process.load(Ordering::Relaxed),
                dbclient.f,
                time.elapsed().as_millis()
            );
            time = time::Instant::now();
        }
    }

    Ok(vec![])
}

pub fn get_only_wasm(
    state: Arc<State>,
    files: &Vec<PathBuf>,
    print_meta: bool,
) -> Result<Vec<PathBuf>, CliError> {
    let mut workers = vec![vec![]; NO_WORKERS];

    let elapsed = time::Instant::now();
    for (idx, file) in files.iter().enumerate() {
        workers[idx % NO_WORKERS].push(file.clone());
    }

    let cp = state.clone();
    let jobs = workers
        .into_iter()
        .enumerate()
        .map(|(_, x)| {
            let cp2 = cp.clone();
            spawn(move || get_wasm_info(cp2, x, print_meta))
        })
        .collect::<Vec<_>>();

    for j in jobs {
        let _ = j.join().map_err(|x| CliError::Any(format!("{:#?}", x)))?;
    }

    log::debug!(
        "{} processed {} in {}ms",
        state.process.load(Ordering::Relaxed),
        state.dbclient.as_ref().unwrap().f,
        elapsed.elapsed().as_millis()
    );
    log::error!(
        "{} parsing errors!",
        state.parsing_error.load(Ordering::Relaxed)
    );
    log::error!("{} errors!", state.error.load(Ordering::Relaxed));

    Ok(vec![])
}

pub fn extract(state: Arc<State>, path: String) -> Result<Vec<PathBuf>, CliError> {
    let mut files = vec![];

    let mut count = 0;
    let mut start = time::Instant::now();
    let meta = fs::metadata(path.clone())?;
    let mut print_meta = false;

    if meta.is_file() {
        files.push(PathBuf::from(path.clone()));
        print_meta = true;
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

            if count % 100 == 99 {
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
