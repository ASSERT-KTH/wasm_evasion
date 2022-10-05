#![allow(missing_docs)]
#![feature(associated_const_equality)]
#[macro_use]
extern crate lazy_static;
use wasmparser::Operator;

pub mod error;
pub mod bridge;
pub mod parser;

/// Type helper to wrap operator and the byte offset in the code section of a Wasm module
pub type OperatorAndByteOffset<'a> = (Operator<'a>, usize);



#[cfg(test)]
pub(crate) fn validate(validator: &mut wasmparser::Validator, bytes: &[u8]) {
    let err = match validator.validate_all(bytes) {
        Ok(_) => return,
        Err(e) => e,
    };
    drop(std::fs::write("test.wasm", &bytes));
    if let Ok(text) = wasmprinter::print_bytes(bytes) {
        drop(std::fs::write("test.wat", &text));
    }

    panic!("wasm failed to validate {:?}", err);
}
