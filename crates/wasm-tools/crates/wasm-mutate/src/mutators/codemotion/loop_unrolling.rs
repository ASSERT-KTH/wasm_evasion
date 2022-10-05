//! This mutator selects a random `loop` construction in a function and tries to unroll it.
//! This mutator only works on empty-returning loops
use std::{collections::HashMap, slice::Iter, iter::empty};
use crate::{probe, send_signal_to_probes_socket};

use rand::prelude::SliceRandom;
use wasm_encoder::{Function, Instruction, ValType};
use wasmparser::{BlockType, Operator};

use crate::{
    module::map_block_type,
    mutators::{
        codemotion::{
            ir::{
                parse_context::{Ast, Node},
                AstWriter,
            },
            AstMutator,
        },
        OperatorAndByteOffset, MutationMap,
    },
    WasmMutate,
};

/// This mutator selects a random `loop` construction in a function and tries to unroll it.
/// This mutator only works on empty-returning loops
pub struct LoopUnrollMutator;

#[derive(Default)]
struct LoopUnrollWriter {
    loop_to_mutate: usize,
}

impl LoopUnrollWriter {
    fn write_and_fix_loop_body<'a>(
        &self,
        chunk: Iter<OperatorAndByteOffset>,
        to_fix: &HashMap<usize, Instruction>,
        newfunc: &mut Function,
        input_wasm: &'a [u8],
    ) -> crate::Result<()> {
        for (idx, ((_, curr_offset), (_, next_offset))) in
            chunk.clone().zip(chunk.skip(1)).enumerate()
        {
            if to_fix.contains_key(&idx) {
                newfunc.instruction(&to_fix[&idx]);
            } else {
                let piece = &input_wasm[*curr_offset..*next_offset];
                newfunc.raw(piece.to_vec());
            }
        }
        Ok(())
    }

    fn unroll_loop<'a>(
        &self,
        ast: &Ast,
        nodeidx: usize,
        newfunc: &mut Function,
        operators: &Vec<OperatorAndByteOffset>,
        input_wasm: &'a [u8],
    ) -> crate::Result<()> {
        let nodes = ast.get_nodes();

        let mut current_depth = 0;
        let mut to_fix = HashMap::new();

        match &nodes[nodeidx] {
            Node::Loop { body: _, ty, range } => {
                let chunk = &operators[range.start + 1 /* skip the loop instruction */..range.end];
                newfunc.instruction(&Instruction::Block(map_block_type(*ty)?));
                for (idx, (op, _)) in chunk.iter().enumerate() {
                    match op {
                        Operator::Block { .. } => {
                            current_depth += 1;
                        }
                        Operator::Loop { .. } => {
                            current_depth += 1;
                        }
                        Operator::If { .. } => {
                            current_depth += 1;
                        }
                        Operator::End { .. } => {
                            current_depth -= 1;
                        }
                        Operator::Br { relative_depth } => {
                            if *relative_depth > current_depth {
                                // Out jump...annotate for fixing
                                to_fix.insert(idx, Instruction::Br(relative_depth + 1));
                            }
                        }
                        Operator::BrIf { relative_depth } => {
                            if *relative_depth > current_depth {
                                // Out jump...annotate for fixing
                                to_fix.insert(idx, Instruction::BrIf(relative_depth + 1));
                            }
                        }
                        Operator::BrTable { table } => {
                            let mut jmpfix = vec![];
                            for i in table.targets() {
                                let d = i?;
                                if d > current_depth {
                                    // Out jump...annotate for fixing
                                    jmpfix.push(d + 1)
                                } else {
                                    jmpfix.push(d)
                                }
                            }

                            let mut def = table.default();
                            if def > current_depth {
                                def += 1;
                            }

                            to_fix.insert(idx, Instruction::BrTable(jmpfix.into(), def));
                        }
                        _ => {}
                    }
                }

                // Write the unroll
                newfunc.instruction(&Instruction::Block(map_block_type(*ty)?));
                // Write A' br B'
                let including_chunk =
                    operators[range.start + 1 /* skip the loop instruction */..range.end + 1]
                        .iter();
                self.write_and_fix_loop_body(including_chunk, &to_fix, newfunc, input_wasm)?;

                // Write A' br B'
                newfunc.instruction(&Instruction::Br(1));
                newfunc.instruction(&Instruction::End);
                // Write the Loop
                newfunc.instruction(&Instruction::Loop(map_block_type(*ty)?));

                // Write A' br B'
                let including_chunk =
                    operators[range.start + 1 /* skip the loop instruction */..range.end + 1]
                        .iter();
                self.write_and_fix_loop_body(including_chunk, &to_fix, newfunc, input_wasm)?;

                // Closing loop
                newfunc.instruction(&Instruction::End);

                // Closing end
                newfunc.instruction(&Instruction::End);
            }
            _ => unreachable!("Invalid node passed as a loop to unroll"),
        }
        Ok(())
    }
}

impl AstWriter for LoopUnrollWriter {
    fn write_loop<'a>(
        &self,
        ast: &Ast,
        nodeidx: usize,
        body: &[usize],
        newfunc: &mut Function,
        operators: &Vec<OperatorAndByteOffset>,
        input_wasm: &'a [u8],
        ty: &wasmparser::BlockType,
    ) -> crate::Result<()> {
        if self.loop_to_mutate == nodeidx {
            self.unroll_loop(ast, nodeidx, newfunc, operators, input_wasm)?;
            probe!("Unroll loop {}/{}", self.loop_to_mutate, ast.get_nodes().len());
        } else {
            self.write_loop_default(ast, nodeidx, body, newfunc, operators, input_wasm, ty)?;
        }
        Ok(())
    }
}

impl LoopUnrollMutator {
    /// Returns the indexes of empty return loop definitions inside the Wasm function
    pub fn get_empty_returning_loops<'a>(&self, ast: &'a Ast) -> Vec<usize> {
        let nodes = ast.get_nodes();
        let mut loops = vec![];
        for idx in ast.get_loops() {
            let n = &nodes[*idx];
            match n {
                Node::Loop {
                    ty,
                    range: _,
                    body: _,
                } => {
                    if let BlockType::Empty = ty {
                        loops.push(*idx)
                    }
                }
                _ => unreachable!("Invalid loop node"),
            }
        }
        loops
    }
}

impl AstMutator for LoopUnrollMutator {
    fn can_mutate<'a>(&self, _config: &crate::WasmMutate, ast: &Ast) -> bool {
        if !cfg!(feature="code_motion_loops") {
            return false
        }
        let empty_returning_loops = self.get_empty_returning_loops(ast);
        !empty_returning_loops.is_empty()
    }


    /// Checks if this mutator can be applied to the passed `ast`
    fn get_mutation_info<'a>(&self, fidx: u32,deeplevel: u32,  config: &'a crate::WasmMutate, ast: &Ast) -> Option<Vec<MutationMap>> {

        let mut results = vec![];
        let empty_returning_loops = self.get_empty_returning_loops(ast);

        for empty_returning_loop in empty_returning_loops {

            let targetid: u128 = 0;
            let targetid = targetid | ((fidx as u128) << 31);
            let targetid = targetid | empty_returning_loop as u128;

            let mut meta: HashMap<String, String> = HashMap::new();
            meta.insert("loop_index".to_string(), format!("{}", empty_returning_loop));
            meta.insert("function_index".to_string(), format!("{}", fidx));

            let info = MutationMap {
                section: wasm_encoder::SectionId::Code,
                is_indexed: true,
                idx: targetid,
                how: "Unrolls a loop".to_string(),
                many: 1, // This happens only onece
                display:  { if deeplevel > 2 { Some(ast.pretty()) } else { None } },
                meta: Some(meta),
            };

            results.push(info);
        }

        Some(results)
    }

    fn mutate<'a>(
        &self,
        config: &'a mut WasmMutate,
        ast: &Ast,
        locals: &[(u32, ValType)],
        operators: &Vec<OperatorAndByteOffset>,
        input_wasm: &'a [u8],
    ) -> crate::Result<Function> {
        // Select the if index
        let mut newfunc = Function::new(locals.to_vec());
        let empty_returning_loops = self.get_empty_returning_loops(ast);
        let loop_index = empty_returning_loops
            .choose(config.rng())
            .expect("This mutator should check first if the AST contains at least one loop node");
        let writer = LoopUnrollWriter {
            loop_to_mutate: *loop_index,
        };
        writer.write(ast, ast.get_root(), &mut newfunc, operators, input_wasm)?;
        Ok(newfunc)
    }
}
