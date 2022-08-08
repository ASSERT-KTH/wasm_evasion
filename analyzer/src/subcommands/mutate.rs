use std::{sync::Arc, fs, io::{Read, Write}, process::ExitStatus, collections::HashSet};
use anyhow::Context;
use rand::{rngs::SmallRng, SeedableRng, Rng};
use wasm_mutate::WasmMutate;
use core::hash::Hash;
use crate::{errors::{AResult, CliError}, State};


pub fn mutate(state: Arc<State>, path: String, command: String, args: Vec<String>,attemps: u32, exit_on_found: bool, peek_count: u64, seed: u64) -> AResult<()> {
    log::debug!("Mutating binary {}", path);
    let mut file = fs::File::open(path.clone())?;
    let session_folder = format!("{}/{}_{}_a{}_p{}", state.dbclient.as_ref().unwrap().f, 
        command.replace("/", "_"), 
        args.iter().map(|f| f.replace("/", "_")).collect::<Vec<_>>().join("_"), 
        attemps, 
        peek_count);
    fs::create_dir(session_folder.clone());
    log::debug!("Saving session in {}", session_folder.clone());


    // Filter first the header to check for Wasm
    let mut buf = [0; 4];
    let r = &file.read_exact(&mut buf)?;

    let mut bin = match &buf {
        // Filter first the header to check for Wasm
        b"\0asm" => {
            fs::read(path)?
        }
        _ => {
            return Err(CliError::Any("Invalid Wasm header".into()))
        }
    };
    
    let mut elapsed = 0;
    let mut hist = vec![];
    let mut gn = SmallRng::seed_from_u64(seed);

    let mut seen: HashSet<blake3::Hash> = HashSet::new();
    let mut collision_count = 0;
    
    'attempts: while elapsed < attemps {
        // mutated = m 
        let s = gn.gen();
        let mut config = WasmMutate::default();
        config.preserve_semantics(true);
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
            let (r, stdout, stderr) = check_binary(bin.clone(), command.clone(), args.clone());

            if r.success() {
                // continue since it is not interesting
            } else {               
                hist.push((s, elapsed, idx, stdout, stderr, newbin));
    
                if exit_on_found {
                    break 'attempts;
                }
            }
    
            elapsed += 1;

            if elapsed % 100 == 99 {
                log::debug!("Elapsed {}/{}. Collision count {}. Interesting count {}", elapsed, attemps, collision_count, hist.len());
            }
        }


    }
    
    log::debug!("Saving interesting {}", hist.len());

    for (s, elapsed, idx, stdout, stderr, newbin) in hist {
        let fname = format!("{session_folder}/e{:0width$}_s{}_i{}", elapsed,  s, idx, width=10);
        fs::create_dir(fname.clone());
        fs::write(format!("{}/stderr.txt", fname.clone()), &stderr)?;

        let mut f = fs::File::create(format!("{}/iteration_info.txt", fname.clone()))?;
        f.write_all(format!("seed: {}\n", seed).as_bytes())?;
        f.write_all(format!("attempts: {}\n", attemps).as_bytes())?;
        f.write_all(format!("elapsed: {}\n", elapsed).as_bytes())?;
        f.write_all(format!("idx: {}\n", idx).as_bytes())?;
        f.write_all(format!("variant_size: {}\n", newbin.len()).as_bytes())?;
        // TODO Add Meta info of the variant ?

        fs::write(format!("{}/stdout.txt", fname.clone()), &stdout)?;
        fs::write(format!("{}/variant.wasm", fname), &newbin)?;
    }
    // Now save the session to a folder ?
    Ok(())

}

fn check_binary(bin: Vec<u8>, command: String, args: Vec<String>) -> (ExitStatus, Vec<u8>, Vec<u8>) {
// Write file to tmparg
    std::fs::write("temp.wasm", &bin.clone()).unwrap();
    let output = std::process::Command::new(&command)
        .args(args.clone())
        .arg("temp.wasm")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .with_context(|| {
            format!(
                "Failed to run predicate script '{}'",
                command
            )
        }).unwrap();

    (output.status, output.stdout, output.stderr)
}

fn swap(a: &mut Vec<u8>, b:  Vec<u8>) {
    *a = b;
}


#[cfg(test)]
pub mod tests {
    use std::sync::{atomic::{AtomicU32, AtomicBool}, Arc};

    use env_logger::{Env, Builder};

    use crate::{db::DB, State};

    use super::mutate;



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
        ],600,false, 1, 0).unwrap()
    }
}