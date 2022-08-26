#![allow(missing_docs)]
use wasmparser::Operator;

pub mod bridge;
pub mod parser;

/// Type helper to wrap operator and the byte offset in the code section of a Wasm module
pub type OperatorAndByteOffset<'a> = (Operator<'a>, usize);
