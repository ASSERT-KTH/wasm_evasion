#![feature(test)]

extern crate test;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Arc;
use std::time::Duration;

use analyzer::db::DB;
use analyzer::subcommands::extract::extract;
use analyzer::State;
use env_logger::{Builder, Env};
use test::{Bencher, black_box};

//#[path="../src/main.rs"]
//pub mod main;

#[bench]
pub fn bench_extract_many_2_100(b: &mut Bencher) {

    let env = Env::default()
        .filter_or("LOG_LEVEL", "analyzer=debug")
        .write_style_or("LOG_STYLE", "always");
    Builder::from_env(env).init();  

    b.iter(move || {
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
            sample_ratio: 100, // 1/100
            patch_metadata: false,
            seed: 0,
        };
        log::debug!("ratio 100");
        black_box(extract(Arc::new(state), "./tests/wasms".into()).unwrap());
    })
}


#[bench]
pub fn bench_extract_many_2_10(b: &mut Bencher) {

    let env = Env::default()
        .filter_or("LOG_LEVEL", "analyzer=debug")
        .write_style_or("LOG_STYLE", "always");
    Builder::from_env(env).init();  

    b.iter(move || {
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
            sample_ratio: 10, // 1/10
            patch_metadata: false,
            seed: 0,
        };
        log::debug!("ratio 10");
        black_box(extract(Arc::new(state), "./tests/wasms".into()).unwrap());
    })
}


#[bench]
pub fn bench_extract_many_2_1(b: &mut Bencher) {

    let env = Env::default()
        .filter_or("LOG_LEVEL", "bench,analyzer=debug")
        .write_style_or("LOG_STYLE", "always");
    Builder::from_env(env).init();  

    b.iter(move || {
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
            seed: 0,
            timeout: 10
        };
        log::debug!("ratio 1");
        black_box(extract(Arc::new(state), "./tests/wasms".into()).unwrap());
    })
}

//#[bench]
pub fn bench_extract_many_0(b: &mut Bencher) {
    let env = Env::default()
        .filter_or("LOG_LEVEL", "trace")
        .filter("RUST_LOG")
        .write_style_or("LOG_STYLE", "always");

    Builder::from_env(env).init();

    b.iter(move || {
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
            depth: 0,
            sample_ratio: 1,
            patch_metadata: false,
            seed: 0,
        };
        extract(Arc::new(state), "./tests/wasms".into()).unwrap();
    })
}



// #[bench]
pub fn bench_extract_many_3(b: &mut Bencher) {
    let env = Env::default()
        .filter_or("LOG_LEVEL", "trace")
        .filter("RUST_LOG")
        .write_style_or("LOG_STYLE", "always");

    Builder::from_env(env).init();

    b.iter(move || {
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
            depth: 3,
            sample_ratio: 1,
            patch_metadata: false,
            seed: 0,
        };
        extract(Arc::new(state), "./tests/wasms".into()).unwrap();
    })
}
