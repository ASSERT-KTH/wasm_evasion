
use anyhow::Context;
use clap::Parser;
use egg::{Runner, EGraph, Id, Language};
use souperdiversifier::bridge::Superdiversifier;
use wasm_mutate::module::{PrimitiveTypeInfo, TypeInfo};
use wasm_mutate::mutators::peephole::dfg::DFGBuilder;
use wasm_mutate::mutators::peephole::eggsy::analysis::PeepholeMutationAnalysis;
use wasm_mutate::mutators::peephole::eggsy::lang::Lang;
use wasmparser::{CodeSectionReader, Operator, LocalsReader};
use std::ffi::CStr;
use std::path::{Path, PathBuf};
use std::{panic, process};
use std::fs;
use std::io::Read;
use wasm_mutate::info::ModuleInfo;
use wasm_mutate::WasmMutate;


/// # TODO
///
#[derive(Parser)]
struct Options {
    /// The input folder that contains the Wasm binaries.
    input: PathBuf,
    /// The seed of the random mutation, 0 by default
    #[clap(short = 's', long = "seed")]
    seed: u64,
}

fn main() -> anyhow::Result<()> {
    // Init logs
    env_logger::init();

    let opts = Options::parse();
    
    let mut input = Box::new(
        fs::File::open(opts.input)
            .with_context(|| format!("failed to open input"))?,
    );
    
    // Parse the binary
    let mut input_wasm = vec![];
    input
        .read_to_end(&mut input_wasm)
        .with_context(|| format!("failed to read input"))?;
    
    // Get the wasm-mutate AST
    let info = ModuleInfo::new(&input_wasm)?;
    log::debug!("Correct parsing of Wasm binary. ");

    // Iterate through the functions and each DFG
    // TODO, returns the new module here
    let sdiversifier = Superdiversifier::new();
    sdiversifier.souperdiversify_peepholes(info)?;
    // TODO, validate module

    // The previous stage makes the peepholes pass
    // Using the AST we can infer constants for the if-else constructions :)
    // TODO

    Ok(())
}



#[cfg(test)]
mod tests {
    use std::ffi::{CStr};

    use env_logger::{Env, Builder};
    use libc::c_char;
    use souperdiversifier::{bridge::{Superdiversifier, superoptimize}, parser::souper2Lang};
    use wasm_mutate::{info::ModuleInfo, WasmMutate};
    use wasmparser::validate;
    use std::ffi::CString;

    #[test]
    fn test_to_superoptimize() {

        
        let wat =  r#"
        (module
            (func (export "exported_func") (result i32)
                i32.const 42
                i32.const 42
                i32.const 42
                i32.add
                i32.add
                i32.const 42
                i32.const 42
                i32.add
                i32.add
            )
        )
        "#;

        let sdiversifier = Superdiversifier::new();
        let expected = &wat::parse_str(wat).unwrap();

        
        // Get the wasm-mutate AST
        //let info = ModuleInfo::new(&expected).unwrap();
        let mut config = WasmMutate::default();
        config.setup(&expected).unwrap();
        println!("Correct parsing of Wasm binary. ");

        // Iterate through the functions and each DFG
        // TODO, returns the new module here
        let info = config.info();
        let replacements = sdiversifier.souperdiversify_peepholes(info.clone()).unwrap();
        let module = sdiversifier.superoptimize(&mut config, replacements.clone()).unwrap();


        println!("Module ");
        let mutated_bytes = &module.finish();
        let text = wasmprinter::print_bytes(mutated_bytes).unwrap();
        validate( mutated_bytes).unwrap();


        println!("{}", text);
        //println!("{:?}", replacements)
    }



    #[test]
    fn test_to_superoptimize2() {
        let wat =  r#"
        (module
            (func (export "exported_func") (result i32)
                i32.const 42
                i32.const 42
                i32.const 1
                i32.const 0
                i32.add
                select
            )
        )
        "#;

        let sdiversifier = Superdiversifier::new();
        let expected = &wat::parse_str(wat).unwrap();

        
        // Get the wasm-mutate AST
        //let info = ModuleInfo::new(&expected).unwrap();
        let mut config = WasmMutate::default();
        config.setup(&expected).unwrap();
        println!("Correct parsing of Wasm binary. ");

        // Iterate through the functions and each DFG
        // TODO, returns the new module here
        let info = config.info();
        let replacements = sdiversifier.souperdiversify_peepholes(info.clone()).unwrap();
        let module = sdiversifier.superoptimize(&mut config, replacements.clone()).unwrap();


        println!("Module ");
        let mutated_bytes = &module.finish();
        let text = wasmprinter::print_bytes(mutated_bytes).unwrap();
        validate( mutated_bytes).unwrap();


        println!("{}", text);
        //println!("{:?}", replacements)
    }



    #[test]
    fn test_to_superoptimize3() {
        let wat =  r#"
        (module
            (func (export "exported_func") (param i32 i32 i32) (result i32)
                local.get 0
                local.get 0
                local.get 0
                i32.add
                i32.add
                local.get 0
                local.get 2
                select
            )
        )
        "#;

        let sdiversifier = Superdiversifier::new();
        let expected = &wat::parse_str(wat).unwrap();

        
        // Get the wasm-mutate AST
        //let info = ModuleInfo::new(&expected).unwrap();
        let mut config = WasmMutate::default();
        config.setup(&expected).unwrap();
        println!("Correct parsing of Wasm binary. ");

        // Iterate through the functions and each DFG
        // TODO, returns the new module here
        let info = config.info();
        let replacements = sdiversifier.souperdiversify_peepholes(info.clone()).unwrap();
        let module = sdiversifier.superoptimize(&mut config, replacements.clone()).unwrap();


        println!("Module ");
        let mutated_bytes = &module.finish();
        let text = wasmprinter::print_bytes(mutated_bytes).unwrap();
        validate( mutated_bytes).unwrap();


        println!("{}", text);
        //println!("{:?}", replacements)
    }


    #[test]
    fn test_to_superoptimize4() {
        let wat =  r#"
        (module
            (func (export "exported_func") (param i32 i32 i32) (result i32)
                local.get 0
                local.get 0
                local.get 0
                local.get 0
                i32.add
                i32.add
                i32.add
            )
        )
        "#;

        let sdiversifier = Superdiversifier::new();
        let expected = &wat::parse_str(wat).unwrap();

        
        // Get the wasm-mutate AST
        //let info = ModuleInfo::new(&expected).unwrap();
        let mut config = WasmMutate::default();
        config.setup(&expected).unwrap();
        println!("Correct parsing of Wasm binary. ");

        // Iterate through the functions and each DFG
        // TODO, returns the new module here
        let info = config.info();
        let replacements = sdiversifier.souperdiversify_peepholes(info.clone()).unwrap();
        let module = sdiversifier.superoptimize(&mut config, replacements.clone()).unwrap();


        println!("Module ");
        let mutated_bytes = &module.finish();
        let text = wasmprinter::print_bytes(mutated_bytes).unwrap();
        validate( mutated_bytes).unwrap();


        println!("{}", text);
        //println!("{:?}", replacements)
    }



    extern "C" fn callback(lhs_ptr: *const i8 /* Original Query */, rhs_ptr: *const i8, cost: i32) -> i32 {
        let rhs = unsafe { CStr::from_ptr(rhs_ptr) };
        println!("RHS {:?}\n Cost: {}", rhs, cost);
        let lang = unsafe { CStr::from_ptr(lhs_ptr) };
        println!("LHS {:?}", lang);

        0
    }


    #[test]
    fn test_to_superoptimize_direct_string() {
        let souperIR = CString::new("
        %0:i32 = var; local
        %1:i32 = add %0,%0
        %2:i32 = add %0,%1
        %3:i32 = add %0,%2
        infer %3").unwrap();

        let startptr = CString::new("(i32.add local.get.0 (i32.add local.get.0 (i32.add local.get.0 local.get.0)))").unwrap();

        unsafe {
            superoptimize( souperIR.as_ptr(), startptr.as_ptr(), callback);
        }
        std::mem::forget(souperIR);
        std::mem::forget(startptr);
        // FIXME, release the memory
        //println!("{:?}", replacements)
    }
}