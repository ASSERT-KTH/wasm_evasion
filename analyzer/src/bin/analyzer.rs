#![feature(internal_output_capture)]

use analyzer::db::DB;
use analyzer::errors::CliError;
use analyzer::{arg_or_error, arge, State};
use clap::{load_yaml, value_t, App};
use env_logger::{Builder, Env};

use std::{
    cell::RefCell,
    fs::OpenOptions,
    io::Write,
    sync::{
        atomic::{AtomicBool, AtomicU32},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use analyzer::meta::Meta;
use std::collections::HashMap;

#[macro_use]
extern crate log;

use analyzer::subcommands::export::export;
use analyzer::subcommands::extract::extract;
use analyzer::subcommands::reduce::reduce;

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn main() -> Result<(), analyzer::errors::CliError> {
    let env = Env::default()
        //.filter_or("LOG_LEVEL", "trace")
        .filter("RUST_LOG")
        .write_style_or("LOG_STYLE", "always");

    Builder::from_env(env).init();

    let yaml = load_yaml!("config.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let dbconn = arg_or_error!(matches, "dbconn");
    let dbclient = DB::new(string_to_static_str(dbconn.clone()))?;
    let mut state = State {
        dbclient: Some(dbclient.clone()),
        process: AtomicU32::new(0),
        error: AtomicU32::new(0),
        parsing_error: AtomicU32::new(0),
        out_folder: None,
        save_logs: false,
        finish: AtomicBool::new(false),
        depth: 0,
        patch_metadata: false,
        sample_ratio: 1,
        seed: 0,
        timeout: 0
    };

    match matches.subcommand() {
        ("extract", Some(args)) => {
            let reset = args.is_present("reset");
            if reset {
                log::debug!("Reseting ");
                std::fs::remove_dir_all(dbconn.clone());
            }
            log::debug!("Extracting...");

            if args.is_present("depth") {
                state.depth = value_t!(args.value_of("depth"), u32).unwrap();
            }

            if args.is_present("seed") {
                state.seed = value_t!(args.value_of("seed"), u64).unwrap();
            }

            if args.is_present("sample") {
                state.sample_ratio = value_t!(args.value_of("sample"), u32).unwrap();
            }

            if args.is_present("timeout") {
                state.timeout = value_t!(args.value_of("timeout"), u32).unwrap();
            }

            log::debug!("Extracting to {}", dbconn.clone());
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
                        let mut outlog = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(logname)
                            .unwrap();

                        outlog.write(
                            format!(
                                "[{}] [{}] <<<{}>>>\n",
                                SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis(),
                                record.module_path().unwrap_or(""),
                                record.args()
                            )
                            .as_bytes(),
                        );

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
            export(&matches, args, dbclient, Arc::new(state))?;
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
    use std::borrow::Borrow;
    use std::cell::RefCell;
    use std::fs;
    use std::sync::atomic::{AtomicBool, AtomicU32};
    use std::sync::Arc;
    use std::time::Duration;

    use analyzer::State;
    use env_logger::{Builder, Env};

    use analyzer::db::DB;
    use analyzer::meta::Meta;
    use analyzer::subcommands::extract;
    use analyzer::subcommands::extract::extract;

    #[test]
    pub fn test_extract() {

        let env = Env::default()
            .filter_or("LOG_LEVEL", "bench,analyzer=debug")
            .write_style_or("LOG_STYLE", "always");
        Builder::from_env(env).init();  

        let state = State {
            dbclient: Some(DB::new("test_db").unwrap()),
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
            timeout: 10
        };
        extract(Arc::new(state), "./tests".to_string()).unwrap();
    }


    #[test]
    pub fn test_extract_depth() {

        let env = Env::default()
            .filter_or("LOG_LEVEL", "bench,analyzer=debug")
            .write_style_or("LOG_STYLE", "always");
        Builder::from_env(env).init();  

        let state = State {
            dbclient: Some(DB::new("test_db").unwrap()),
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
            timeout: 60, // 2 minutes per binary
        };
        extract(Arc::new(state), "./tests/wasms".to_string()).unwrap();
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
