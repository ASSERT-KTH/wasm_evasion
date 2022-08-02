use std::string::FromUtf8Error;

use wasmparser::BinaryReaderError;

#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error("IO error.")]
    Io(#[from] std::io::Error),

    #[error("Invalid argument '{0}'")]
    Arg(String),

    #[error("{0}")]
    Any(String),

    #[error("key not found {0}")]
    KeyNotFound(String),

    #[error("UTF Error {0}")]
    UTF8Error(#[from] FromUtf8Error),

    #[error("Not valid database ")]
    NotValidDB(),

    #[error("serde error")]
    Serde(#[from] serde_json::error::Error),

    #[error("Wasmparser error")]
    Parser(#[from] BinaryReaderError),

    #[error("Sled error")]
    Sled(#[from] sled::Error),
}

pub type AResult<T> = Result<T, CliError>;
