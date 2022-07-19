use wasmparser::BinaryReaderError;

#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error("IO error.")]
    Io(#[from] std::io::Error),

    #[error("Invalid argument '{0}'")]
    Arg(String),

    #[error("{0}")]
    Any(String),

    #[error("mongodb error")]
    Mongo(#[from] mongodb::error::Error),

    #[error("Wasmparser error")]
    Parser(#[from] BinaryReaderError),
}

pub type AResult<T> = Result<T, CliError>;
