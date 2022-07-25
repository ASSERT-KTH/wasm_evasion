#![feature(internal_output_capture)]

use clap::{load_yaml, value_t, App};
use env_logger::{Builder, Env, Target};
use errors::{AResult, CliError};
use info::InfoExtractor;
use log::Record;
use sha2::{Digest, Sha256};
use std::{
    borrow::{Borrow, BorrowMut},
    fmt::Display,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    process,
    rc::Rc,
    str::FromStr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    thread::spawn,
    time::{self, Duration, SystemTime, UNIX_EPOCH},
};
use wasm_mutate::WasmMutate;

use crate::meta::Meta;
use mongodb::options::{ClientOptions, Credential, Predicate};
use mongodb::sync::Client;
use wasm_shrink::{IsInteresting, WasmShrink};

#[macro_use]
extern crate log;

mod errors;
pub mod extract;
pub mod info;
mod meta;
pub mod reduce;

use crate::extract::extract;
use crate::reduce::reduce;

use anyhow::Context;
use tempfile::NamedTempFile;

pub const NO_WORKERS: usize = 16;

pub trait Hasheable {
    fn hash256(self: &Self) -> Vec<u8>;
}

impl Hasheable for Vec<u8> {
    fn hash256(self: &Self) -> Vec<u8> {
        let mut encoder = Sha256::new();
        encoder.update(self);
        let hash_bytes = encoder.finalize();
        hash_bytes.to_vec()
    }
}

pub trait Printable {
    fn fmt1(self: &Self) -> String;
}

impl Printable for Vec<u8> {
    fn fmt1(self: &Self) -> String {
        self.iter()
            .map(|x| format!("{:02x}", x))
            .collect::<Vec<_>>()
            .join("")
    }
}

#[derive(Debug)]
pub struct State {
    dbclient: Option<Client>,
    collection_name: String,
    dbname: String,
    process: AtomicU32,
    error: AtomicU32,
    parsing_error: AtomicU32,
    out_folder: Option<String>,
}

macro_rules! arge {
    ($str: literal) => {
        CliError::Arg($str.to_string())
    };
}

macro_rules! arg_or_error {
    ($matches: ident,$arg: literal) => {
        $matches.value_of($arg).ok_or(arge!($arg))?.to_string()
    };
}

pub fn main() -> Result<(), errors::CliError> {
    let env = Env::default()
        //.filter_or("LOG_LEVEL", "trace")
        .filter("RUST_LOG")
        .write_style_or("LOG_STYLE", "never");

    Builder::from_env(env)
        .format(move |buf, record: &Record| {
            // Send to a diff file, depending on thread
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!(
                    "output{}.log",
                    std::thread::current().name().unwrap()
                ))
                .unwrap();
            let _ = file.write(
                &format!(
                    "[{}] [{}] <<<{}>>>\n",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis(),
                    record.module_path().unwrap_or(""),
                    record.args()
                )
                .into_bytes(),
            );
            Ok(())
        })
        .init();

    let yaml = load_yaml!("config.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let mut dbclientoptions = ClientOptions::parse(arg_or_error!(matches, "dbconn"))?;

    dbclientoptions.app_name = Some(arg_or_error!(matches, "dbname"));

    let mut credentials = Credential::default();
    credentials.password = Some(arg_or_error!(matches, "dbpass"));
    credentials.username = Some(arg_or_error!(matches, "dbuser"));

    dbclientoptions.credential = Some(credentials);

    let dbclient = Client::with_options(dbclientoptions)?;
    let mut state = State {
        dbclient: Some(dbclient.clone()),
        process: AtomicU32::new(0),
        error: AtomicU32::new(0),
        parsing_error: AtomicU32::new(0),
        collection_name: arg_or_error!(matches, "collection_name"),
        dbname: arg_or_error!(matches, "dbname"),
        out_folder: None,
    };

    match matches.subcommand() {
        ("extract", Some(args)) => {
            let reset = args.is_present("reset");
            if reset {
                println!("Reseting ");
                dbclient
                    .database(&arg_or_error!(matches, "dbname"))
                    .collection::<Meta>(&arg_or_error!(matches, "collection_name"))
                    .drop(None)?;
            }
            println!("Extracting...");
            extract(Arc::new(state), arg_or_error!(args, "folder"))?;
        }
        ("reduce", Some(args)) => {
            let reset = args.is_present("reset");
            if reset {
                println!("Reseting ");
                dbclient
                    .database(&arg_or_error!(matches, "dbname"))
                    .collection::<Meta>(&arg_or_error!(matches, "collection_name"))
                    .drop(None)?;
            }
            println!("Reducing...");
            state.out_folder = Some(arg_or_error!(args, "out"));
            reduce(Arc::new(state), arg_or_error!(args, "folder"))?;
        }
        ("export", Some(args)) => {
            println!("Exporting");
            let collection = dbclient
                .database(&arg_or_error!(matches, "dbname"))
                .collection::<Meta>(&arg_or_error!(matches, "collection_name"));

            let records = collection.find(None, None).unwrap();
            let mut outfile = std::fs::File::create(arg_or_error!(args, "out")).unwrap();

            if args.is_present("csv") {
                let mut writer = csv::Writer::from_writer(outfile);

                for record in records {
                    writer.serialize(record.unwrap()).unwrap();
                }

                writer.flush().unwrap()
            } else {
                let mut all = vec![];

                for record in records {
                    all.push(record.unwrap());
                }

                outfile
                    .write_all(serde_json::to_string_pretty(&all).unwrap().as_bytes())
                    .unwrap();
            }
        }
        ("clean", Some(_)) => {
            println!("Reseting ");
            dbclient
                .database(&arg_or_error!(matches, "dbname"))
                .collection::<Meta>(&arg_or_error!(matches, "collection_name"))
                .drop(None)?;
        }
        (c, _) => {
            todo!("Command {}", c);
        }
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use std::sync::atomic::AtomicU32;
    use std::sync::Arc;
    use std::time::Duration;

    use mongodb::options::{ClientOptions, Credential};
    use mongodb::sync::Client;

    use crate::meta::Meta;
    use crate::{extract, State};

    //#[test]
    pub fn test_extract() {
        let state = State {
            dbclient: None,
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            collection_name: "wasms".to_string(),
            dbname: "obfuscator".to_string(),
            out_folder: None,
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
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            collection_name: "wasms".to_string(),
            dbname: "obfuscator".to_string(),
            out_folder: None,
        };
        extract(Arc::new(state), "./".to_string()).unwrap();
    }

    pub fn test_db() {
        let mut dbclientoptions = ClientOptions::parse("mongodb://localhost:27017").unwrap();

        dbclientoptions.app_name = Some("obfuscator".to_string());
        dbclientoptions.connect_timeout = Some(Duration::from_millis(500));
        dbclientoptions.server_selection_timeout = Some(Duration::from_millis(500));

        let mut credentials = Credential::default();
        credentials.password = Some("admin".to_string());
        credentials.username = Some("admin".to_string());

        dbclientoptions.credential = Some(credentials);

        let client = Client::with_options(dbclientoptions).unwrap();

        // List the names of the databases in that deployment.
        for db_name in client.list_database_names(None, None).unwrap() {
            println!("{}", db_name);
        }
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
