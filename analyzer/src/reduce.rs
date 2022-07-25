use std::{
    fmt::Display,
    fs,
    io::Read,
    path::PathBuf,
    str::FromStr,
    sync::{atomic::Ordering, Arc},
    time,
};

use anyhow::Context;
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
            // Filter first the header to check for Wasm
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

                let output =
                    PathBuf::from_str(&format!("{}/{}.shrunken.wasm", outfolder, name)).unwrap();
                let logs =
                    PathBuf::from_str(&format!("{}/{}.shrunken.logs", outfolder, name)).unwrap();

                // copy the original in the folder to get it as the new shrunked binary

                std::fs::write(output.clone(), bindata.clone()).unwrap();

                //let predicate = PathBuf::from_str(&format!("{}/{}.shrunken.predicate", outfolder, name)).unwrap();
                //let wat_path = PathBuf::from_str(&format!("{}/{}.shrunken.wat", outfolder, name)).unwrap();

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
                    }
                    Ok(_i) => {
                        let name = std::thread::current();
                        let name = name.name().unwrap();
                        let name = format!("output{}.log", name);
                        let content = std::fs::read(name).unwrap();

                        String::from_utf8(content).unwrap()
                    }
                };

                let mut meta = meta::Meta::new();
                meta.id = output.display().to_string();

                let bindata = loop {
                    let bindata = fs::read(output.clone());

                    match bindata {
                        Err(e) => {
                            println!("{}", e);
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
                meta.description = format!(
                    "seed: {}, fuel: {}, ratio {}",
                    0,
                    1000,
                    bindata.len() as f32 / initial_size as f32
                );
                meta.logs = logs;

                let db = dbclient.database(&dbname);
                let collection = db.collection::<Meta>(&collection_name);

                let docs = vec![meta];

                match collection.insert_many(docs, None) {
                    Ok(_) => {}
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
