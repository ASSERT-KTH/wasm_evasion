use std::sync::atomic::{AtomicBool, AtomicU32};
use sha2::Digest;
use db::DB;
use sha2::Sha256;


pub mod errors;
pub mod info;
pub mod meta;
pub mod subcommands;
pub mod db;



pub const NO_WORKERS: usize = 8;

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
    pub dbclient: Option<DB<'static>>,
    pub collection_name: String,
    pub mutation_cl_name: String,
    pub dbname: String,
    pub process: AtomicU32,
    pub error: AtomicU32,
    pub parsing_error: AtomicU32,
    pub out_folder: Option<String>,
    pub save_logs: bool,
    pub patch_metadata: bool,
    pub finish: AtomicBool,
    pub depth: u32,
    pub seed: u64,
    pub sample_ratio: u32
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
