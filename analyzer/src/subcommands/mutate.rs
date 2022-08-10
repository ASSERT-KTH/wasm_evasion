use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, fs, io::{Read, Write}, process::ExitStatus, collections::HashSet, path::Path, os::unix::net::UnixListener, thread::spawn};
use anyhow::Context;
use rand::{rngs::SmallRng, SeedableRng, Rng};
use sled::Mode;
use wasm_mutate::WasmMutate;
use core::hash::Hash;

use crate::{errors::{AResult, CliError}, State, SOCKET_PATH, send_signal_to_probes_socket};
use std::thread;

#[derive(Debug)]
pub enum MODE {
    SEQUENTIAL,
    BISECT(u32,u32)
}

fn open_socket() -> AResult<String> {

    let outfile = format!("probes.logs.txt");
    let mut f = fs::File::create(outfile.clone())?;

    log::debug!("Opening probes sockets in {}", SOCKET_PATH);

    let socket = Path::new(SOCKET_PATH);

    if socket.exists() {
        log::debug!("File exist");
        fs::remove_file(SOCKET_PATH)?;
        // fs::unlink(&socket)?
    }

    let stream = UnixListener::bind(&socket)?;

    log::debug!("Socket waiting for probe messages");

    'always_listening: loop {

        for (mut stream, _) in stream.accept() {
            let mut buff =  String::new();
            stream.read_to_string(&mut buff)?;
            let splat = buff.split("][").collect::<Vec<_>>();

            match splat[0]  {
                "STOP" => { break 'always_listening }
                "SAVE" => {
                    f.write_all(&"NEW VARIANT\n".as_bytes());
                    f.write_all(&splat[1].as_bytes());
                    f.write_all(&"\n".as_bytes());
                    f.flush()?;
                }
                _ => {
                    buff.push_str(&"\n");
                    f.write_all(&buff.as_bytes())?;
                    f.flush()?;
                }
            }

        }
    }

    Ok(outfile)
}

pub fn mutate_bisect(state: Arc<State>, path: String, command: String, args: Vec<String>,attemps: u32, peek_count: u32, seed: u64, tree_size: u32) -> AResult<()> {


    
    let mut dimensions = vec![
        (0, tree_size*2, tree_size, "tree_size"),
        (0, attemps*2, attemps, "attempts"),
        (0, peek_count*2, peek_count, "peek")
    ];
    let mut finished = vec![
        false,
        false,
        false
    ];
    let mut dimindex = 0;
    loop {
        if finished.iter().all(|f|*f) {
            break;
        }
        // Iterate through dimensions until all finished
        loop {
            let mut dim = &dimensions[dimindex];            
            println!("dim {:?}", dim);
            match dimindex {
                0 => {
                    let ( mut low, mut high, mut last_success,  _)  = dim;
                    if high - low <= 1 {
                        finished[dimindex] = true;
                        break
                    }
                    let tsize = (low + high) / 2;
                    log::debug!("Going mutate for tree_size {}", tsize);
                    let statecp = state.clone();
                    let pathcp = path.clone();
                    let commandcp = command.clone();
                    let argsclone = args.clone();
                    let lastattempts = dimensions[1].2;
                    let lastpeek = dimensions[2].2;
            
                    let (elapsed, interesting_count) = mutate_sequential(statecp, pathcp, commandcp, argsclone, lastattempts, true, lastpeek as u64, seed, tsize)?;
            
                    log::debug!("Elapsed {}, interesting count {}", elapsed, interesting_count);
                    if interesting_count == 0 {
                        // Go lower
                        let dim = ((low+high)/2, high, last_success, "tree");
                        dimensions[dimindex] = dim;
                    
                    } else {
                        // Go higher
                        let dim = (low, (low+high)/2, tsize, "tree");
                        dimensions[dimindex] = dim;
                    }
                }
                1 => {
                    let (mut low, mut high, mut last_success, _) = dim;
                    if high - low <= 1 {
                        finished[dimindex] = true;
                        break
                    }
                    let attempts = (low + high) / 2;
                    log::debug!("Going mutate for attempt {}", attempts);
                    let statecp = state.clone();
                    let pathcp = path.clone();
                    let commandcp = command.clone();
                    let argsclone = args.clone();
                    let lastsize = dimensions[0].2;
                    let lastpeek = dimensions[2].2;
            
                    let (elapsed, interesting_count) = mutate_sequential(statecp, pathcp, commandcp, argsclone, attempts, true, lastpeek as u64, seed, lastsize)?;
            
                    log::debug!("Elapsed {}, interesting count {}", elapsed, interesting_count);
                    if interesting_count == 0 {
                        
                        // Go lower
                        let dim = ((low+high)/2, high, last_success, "attempts");
                        dimensions[dimindex] = dim;
                        
                    } else {
                        // Go higher
                        let dim = (low, (low+high)/2, attempts, "attempts");
                        dimensions[dimindex] = dim;
                    }
                }
                2 => {
                    let (mut low, mut high, mut last_success, _) = dim;
                    if high - low <= 1 {
                        finished[dimindex] = true;
                        break
                    }
                    let peeks = (low + high) / 2;
                    log::debug!("Going mutate for peeks {}", peeks);
                    let statecp = state.clone();
                    let pathcp = path.clone();
                    let commandcp = command.clone();
                    let argsclone = args.clone();
                    let lastsize = dimensions[0].2;
                    let lastattemps = dimensions[1].2;
                    
            
                    let (elapsed, interesting_count) = mutate_sequential(statecp, pathcp, commandcp, argsclone, lastattemps, true, peeks as u64, seed, lastsize)?;
            
                    log::debug!("Elapsed {}, interesting count {}", elapsed, interesting_count);
                    if interesting_count == 0 {
                        
                        // Go lower
                        let dim = ((low+high)/2, high, last_success, "peek");
                        dimensions[dimindex] = dim;
                        
                    } else {
                        // Go higher
                        let dim = (low, (low+high)/2, peeks, "peek");
                        dimensions[dimindex] = dim;
                    }
                }
                _ => {
                    panic!("Invalid dimension")
                }
            }
            
        }
        dimindex = (dimindex + 1) % dimensions.len();
    }
    println!("Minimum config {:?}", dimensions);
    Ok(())
}


pub fn mutate_sequential(state: Arc<State>, path: String, command: String, args: Vec<String>,attemps: u32, exit_on_found: bool, peek_count: u64, seed: u64, tree_size: u32) -> AResult<(u32, u32)> {
    log::debug!("Mutating binary {}", path);
    let th = spawn(move || {
        open_socket()
    });


    let mut file = fs::File::open(path.clone())?;
    let session_folder = format!("{}/{}_{}_a{}_p{}_ts{}", state.dbclient.as_ref().unwrap().f, 
        command.replace("/", "_"), 
        args.iter().map(|f| f.replace("/", "_")).collect::<Vec<_>>().join("_"), 
        attemps, 
        peek_count,tree_size);
    fs::create_dir(session_folder.clone());
    log::debug!("Saving session in {}", session_folder.clone());


    // Filter first the header to check for Wasm
    let mut buf = [0; 4];
    let r = &file.read_exact(&mut buf)?;

    let mut bin = match &buf {
        // Filter first the header to check for Wasm
        b"\0asm" => {
            fs::read(path.clone())?
        }
        _ => {
            return Err(CliError::Any("Invalid Wasm header".into()))
        }
    };
    
    let mut elapsed = 0;
    let mut gn = SmallRng::seed_from_u64(seed);

    let mut seen: HashSet<blake3::Hash> = HashSet::new();
    let mut collision_count = 0;
    let mut interesting_count = 0;
    let mut parent = String::new();
    'attempts: while elapsed < attemps {
        // mutated = m 
        let s = gn.gen();
        let mut config = WasmMutate::default();
        config.preserve_semantics(true);
        config.peephole_size(tree_size);
        config.seed(s);
        //let stinfo = config
        //.setup(&bin)
        //.map_err(|x| CliError::Any(format!("{:#?}", x)))?;

        let cp = bin.clone();

        let m = config.run(&cp);

        let mut worklist = vec![];
        match m {
            Err(e) => {
                log::error!("{}", e)
            },
            Ok(it) => {
                for (idx, b) in it.enumerate().take(peek_count as usize) {
                    match b {
                        Err(e) => {
                            log::error!("{}", e)
                        },
                        Ok(b) => {
                            // FIXME, Prevent to save a previous seen binary
                            
                            // TODO, validate as well
                            let hash = blake3::hash(&b.clone());

                            if ! seen.contains(&hash) {
                                worklist.push((
                                    b.clone(), idx
                                ));
                                seen.insert(hash);
                            } else {
                                collision_count += 1;
                            }
                            
                        }
                    }
                }
            }
        }

        while let Some((newbin, idx)) = worklist.pop() {

            swap(&mut bin, newbin.clone());
            // TODO Move this to parallel execution
            let (r, stdout, stderr) = check_binary(bin.clone(), command.clone(), args.clone())?;

            let (interesting, out) = if r.success() {
                
                let fname = format!("{session_folder}/non_interesting");                
                fs::create_dir(fname.clone());
                (false, fname)
            } else {
                interesting_count += 1;   
                let fname = format!("{session_folder}/interesting");   
                fs::create_dir(fname.clone());                   (true, fname)
            };
    
            let fname = format!("{out}/e{:0width$}_s{}_i{}", elapsed,  s, idx, width=10);
            fs::create_dir(fname.clone());
            fs::write(format!("{}/stderr.txt", fname.clone()), &stderr)?;

            let mut f = fs::File::create(format!("{}/iteration_info.txt", fname.clone()))?;
            f.write_all(format!("seed: {}\n", s).as_bytes())?;
            f.write_all(format!("attempts: {}\n", attemps).as_bytes())?;
            f.write_all(format!("elapsed: {}\n", elapsed).as_bytes())?;
            f.write_all(format!("idx: {}\n", idx).as_bytes())?;
            f.write_all(format!("interesting: {}\n", interesting).as_bytes())?;
            f.write_all(format!("variant_size: {}\n", newbin.len()).as_bytes())?;
            f.write_all(format!("parent: {}\n", parent).as_bytes())?;
            // TODO Add Meta info of the variant ?

            fs::write(format!("{}/stdout.txt", fname.clone()), &stdout)?;
            fs::write(format!("{}/variant.wasm", fname), &newbin)?;

            send_signal_to_probes_socket(format!("SAVE][{}/probes.logs.txt", fname));
            // Send filena name
            parent = fname;

            if exit_on_found && interesting {
                break 'attempts;
            }
        }

        elapsed += 1;


        if elapsed % 10 == 9 {
            println!("Elapsed {}/{}. Collision count {}. Interesting count {}", elapsed, attemps, collision_count, interesting_count);
        }
    }
    
    println!("Elapsed {}/{}. Collision count {}. Interesting count {}", elapsed, attemps, collision_count, interesting_count);

    // Now save the session to a folder ?
    send_signal_to_probes_socket("STOP".into());
    let outfile = th.join().unwrap()?;
    Ok((elapsed, interesting_count))
}

pub fn mutate(state: Arc<State>, path: String, command: String, args: Vec<String>,attemps: u32, exit_on_found: bool, peek_count: u64, seed: u64, tree_size: u32, mode: MODE) -> AResult<()> {
    
    match mode {
        MODE::SEQUENTIAL => {
            mutate_sequential(state, path, command, args, attemps, exit_on_found, peek_count, seed, tree_size)?;
        }
        MODE::BISECT(_, _) => {
            mutate_bisect(state, path, command, args, attemps, peek_count as u32, seed, tree_size)?;
        }
    };

    Ok(())
}

fn check_binary(bin: Vec<u8>, command: String, args: Vec<String>) -> AResult<(ExitStatus, Vec<u8>, Vec<u8>)> {
// Write file to tmparg
    std::fs::write("temp.wasm", &bin.clone()).unwrap();
    let output = std::process::Command::new(&command)
        .args(args.clone())
        .arg("temp.wasm")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| CliError::Any(format!("Failed to run command {} args {:?}. Error {}", command, args, e)));

    let output = output?;

    Ok((output.status, output.stdout, output.stderr))
}

fn swap(a: &mut Vec<u8>, b:  Vec<u8>) {
    *a = b;
}


#[cfg(test)]
pub mod tests {
    use std::sync::{atomic::{AtomicU32, AtomicBool}, Arc};

    use env_logger::{Env, Builder};
    use sled::Mode;

    use crate::{db::DB, State};

    use super::{mutate, MODE};



    #[test]
    pub fn test() {
        let env = Env::default()
            //.filter_or("LOG_LEVEL", "bench,analyzer,wasm-mutate=debug")
            .write_style_or("LOG_STYLE", "always");
        Builder::from_env(env).init();  

        let state = State {
            dbclient: Some(DB::new("test_db", 10000).unwrap()),
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            out_folder: None,
            save_logs: false,
            finish: AtomicBool::new(false),
            depth: 2,
            sample_ratio: 1,
            patch_metadata: false,
            seed: 0,
            timeout: 5,
            snapshot: None,
            snapshot_time: None
        };

        mutate(Arc::new(state), "tests/1.wasm".into(), "/bin/bash".into(),  vec![ 
            "tests/oracle_size.sh".into()
        ],600,false, 1, 0, 1, MODE::SEQUENTIAL).unwrap()
    }
}