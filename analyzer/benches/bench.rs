#![feature(test)]

extern crate test;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::fs;
use std::sync::atomic::{AtomicU32, AtomicBool};
use std::sync::Arc;
use std::time::Duration;

use env_logger::{Env, Builder};
use analyzer::State;
use analyzer::db::DB;
use analyzer::subcommands::extract::extract;
use test::{Bencher};

//#[path="../src/main.rs"]
//pub mod main;

#[bench]
pub fn bench_extract_many(b: &mut Bencher) {

    let env = Env::default()
    .filter_or("LOG_LEVEL", "trace")
    .filter("RUST_LOG")
    .write_style_or("LOG_STYLE", "always");

    Builder::from_env(env)
        .init();
        
    b.iter(move ||{
        // Remove the testdb
        fs::remove_dir_all("test_db");

        let mut state = State {
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
            seed: 0
        };
        extract(
            Arc::new(state),
            "./tests/wasms".into(),
        )
        .unwrap();
    })
    
}
