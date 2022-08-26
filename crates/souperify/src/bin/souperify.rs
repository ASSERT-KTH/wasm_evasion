use anyhow::Context;
use clap::Parser;
use egg::{Runner, EGraph, Id, Language};
use souperify::OperatorAndByteOffset;
use souperify::bridge::{hello, superoptimize};
use souperify::parser::souper2Lang;
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
    souperify_peepholes(info)?;
    // TODO, validate module

    // The previous stage makes the peepholes pass
    // Using the AST we can infer constants for the if-else constructions :)
    // TODO

    Ok(())
}


// Collect and unfold params and locals, [x, ty, y, ty2] -> [ty....ty, ty2...ty2]
fn get_func_locals(
    info: &ModuleInfo,
    funcidx: u32,
    localsreader: &mut LocalsReader,
) -> anyhow::Result<Vec<PrimitiveTypeInfo>> {
    let ftype = info.get_functype_idx(funcidx);
    match ftype {
        TypeInfo::Func(tpe) => {
            let mut all_locals = Vec::new();

            for primitive in &tpe.params {
                all_locals.push(primitive.clone())
            }
            for _ in 0..localsreader.get_count() {
                let (count, ty) = localsreader.read()?;
                let tymapped = PrimitiveTypeInfo::from(ty);
                for _ in 0..count {
                    all_locals.push(tymapped.clone());
                }
            }

            Ok(all_locals)
        }
    }
}

fn generate_souper_query(root: Id, egraph: EGraph<Lang, PeepholeMutationAnalysis>) -> anyhow::Result<String> {
    // Iterate through Lang tree and generate the Souper IR 
    enum Event {
        Enter,
        Exit
    }

    let mut result = String::new();
    let mut worklist = vec![
        (root, Event::Exit),
        (root, Event::Enter)
        ];
    let mut varidx = 1;
    let mut stack = vec![];
    while let Some((current, event)) = worklist.pop() {
        let l = &egraph[current].nodes[0];

        match event {
            Event::Enter => {
                for ch in l.children() {
                    worklist.push((*ch, Event::Exit));
                    worklist.push((*ch, Event::Enter));
                } 
            },
            Event::Exit => {
                println!(";%{}", l);
                let rpte = egraph.analysis.get_returning_tpe(l, &egraph);
                let souper_width = match rpte {
                    Ok(tpe) => {
                        match tpe {
                            PrimitiveTypeInfo::I32 => "i32",
                            PrimitiveTypeInfo::I64 => "i64",
                            _ => anyhow::bail!("Invalid type  {:?}", tpe)
                        }
                    }
                    Err(e) => anyhow::bail!("Invalid type  {:?}", e)
                };
                let mut operators = vec![];
                for _ in l.children() {
                    let op = stack.pop();
                    match op {
                        None => {
                            anyhow::bail!("Invalid stack state for {}", l);
                        }
                        Some(op) => {
                            operators.push(op);
                        }
                    }
                }
                match l {
                    Lang::I32Add(_) | Lang::I64Add(_) => {
                        result.push_str(&format!("%{}:{} = add ", varidx, souper_width));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    },
                    Lang::LocalGet(_) => {
                        // Swap operands
                        result.push_str(&format!("%{}:{} = var", varidx, souper_width));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    },
                    
                    Lang::GlobalGet(_) => {
                        // Swap operands
                        result.push_str(&format!("%{} = var", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    },
                    Lang::I32GeS(_) | Lang::I64GeS(_) => {

                        let o = operators.pop().unwrap();
                        let o2 = operators.pop().unwrap();
                        operators.push(o);
                        operators.push(o2);
                        
                        result.push_str(&format!("%{} = slt ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;

                    }
                    Lang::I32GeU(_) | Lang::I64GeU(_) => {

                        let o = operators.pop().unwrap();
                        let o2 = operators.pop().unwrap();
                        operators.push(o);
                        operators.push(o2);
                        
                        result.push_str(&format!("%{} = ule ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;

                    }
                    Lang::I32GtU(_) | Lang::I64GtU(_) => {

                        let o = operators.pop().unwrap();
                        let o2 = operators.pop().unwrap();
                        operators.push(o);
                        operators.push(o2);
                        
                        result.push_str(&format!("%{} = ult ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;

                    }
                    Lang::I32GtS(_) | Lang::I64GtS(_) => {

                        let o = operators.pop().unwrap();
                        let o2 = operators.pop().unwrap();
                        operators.push(o);
                        operators.push(o2);
                        
                        result.push_str(&format!("%{} = sle ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;

                    }
                    Lang::I32And(_) | Lang::I64And(_) => {
                        result.push_str(&format!("%{} = and ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::I32Or(_) | Lang::I64Or(_) => {
                        result.push_str(&format!("%{} = or ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::I32Xor(_) | Lang::I64Xor(_) => {
                        result.push_str(&format!("%{} = xor ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::I32Sub(_) | Lang::I64Sub(_) => {
                        result.push_str(&format!("%{} = sub ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::I32Mul(_) | Lang::I64Mul(_) => {
                        result.push_str(&format!("%{} = sub ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::I32DivU(_) | Lang::I64DivU(_) => {
                        result.push_str(&format!("%{} = udiv ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::I32DivS(_) | Lang::I64DivS(_) => {
                        result.push_str(&format!("%{} = sdiv ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::I32RemU(_) | Lang::I64RemU(_) => {
                        result.push_str(&format!("%{} = urem ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::I32Shl(_) | Lang::I64Shl(_) => {
                        result.push_str(&format!("%{} = shl ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::I32ShrU(_) | Lang::I64ShrU(_) => {
                        result.push_str(&format!("%{} = lshr ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::I32ShrS(_) | Lang::I64ShrS(_) => {
                        result.push_str(&format!("%{} = ashr ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::I32RemS(_) | Lang::I64RemS(_) => {
                        result.push_str(&format!("%{} = srem ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::Select(_) => {
                        result.push_str(&format!("%{} = select ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::I32Eq(_) | Lang::I64Eq(_) => {
                        result.push_str(&format!("%{} = eq ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::I32Ne(_) | Lang::I64Ne(_) => {
                        result.push_str(&format!("%{} = ne ", varidx));
                        stack.push(format!("%{}", varidx));
                        varidx += 1;
                    }
                    Lang::I32(v) => {
                        stack.push(format!("{}:i32", v));
                    },
                    Lang::I64(v) => {
                        stack.push(format!("{}:i64", v));
                    },
                    _ => {
                        anyhow::bail!("Invalid operator {}", l)
                    }
                }
                let mut i = 0;
                let C = operators.len();
                for op in operators {
                    result.push_str(&format!("{}", op));
                    if i < C - 1 {
                        result.push_str(",");
                    }
                    i += 1;
                }
                result.push_str(&
                    format!("\n"));
            },
        }
    }
    // set to infer last operation
    result.push_str(&format!("infer %{}", varidx - 1 ));
    Ok(result)
}

extern "C" fn callback(rhs_ptr: *const i8, cost: i32) -> i32 {
    let rhs = unsafe { CStr::from_ptr(rhs_ptr) };
    println!("RHS {:?}\n Cost: {}", rhs, cost);
    let lang = souper2Lang(rhs.to_str().unwrap()).unwrap(); // Save this into a global State ?
    
    println!("lang parsing {:?}", lang.as_ref());
    0
}

fn souperify_peepholes(info: ModuleInfo)  -> anyhow::Result<()>{

    let code_section = info.get_code_section();
    let mut sectionreader = CodeSectionReader::new(code_section.data, 0)?;
    let function_count = sectionreader.get_count();

    log::debug!("Function count {}. ", function_count);
    let config = WasmMutate::default();

    for i in 0..function_count {
        // TODO create a new function in the new module
        
        log::debug!("Visiting function {}", i);
        let reader = sectionreader.read().unwrap();

        // Get operators
        let operatorreader = reader.get_operators_reader()?;
        let mut localsreader = reader.get_locals_reader()?;
        let operators = operatorreader
            .into_iter_with_offsets()
            .collect::<wasmparser::Result<Vec<OperatorAndByteOffset>>>()?;
        let operatorscount = operators.len();

        log::debug!("\tOperators count {}", operatorscount);

        let locals = get_func_locals(
            &info,
            i + info.num_imported_functions(), /* the function type is shifted
                                                                        by the imported functions*/
            &mut localsreader,
        )?;

        // Visit operators from back to front, for better DFG construction

        for opidx in 0.. operatorscount {

            let mut dfg = DFGBuilder::new(&config);
            let reverseidx = operatorscount - 1 - opidx;
            let basicblock = dfg.get_bb_from_operator(reverseidx, &operators);


            let basicblock = match basicblock {
                None => {
                    log::trace!(
                        "Basic block cannot be constructed for opcode {:?}",
                        &operators[reverseidx]
                    );
                    continue;
                }
                Some(basicblock) => basicblock,
            };

            let minidfg = dfg.get_dfg(&info, &operators, &basicblock);

            let minidfg = match minidfg {
                None => {
                    log::trace!("DFG cannot be constructed for opcode {}", reverseidx);
                    continue;
                }
                Some(minidfg) => minidfg,
            };
            

            if !minidfg.map.contains_key(&reverseidx) {
                continue;
            }

            if !minidfg.is_subtree_consistent_from_root() {
                continue;
            };

            let start = minidfg.get_expr(reverseidx);


            let analysis = PeepholeMutationAnalysis::new(&info, locals.clone());
            let runner = Runner::<Lang, PeepholeMutationAnalysis, ()>::new(analysis.clone())
                .with_expr(&start);
                // Since no rule is provided the created egraph is lane

            let mut egraph = runner.egraph;
            // In theory this will return the Id of the operator eterm
            let root = egraph.add_expr(&start);

            let rtpe = analysis.get_returning_tpe(&egraph[root].nodes[0], &egraph)?;

            // Check if the operator returns an integer value
            // if so, call souper for a replacement
            match rtpe {
                PrimitiveTypeInfo::I32 | PrimitiveTypeInfo::I64 => {
                    log::info!("Creating souper IR for subtree");
                    log::debug!("E-expre {}", &start);
                    match generate_souper_query(root, egraph) {
                        Ok(souperIR) => {
                            // Call Souper here
                            println!("{}",souperIR);
                            unsafe {
                                superoptimize(souperIR.as_ptr() as *const i8, callback);
                            }

                            // TODO, Collect the RHS in the callbacks
                            // Select best RHS
                            // Get the LANG expression
                            // Replace and set the new function
                        }
                        Err(e) => {
                            println!("{}", e);
                            continue;
                        }
                    }

                    
                },
                _ => {
                    log::info!("Invalid return type, not integer");
                    // TODO add as it is to the new function, and jump to the
                    // next one
                    continue;
                }
            }
        }
        return Ok(())
    }

    Ok(())
}




#[cfg(test)]
mod tests {
    use std::ffi::{CStr};

    use libc::c_char;
    use souperify::{bridge::superoptimize, parser::souper2Lang};
    use wasm_mutate::info::ModuleInfo;

    use crate::souperify_peepholes;

    extern "C" fn callback(rhs_ptr: *const c_char, cost: i32) -> i32 {
        let rhs = unsafe { CStr::from_ptr(rhs_ptr) };
        println!("RHS {:?}\n Cost: {}", rhs, cost);
        souper2Lang(rhs.to_str().unwrap());
        0
    }

    //#[test]
    fn test_call_souper() {
        let lhs = r#"
        ;%i32.const.8192
        ;%local.get.2
        ;%i32.ge_s

        %1:i32 = var;
        %2:i1 = slt 8192:i32, %1;
        %3:i1 = eq %2, 1:i1 ;
        %4:i1 = eq %3, 1:i1 ;
        %5:i1 = eq %4, 1:i1 ;
        %6:i1 = eq %5, 1:i1 ;
        %7:i1 = eq %6, 1:i1 ;
        infer %7;
;        "#; 

        let t = unsafe {
            superoptimize(lhs.as_ptr() as *const i8, callback)
        };

        println!("{}", t);
    }


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
            )
        )
        "#;

        let expected = &wat::parse_str(wat).unwrap();

        
        // Get the wasm-mutate AST
        let info = ModuleInfo::new(&expected).unwrap();
        println!("Correct parsing of Wasm binary. ");

        // Iterate through the functions and each DFG
        // TODO, returns the new module here
        souperify_peepholes(info).unwrap();
    }
}