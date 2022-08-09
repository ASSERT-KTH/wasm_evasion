

use db::DB;
use sha2::Digest;
use sha2::Sha256;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32};

pub mod db;
pub mod errors;
pub mod info;
pub mod meta;
pub mod subcommands;
pub mod logger;

pub const NO_WORKERS: usize = 8;
pub static SOCKET_PATH: &'static str = "probes.sock";

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
    pub process: AtomicU32,
    pub error: AtomicU32,
    pub parsing_error: AtomicU32,
    pub out_folder: Option<String>,
    pub save_logs: bool,
    pub patch_metadata: bool,
    pub finish: AtomicBool,
    pub depth: u32,
    pub seed: u64,
    pub timeout: u32,
    pub sample_ratio: u32,
    pub snapshot: Option<String>,
    pub snapshot_time: Option<u32>
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

pub fn send_signal_to_probes_socket(signal: String) {
    let socket = Path::new(SOCKET_PATH);

    // Connect to socket
    let mut stream = match UnixStream::connect(&socket) {
        Err(_) => panic!("server is not running"),
        Ok(stream) => stream,
    };

    // Send message
    match stream.write(&signal.as_bytes()) {
        Err(_) => panic!("couldn't send message"),
        Ok(_) => {}
    }
}