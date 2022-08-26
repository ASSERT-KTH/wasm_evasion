// Extract subcommand logic

use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, Mutex,
    },
    thread::spawn,
    time::{self, Duration},
};

use stop_thread::kill_thread_graceful;
use wasm_mutate::WasmMutate;

use crate::{
    errors::{AResult, CliError},
    info::InfoExtractor,
    meta::{self, Meta},
    State, NO_WORKERS, subcommands::export::create_chunk,
};


pub fn get_single_wasm_info(f: &PathBuf, state: Arc<State>, sample: u32, seed: u64, stopsignal: Arc<AtomicBool>) -> AResult<()> {
    
    let mut file = fs::File::open(f)?;

    let name = f.file_name().unwrap().to_str().unwrap().to_string();

    let dbclient = state.dbclient.as_ref().unwrap().clone();
    let entry: AResult<Meta> = dbclient.get(&name.clone());

    // Add the Stop signal in the expensive places
    if stopsignal.load(Ordering::SeqCst) {
        log::error!("Stopping due to signal");
        return Err(CliError::ThreadTimeout)
    }

    match entry {
        Err(e) => {
            log::trace!(
                "Extracting {} {}",
                name.clone(),
                state.process.load(Ordering::Relaxed)
            );
        }
        Ok(d) => {
            return Ok(())
        }
    }

    // Filter first the header to check for Wasm
    let mut buf = [0; 4];
    let r = file.read_exact(&mut buf);

    match r {
        Err(e) => {
            log::error!("{}", e);
            return Ok(())
        }
        Ok(_) => {}
    }

    // Add the Stop signal in the expensive places
    if stopsignal.load(Ordering::Relaxed) {
        log::error!("Stopping due to signal");
        return Err(CliError::ThreadTimeout)
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

            // Add the Stop signal in the expensive places
            if stopsignal.load(Ordering::Relaxed) {
                log::error!("Stopping due to signal");
                return Err(CliError::ThreadTimeout)
            }

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
                    
                    return Ok(())
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
                    return Ok(());
                }
                _ => {
                    // continue
                }
            }

            // Get mutation info, TODO

            // Add the Stop signal in the expensive places
            if stopsignal.load(Ordering::Relaxed) {
                log::error!("Stopping due to signal");
                return Err(CliError::ThreadTimeout)
            }
            let mut config = WasmMutate::default();
            let stinfo = config
                .setup(&bindata)
                .map_err(|x| CliError::Any(format!("{:#?}", x)));

            // Add the Stop signal information the expensive places
            if stopsignal.load(Ordering::Relaxed) {
                log::error!("Stopping due to signal");
                return Err(CliError::ThreadTimeout)
            }
            match stinfo {
                Err(_e) => {
                    
                    return Ok(())
                }
                Ok(_) => {}
            }

            config.preserve_semantics(true);

            let mut cp = info?.clone();

            // Add the Stop signal in the expensive places
            if stopsignal.load(Ordering::Relaxed) {
                log::error!("Stopping due to signal");
                return Err(CliError::ThreadTimeout)
            }
            
            let info = InfoExtractor::get_mutable_info(
                &mut cp,
                config,
                state.depth,
                seed,
                sample,
                stopsignal.clone()
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
                                match e {
                                    CliError::ThreadTimeout => {
            
                                        return Err(CliError::ThreadTimeout)
                                    },
                                    _ => {
            
                                        log::error!("{:?}", e)
                                    }
                                }
                            }
                        }
                    } else {
                        log::error!("Where is the client")
                    }
                }
                Err(e) => {
                    match e {
                        CliError::ThreadTimeout => {

                            return Err(CliError::ThreadTimeout)
                        },
                        _ => {

                            log::error!("{:?}", e)
                        }
                    }
                }
            }
        }
        _ => {
            log::error!("\nJust discard {:?}\n", f);
        }
    }


    Ok(())
}

pub fn get_wasm_info(
    state: Arc<State>,
    chunk: Arc<Mutex<Vec<PathBuf>>>,
    workerid: u32,
    total: usize,
    snapsignal: Arc<Mutex<bool>>
) -> AResult<Vec<PathBuf>> {
    loop {
        if chunk.lock().unwrap().is_empty() {
            return Ok(vec![]);
        }

        let f = chunk.lock().unwrap().pop();
        if let Some(f) = f {
            let s = chunk.lock().unwrap().len();
            println!("worker {} takes {:?}. List size {}/{}", workerid, f, s, total);

            // Send this to a thread and create a monitor
            let mut waitfor = state.timeout as u64; // wait for x seconds, get from arguments
            log::debug!("Timeout {}", waitfor);
            let mut cp = state.clone();
            let mut sample = cp.sample_ratio;
            let mut seed = cp.seed;
            loop {
                // Check if it is time for snapshot
                {
                    let d = snapsignal.lock();

                    match d {
                        Err(e) => println!("{}", e),
                        Ok(_) => {

                        }
                    }
                }
                let movecp = cp.clone();
                let fcp = f.clone();
                let fcp2 = f.clone();
                let signal = Arc::new(AtomicBool::new(false));
                let signalcp = signal.clone();

                let s = chunk.lock().unwrap().len();
                log::debug!("Restarting thread. Worklist size {}/{}", s + 1 /* the one already working */, total);
                let time = time::Instant::now();
                let th = spawn(move || get_single_wasm_info(&fcp.clone(), movecp.clone(), sample, seed, signalcp));

                loop {
                    let lapsed = time.elapsed().as_secs();
                    
                    if th.is_finished() { 
                        break
                    }
                    if lapsed > waitfor {
                        signal.store(true, Ordering::SeqCst);
                        break
                    }
                }
                //std::thread::sleep(Duration::from_secs(waitfor));
                //log::debug!("Thread for {} is finished", fcp2.clone().display());
                let r = th.join().unwrap();
                log::debug!("Result after {}s", time.elapsed().as_secs());

                match r {
                    Err(e) => {
                        match e {
                            CliError::ThreadTimeout => {
                                let lapsed = time.elapsed().as_secs();
                                log::warn!("Thread is taking to much ({}s) {} {}, setting sample to 1/{} and restarting",lapsed, fcp2.clone().display(), e, sample*2);
                                signal.store(false, Ordering::SeqCst);
                                sample = sample * 2;
                                seed += 1; // like a nonce
                                if sample > 1024 {
                                    log::error!("The binary {} cannot be processed", fcp2.clone().display());
                                    break;
                                }
                            }
                            e => {
                                // Any other error break
                                log::error!("Error {}", e);
                                break
                            }
                        }
                    },
                    Ok(_) => {
                         state
                            .process
                            .fetch_add(1, std::sync::atomic::Ordering::Acquire);
                            break
                    }
                }
                
            }
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
    let cp2 = state.clone();
    let filtered = files.iter()
        .filter(move |&f| {
            let ccp = cp2.clone();
            ccp.dbclient.as_ref().unwrap().get::<String, Meta>(&f.file_name().unwrap().to_str().unwrap().to_string()).is_err()
        })
        .map(|f|f.clone())
        .collect::<Vec<_>>();
    println!("Missing {}", filtered.len());

    state.process.store((files.len() - filtered.len()) as u32, Ordering::SeqCst);

    let alljobs = Arc::new(Mutex::new(filtered.clone()));

    let mut jobs = vec![];
    let total = filtered.len();
    let snapsignal = Arc::new(Mutex::new(true));
    for j in 0..NO_WORKERS {
        let cp2 = cp.clone();
        let cp3 = alljobs.clone();
        let cp4 = snapsignal.clone();
        let th = spawn(move || get_wasm_info(cp2, cp3, j as u32, total, cp4));
        
        jobs.push(th);
    }

    // Create snapshot thread 
    let stopsignal = Arc::new(AtomicBool::new(true));
    log::debug!("Snapshot {:?} {:?}", state.snapshot, state.snapshot_time);
    if let Some(snapshotfile) = &state.snapshot  {

        if let Some(snaptime) = &state.snapshot_time {

            let stime = snaptime.clone();
            let snapfile = snapshotfile.clone();
            let stopsignalcp = stopsignal.clone();
            let statecp = state.clone();
            log::debug!("Creating snapshot thread");
            let snapth = spawn(move || {
                loop {
                    // Sleep the interval time
                    std::thread::sleep(Duration::from_secs(stime as u64));
                    let d = snapsignal.lock().unwrap();
                    log::debug!("Saving snapshot {}", snapfile.clone() );
                    
                    let mut outfile = std::fs::File::create(snapfile.clone()).unwrap();

                    outfile.write_all(
                        "id,num_tags,num_functions,num_globals,num_tables,num_elements,num_data,num_types,num_memory,num_instructions,class_name,mutable_count\n".as_bytes()
                     ).unwrap();

                     for m in statecp.dbclient.as_ref().unwrap().get_all::<Meta>().unwrap() {
                        let ch = create_chunk(m, 2);
                        outfile.write_all(&ch.as_bytes()).unwrap();
                     }
                     
                    log::debug!("Saved {:?}", d);

                    if !stopsignalcp.load(Ordering::Relaxed) {
                        break
                    }
                    
                }
                let mut outfile = std::fs::File::create(snapfile.clone()).unwrap();

                outfile.write_all(
                    "id,num_tags,num_functions,num_globals,num_tables,num_elements,num_data,num_types,num_memory,num_instructions,class_name,mutable_count\n".as_bytes()
                 ).unwrap();

                 for m in statecp.dbclient.as_ref().unwrap().get_all::<Meta>().unwrap() {
                    let ch = create_chunk(m, 2);
                    outfile.write_all(&ch.as_bytes()).unwrap();
                 }
                 
                log::debug!("Saved");
                // One last time after all finished
            });

        }
    };

    for j in jobs {
        let _ = j.join().map_err(|x| CliError::Any(format!("{:#?}", x)))?;
    }
    stopsignal.store(false, Ordering::SeqCst);

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
