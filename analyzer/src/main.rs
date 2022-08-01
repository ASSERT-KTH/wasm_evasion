#![feature(internal_output_capture)]

use clap::{load_yaml, App, value_t};
use env_logger::{Builder, Env};
use errors::{CliError};


use sha2::{Digest, Sha256};
use std::{
    io::{Write},
    sync::{
        atomic::{AtomicU32, AtomicBool},
        Arc,
    }, fs::OpenOptions, time::{SystemTime, UNIX_EPOCH}, cell::RefCell,
};


use crate::meta::Meta;
use mongodb::{options::{ClientOptions, Credential}, bson::Bson};
use mongodb::sync::Client;


#[macro_use]
extern crate log;

mod errors;
pub mod info;
mod meta;
pub mod subcommands;

use crate::subcommands::extract::extract;
use crate::subcommands::reduce::reduce;




pub const NO_WORKERS: usize = 10;

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
    dbclient: Option<Client>,
    collection_name: String,
    mutation_cl_name: String,
    dbname: String,
    process: AtomicU32,
    error: AtomicU32,
    parsing_error: AtomicU32,
    out_folder: Option<String>,
    save_logs: bool,
    finish: AtomicBool,
    depth: u32,
    seed: u64,
    sample_ratio: u32
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
    .write_style_or("LOG_STYLE", "always");

    Builder::from_env(env)
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
        mutation_cl_name: "muts".into(),
        process: AtomicU32::new(0),
        error: AtomicU32::new(0),
        parsing_error: AtomicU32::new(0),
        collection_name: arg_or_error!(matches, "collection_name"),
        dbname: arg_or_error!(matches, "dbname"),
        out_folder: None,
        save_logs: false,
        finish: AtomicBool::new(false),
        depth: 0,
        sample_ratio: 1,
        seed: 0
    };

    match matches.subcommand() {
        ("extract", Some(args)) => {
            let reset = args.is_present("reset");
            if reset {
                log::debug!("Reseting ");
                dbclient
                    .database(&arg_or_error!(matches, "dbname"))
                    .collection::<Meta>(&arg_or_error!(matches, "collection_name"))
                    .drop(None)?;
            }
            log::debug!("Extracting...");

            if args.is_present("mutation_cl_name") {
                state.mutation_cl_name = args.value_of("mutation_cl_name").unwrap().into();
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

            extract(RefCell::new(state), arg_or_error!(args, "input"))?;
        }
        ("reduce", Some(args)) => {
            let reset = args.is_present("reset");
            if reset {
                log::debug!("Reseting ");
                dbclient
                    .database(&arg_or_error!(matches, "dbname"))
                    .collection::<Meta>(&arg_or_error!(matches, "collection_name"))
                    .drop(None)?;
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
            reduce(RefCell::new(state), arg_or_error!(args, "input"))?;
        }
        ("export", Some(args)) => {
            

            if args.is_present("list") {
                let collection = dbclient
                    .database(&arg_or_error!(matches, "dbname"));

                println!("Collections");

               for l in  collection.list_collection_names(None).unwrap() {
                    println!("\t{}", l);
               }

            } else {
                log::debug!("Exporting");
                let collection = dbclient
                    .database(&arg_or_error!(matches, "dbname"))
                    .collection::<Bson>(&arg_or_error!(matches, "collection_name"));

                let records = collection.find(None, None).unwrap();
                let mut outfile = std::fs::File::create(arg_or_error!(args, "out")).unwrap();

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
            log::debug!("Reseting ");
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
    use std::cell::RefCell;
    use std::sync::atomic::{AtomicU32, AtomicBool};
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
            seed: 0
        };
        extract(
            RefCell::new(state),
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
            finish: AtomicBool::new(false),
            depth: 0,
            sample_ratio: 1,
            seed: 0
        };
        extract(RefCell::new(state), "./".to_string()).unwrap();
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
