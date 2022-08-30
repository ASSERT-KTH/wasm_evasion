use std::{ffi::CStr, sync::Mutex};

use egg::{Id, EGraph, Language, Runner, RecExpr, rewrite, Rewrite, Searcher, Pattern};
use libc::c_char;
use wasm_encoder::{Function, ValType, CodeSection, GlobalSection, Instruction};
use wasm_mutate::{info::ModuleInfo, module::{PrimitiveTypeInfo, TypeInfo, map_type}, mutators::peephole::{eggsy::{analysis::PeepholeMutationAnalysis, lang::Lang, encoder::{Encoder, expr2wasm::ResourceRequest}}, dfg::DFGBuilder}, WasmMutate};
use wasmparser::{LocalsReader, CodeSectionReader, FunctionBody, GlobalSectionReader};
use wasmtime::Module;

use std::collections::HashMap;

use crate::{parser::souper2Lang, OperatorAndByteOffset};

lazy_static! {
    // Once we have the static REPLACEMENT_MAP filled up with the possible replacements, then we do something
    static ref REPLACEMENT_MAP: Mutex<HashMap<String, Vec<(String, i32)>>> = {
        let mut m = Mutex::new(HashMap::new());
        m
    };
}

pub struct Superdiversifier;

impl Superdiversifier {

    // Collect and unfold params and locals, [x, ty, y, ty2] -> [ty....ty, ty2...ty2]
    fn get_func_locals(
        &self,
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

    fn generate_souper_query(&self,root: Id, egraph: EGraph<Lang, PeepholeMutationAnalysis>) -> anyhow::Result<(String, u32)> {
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
        // Return the result and the number of operands
        Ok((result, varidx))
    }

    extern "C" fn callback(lhs_ptr: *const i8 /* Original Query */, rhs_ptr: *const i8, cost: i32) -> i32 {
        let rhs = unsafe { CStr::from_ptr(rhs_ptr) };
        println!("RHS {:?}\n Cost: {}", rhs, cost);
        let lang = unsafe { CStr::from_ptr(lhs_ptr) };
        println!("LHS {:?}", lang);

        let lang = souper2Lang(rhs.to_str().unwrap()).unwrap(); // Save this into a global State ?
        let lhs = unsafe { CStr::from_ptr(lhs_ptr) };
        let lhs = lhs.to_str().unwrap().to_string();

        if !REPLACEMENT_MAP.lock().unwrap().contains_key(&lhs){
            REPLACEMENT_MAP.lock().unwrap().insert(lhs.clone(), vec![]);
        }

        REPLACEMENT_MAP.lock().unwrap().get_mut(&lhs).unwrap().push((lang.to_string(), cost));
        0
    }

 
    fn copy_locals(&self, reader: FunctionBody) -> anyhow::Result<Function> {
        // Create the new function
        let mut localreader = reader.get_locals_reader()?;
        // Get current locals and map to encoder types
        let mut local_count = 0;
        let current_locals = (0..localreader.get_count())
            .map(|_| {
                let (count, ty) = localreader.read().unwrap();
                local_count += count;
                (count, map_type(ty).unwrap())
            })
            .collect::<Vec<(u32, ValType)>>();

        Ok(Function::new(current_locals /*copy locals here*/))
    }

    pub fn superoptimize(&self, config: &mut WasmMutate, replacement_map: HashMap<String, Vec<(String, i32)>>) -> anyhow::Result<wasm_encoder::Module> {

        println!("Superoptimizing");


        // Create the rewriting rules with the replacement maps
        let mut rules: Vec<Rewrite<Lang, PeepholeMutationAnalysis>> = vec![];
        let mut i = 0;
        for (k, v) in replacement_map.iter() {
            
            // Get best replacement.
            let mut best: Option<String> = None;
            let mut bestcost  = i32::MAX;

            for (rep, cost) in v.iter() {
                if *cost < bestcost {
                    bestcost = *cost;
                    best = Some(rep.clone());
                } 
            }

            if let Some(rhs) = best {
                let cplhs = k.clone();
                match cplhs.parse::<Pattern<_>>() {
                    Ok(lhs) => {
                        println!("{} <=> {}", cplhs.clone(), rhs.clone());
                        rules.push(
                            Rewrite::new(
                                format!("Souper replacement {i}"),
                                "Souper replacement",
                                lhs.clone(),
                                rhs.clone().parse::<Pattern<_>>().unwrap()
                            ).unwrap()
                        );
                        rules.push(
                            Rewrite::new(
                                format!("Souper replacement {i} reverse"),
                                "Souper replacement",
                                rhs.clone().parse::<Pattern<_>>().unwrap(),
                                lhs,
                            ).unwrap()
                        );
                        i += 1;
                    }
                    Err(e) => {
                        log::error!("{} {}", e, cplhs);
                    }
                }
            }
        }

        if rules.len() == 0 {
            println!("No rules to apply");
            return Ok(config.info().replace_multiple_sections(|_,_,_| false));
        }

        // Get the first key as the init expression
        
        let code_section = config.info().get_code_section();
        let mut sectionreader = CodeSectionReader::new(code_section.data, 0)?;
        let function_count = sectionreader.get_count();

        log::debug!("Function count {}. ", function_count);
        //let mut config = WasmMutate::default();

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

            let locals = self.get_func_locals(
                &config.info(),
                i + config.info().num_imported_functions(), /* the function type is shifted
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

                let minidfg = dfg.get_dfg(&config.info(), &operators, &basicblock);

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
                println!("Looking for valid replacements {}", start);


                // Create the egraph
                let analysis = PeepholeMutationAnalysis::new(&config.info(), /* For now pass the locals empty */ locals.clone());
                let runner = Runner::<Lang, PeepholeMutationAnalysis, ()>::new(analysis)
                    .with_expr(&start)
                    .run(&rules)
                    .with_hook(|runner|{
                        println!("The egraph is {} this big", runner.egraph.total_size());
                        Ok(())
                    });

                let mut egraph = runner.egraph;                    
                
                // In theory this will return the Id of the operator eterm
                let root = egraph.add_expr(&start);
                egraph.rebuild();

                let mut extractor = egg::Extractor::new(&egraph, egg::AstSize);
                let (_best_cost, best_expr) = extractor.find_best(root);

                println!("best, {}", best_expr);
                if start != best_expr {
                    println!("Replacing {} => {}", start, best_expr);

                    let mut newfunc = self.copy_locals(reader)?;

                    let needed_resources = Encoder::build_function(
                        config,
                        
                        reverseidx,
                        &best_expr,
                        &operators,
                        &basicblock,
                        &mut newfunc,
                        &minidfg,
                        &egraph,
                    )?;

                    let mut codes = CodeSection::new();
                    let code_section = config.info().get_code_section();
                    let mut sectionreader = CodeSectionReader::new(code_section.data, 0)?;

                    // this mutator is applicable to internal functions, so
                    // it starts by randomly selecting an index between
                    // the imported functions and the total count, total=imported + internal
                    for fidx in 0..config.info().num_local_functions() {
                        let reader = sectionreader.read()?;
                        if fidx == i {
                            println!("Replacing...{}", i);
                            codes.function(&newfunc);
                        } else {
                            codes.raw(
                                &code_section.data[reader.range().start..reader.range().end],
                            );
                        }
                    }

                    // Here break...this will prevent unconsistent replacements
                    
                        // Process the outside function needed resources
                        // Needed globals
                        let mut new_global_section = GlobalSection::new();
                        // Reparse and reencode global section
                        if let Some(_) = config.info().globals {
                            // If the global section was already there, try to copy it to the
                            // new raw section
                            let global_section = config.info().get_global_section();
                            let mut globalreader =
                                GlobalSectionReader::new(global_section.data, 0)?;
                            let count = globalreader.get_count();
                            let mut start = globalreader.original_position();

                            for _ in 0..count {
                                let _ = globalreader.read()?;
                                let current_pos = globalreader.original_position();
                                let global = &global_section.data[start..current_pos];
                                new_global_section.raw(global);
                                start = current_pos;
                            }
                        }

                        if needed_resources.len() > 0 {
                            log::trace!("Adding {} additional resources", needed_resources.len());
                        }

                        for resource in &needed_resources {
                            match resource {
                                ResourceRequest::Global {
                                    index: _,
                                    tpe,
                                    mutable,
                                } => {
                                    // Add to globals
                                    new_global_section.global(
                                        wasm_encoder::GlobalType {
                                            mutable: *mutable,
                                            val_type: match tpe {
                                                PrimitiveTypeInfo::I32 => ValType::I32,
                                                PrimitiveTypeInfo::I64 => ValType::I64,
                                                PrimitiveTypeInfo::F32 => ValType::F32,
                                                PrimitiveTypeInfo::F64 => ValType::F64,
                                                PrimitiveTypeInfo::V128 => ValType::V128,
                                                _ => {
                                                    unreachable!("Not valid for globals")
                                                }
                                            },
                                        },
                                        match tpe {
                                            PrimitiveTypeInfo::I32 => &Instruction::I32Const(0),
                                            PrimitiveTypeInfo::I64 => &Instruction::I64Const(0),
                                            PrimitiveTypeInfo::F32 => &Instruction::F32Const(0.0),
                                            PrimitiveTypeInfo::F64 => &Instruction::F64Const(0.0),
                                            PrimitiveTypeInfo::V128 => &Instruction::V128Const(0),
                                            _ => {
                                                unreachable!("Not valid for globals")
                                            }
                                        },
                                    );
                                }
                            }
                        }

                        let code_index = config.info().code;
                        let global_index = config.info().globals;

                        // This conditional placing enforces to write the global
                        // section by respecting its relative order in the Wasm module
                        let insert_globals_before = config
                            .info()
                            .globals
                            .or(config.info().exports)
                            .or(config.info().start)
                            .or(config.info().elements)
                            .or(config.info().data_count)
                            .or(code_index);

                        // If the mutator is in this staeg, then it passes the can_mutate flter,
                        // which checks for code section existance
                        let insert_globals_before = insert_globals_before.unwrap();
                        let module = config.info().replace_multiple_sections(
                            move |index, _sectionid, module: &mut wasm_encoder::Module| {
                                if insert_globals_before == index
                            // Write if needed or if it wasm in the init Wasm
                            && (new_global_section.len() > 0 || global_index.is_some() )
                                {
                                    // Insert the new globals here
                                    module.section(&new_global_section);
                                }
                                if index == code_index.unwrap() {
                                    // Replace code section
                                    module.section(&codes);

                                    return true;
                                }
                                if let Some(gidx) = global_index {
                                    // return true since the global section is written by the
                                    // conditional position writer
                                    return gidx == index;
                                }
                                // False to say the underlying encoder to write the prexisting
                                // section
                                false
                            },
                        );
                    return Ok(module)
                }
                // Check if the operator returns an integer value
                // if so, call souper for a replacement
                
            }

        }
        Ok(config.info().replace_multiple_sections(|_,_,_| false))
    }

    /// This method calls Souperdiversifier to construct a map of operator => replacements
    /// The caller shoudl then decide what to do with the replacements, superoptimize or diversify
    pub fn souperdiversify_peepholes(&self, info: ModuleInfo)  -> anyhow::Result<HashMap<String, Vec<(String, i32)>>>{

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

            let locals = self.get_func_locals(
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
                // The returning type of the subexpression should be an integer returning instruction
                match rtpe {
                    PrimitiveTypeInfo::I32 | PrimitiveTypeInfo::I64 => {
                        log::info!("Creating souper IR for subtree");
                        log::debug!("E-expre {}", &start);
                        match self.generate_souper_query(root, egraph) {
                            Ok((souperIR, numoperands)) => {
                                println!("Souper IR '{}'", souperIR);
                                println!("LANG IR '{}'", start);
                                if numoperands > 1 {
                                    let startptr = format!("{}\x00", start.to_string()).as_ptr();

                                    println!("{}",souperIR);                                
                                    // Call Souper here
                                    unsafe {
                                        superoptimize( souperIR.as_ptr() as *const i8, startptr as *const i8 ,Superdiversifier::callback);
                                    }
                                }
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
        }

        // We saved a map...(operator index) => (replacements found by Souper)
        // With this map we can create the Souperdiversifier
        // Each new replacement, we add to the rewriting egraph engine
        // We then run it to get the smallets possible :) => Superoptimization
        // Or, replace one by one and generate new modules
        Ok(REPLACEMENT_MAP.lock().unwrap().clone())
    }



}


pub type CBType = extern "C" fn(*const c_char, *const c_char, i32) -> i32;

extern {
    pub fn hello();
    pub fn superoptimize(lhs: *const c_char, lang: *const c_char, cb: CBType) -> i32;
}