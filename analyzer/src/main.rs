#![feature(internal_output_capture)]

use clap::{load_yaml, App, value_t};
use db::DB;
use env_logger::{Builder, Env};
use errors::{CliError};


use sha2::{Digest, Sha256};
use subcommands::export;
use std::{
    io::{Write},
    sync::{
        atomic::{AtomicU32, AtomicBool},
        Arc,
    }, fs::OpenOptions, time::{SystemTime, UNIX_EPOCH}, cell::RefCell,
};

use std::collections::HashMap;
use crate::meta::Meta;

#[macro_use]
extern crate log;

mod errors;
pub mod info;
mod meta;
pub mod subcommands;
pub mod db;

use crate::subcommands::extract::extract;
use crate::subcommands::reduce::reduce;
use crate::subcommands::export::export;




pub const NO_WORKERS: usize = 12;

pub trait Hasheable {
    fn hash256(&self) -> Vec<u8>;
}

impl Hasheable for Vec<u8> {
    fn hash256(&self) -> Vec<u8> {
        let mut encoder = Sha256::new();
        encoder.update(self);
        let hash_bytes = encoder.finalize();
        hash_bytes.to_vec()
    }
}

pub trait Printable {
    fn fmt1(&self) -> String;
}

impl Printable for Vec<u8> {
    fn fmt1(&self) -> String {
        self.iter()
            .map(|x| format!("{:02x}", x))
            .collect::<Vec<_>>()
            .join("")
    }
}

#[derive(Debug)]
pub struct State {
    dbclient: Option<DB<'static>>,
    collection_name: String,
    mutation_cl_name: String,
    dbname: String,
    process: AtomicU32,
    error: AtomicU32,
    parsing_error: AtomicU32,
    out_folder: Option<String>,
    save_logs: bool,
    patch_metadata: bool,
    finish: AtomicBool,
    depth: u32,
    seed: u64,
    sample_ratio: u32
}
#[macro_export]
macro_rules! arge {
    ($str: literal) => {
        CliError::Arg($str.to_string())
    };
}

#[macro_export]
macro_rules! arg_or_error {
    ($matches: ident,$arg: literal) => {
        $matches.value_of($arg).ok_or(arge!($arg))?.to_string()
    };
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn main() -> Result<(), errors::CliError> {
    
    let env = Env::default()
    //.filter_or("LOG_LEVEL", "trace")
    .filter("RUST_LOG")
    .write_style_or("LOG_STYLE", "always");

    Builder::from_env(env)
        .init();

    let yaml = load_yaml!("config.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let dbconn = arg_or_error!(matches, "dbconn");
    let dbclient = DB::new(string_to_static_str(dbconn.clone()))?;
    let mut state = State {
        dbclient: Some(dbclient.clone()),
        mutation_cl_name: "muts".into(),
        process: AtomicU32::new(0),
        error: AtomicU32::new(0),
        parsing_error: AtomicU32::new(0),
        collection_name: "t".into(),
        dbname: "".into(),
        out_folder: None,
        save_logs: false,
        finish: AtomicBool::new(false),
        depth: 0,
        patch_metadata: false,
        sample_ratio: 1,
        seed: 0
    };

    match matches.subcommand() {
        ("extract", Some(args)) => {
            let reset = args.is_present("reset");
            if reset {
                log::debug!("Reseting ");
                std::fs::remove_dir_all(dbconn);
            }
            log::debug!("Extracting...");

            if args.is_present("mutation_cl_name") {
                state.mutation_cl_name = args.value_of("mutation_cl_name").unwrap().into();
            }

            if args.is_present("patch") {
                state.patch_metadata = true;
            }

            if args.is_present("depth") {
                state.depth = value_t!(args.value_of("depth"), u32).unwrap();
            }

            if args.is_present("seed") {
                state.seed = value_t!(args.value_of("seed"), u64).unwrap();
            }


            if args.is_present("sample") {
                state.sample_ratio = value_t!(args.value_of("sample"), u32).unwrap();
            }

            extract(Arc::new(state), arg_or_error!(args, "input"))?;
        }
        ("reduce", Some(args)) => {
            let reset = args.is_present("reset");
            if reset {
                log::debug!("Reseting ");
                std::fs::remove_dir_all(dbconn);
            }

            if args.is_present("save_logs") {
                let env = Env::default()
                //.filter_or("LOG_LEVEL", "trace")
                .filter("RUST_LOG")
                .write_style_or("LOG_STYLE", "never");

                Builder::from_env(env)
                    .format(move |buff, record| {
                        let name = std::thread::current();
                        let name = name.name().unwrap();
                        let logname = format!("output{}.log", name);
                        let mut outlog = OpenOptions::new().create(true).append(true).open(logname).unwrap();

                        outlog.write(format!("[{}] [{}] <<<{}>>>\n", 
                            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(), 
                            record.module_path().unwrap_or(""), 
                            record.args()).as_bytes());

                        Ok(())
                    })
                    .init();

                state.save_logs = true;
            } 

            log::debug!("Reducing...");
            state.out_folder = Some(arg_or_error!(args, "out"));
            reduce(Arc::new(state), arg_or_error!(args, "input"))?;
        }
        ("export", Some(args)) => {
            export(&matches, args, dbclient)?;
        }
        ("clean", Some(_)) => {
            log::debug!("Reseting ");
            std::fs::remove_dir_all(dbconn);
        }
        (c, _) => {
            todo!("Command {}", c);
        }
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use std::cell::RefCell;
    use std::sync::atomic::{AtomicU32, AtomicBool};
    use std::sync::Arc;
    use std::time::Duration;

    use crate::meta::Meta;
    use crate::{extract, State};

    //#[test]
    pub fn test_extract() {
        let state = State {
            dbclient: None,
            mutation_cl_name: "muts".to_string(),
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            collection_name: "wasms".to_string(),
            dbname: "obfuscator".to_string(),
            out_folder: None,
            save_logs: false,
            finish: AtomicBool::new(false),
            depth: 0,
            sample_ratio: 1,
            patch_metadata: false,
            seed: 0
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
            mutation_cl_name: "muts".to_string(),
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            collection_name: "wasms".to_string(),
            dbname: "obfuscator".to_string(),
            out_folder: None,
            save_logs: false,
            patch_metadata: false,
            finish: AtomicBool::new(false),
            depth: 0,
            sample_ratio: 1,
            seed: 0
        };
        extract(Arc::new(state), "./".to_string()).unwrap();
    }

    #[test]
    pub fn test_csv() {
        let mut writer = csv::Writer::from_writer(std::io::stdout());
        let m = Meta::new();

        // writer.write_record(&["a"]).unwrap();

        writer.serialize(m).unwrap();
        writer.flush().unwrap();
    }
}
