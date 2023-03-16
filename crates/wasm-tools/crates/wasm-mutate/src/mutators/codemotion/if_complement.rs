//! This mutator selects a random `if` construction in a function and swap its branches.
//! Since this mutator preserves the original semantic of the input Wasm,
//! before the mutated if structure is encoded, a "negation" of the previous operand
//! in the stack is written. The "negation" is encoded with a `i32.eqz` operator.
use std::collections::HashMap;
use crate::{probe, send_signal_to_probes_socket};

use rand::prelude::SliceRandom;
use wasm_encoder::{Function, Instruction, ValType};

use crate::{
    module::map_block_type,
    mutators::{
        codemotion::{
            ir::{parse_context::Ast, AstWriter},
            AstMutator,
        },
        OperatorAndByteOffset, MutationMap,
    },
    WasmMutate,
};

/// This mutator selects a random `if` construction in a function and swap its branches.
/// Since this mutator preserves the original semantic of the input Wasm,
/// before the mutated if structure is encoded, a "negation" of the previous operand
/// in the stack is written. The "negation" is encoded with a `i32.eqz` operator.
pub struct IfComplementMutator;

#[derive(Default)]
struct IfComplementWriter {
    if_to_mutate: usize,
}

impl IfComplementWriter {
    /// Swap the then and alternative branches and negates the expected value for the "if" condition
    fn write_complement<'a>(
        &self,
        ast: &Ast,
        _: usize,
        then: &[usize],
        alternative: &Option<Vec<usize>>,
        newfunc: &mut wasm_encoder::Function,
        operators: &Vec<crate::mutators::OperatorAndByteOffset>,
        input_wasm: &'a [u8],
        ty: &wasmparser::BlockType,
    ) -> crate::Result<()> {
        // negate the value on the stack
        newfunc.instruction(&Instruction::I32Eqz);
        newfunc.instruction(&Instruction::If(map_block_type(*ty)?));

        // Swap, write alternative first
        if let Some(alternative) = alternative {
            for ch in alternative {
                self.write(ast, *ch, newfunc, operators, input_wasm)?;
            }
        } else {
            // Write an unreachable instruction
            newfunc.instruction(&Instruction::Nop);
        }
        newfunc.instruction(&Instruction::Else);
        for ch in then {
            self.write(ast, *ch, newfunc, operators, input_wasm)?;
        }
        newfunc.instruction(&Instruction::End);

        Ok(())
    }
}

impl AstWriter for IfComplementWriter {
    /// Replaces the default implementation of the if_else_writing
    fn write_if_else<'a>(
        &self,
        ast: &Ast,
        nodeidx: usize,
        then: &[usize],
        alternative: &Option<Vec<usize>>,
        newfunc: &mut wasm_encoder::Function,
        operators: &Vec<crate::mutators::OperatorAndByteOffset>,
        input_wasm: &'a [u8],
        ty: &wasmparser::BlockType,
    ) -> crate::Result<()> {
        if self.if_to_mutate == nodeidx {
            self.write_complement(
                ast,
                nodeidx,
                then,
                alternative,
                newfunc,
                operators,
                input_wasm,
                ty,
            )?;

            probe!("Write if complement {}/{}", self.if_to_mutate, ast.get_nodes().len());
        } else {
            self.write_if_else_default(
                ast,
                nodeidx,
                then,
                alternative,
                newfunc,
                operators,
                input_wasm,
                ty,
            )?;
        }
        Ok(())
    }
}

impl AstMutator for IfComplementMutator {
    fn can_mutate<'a>(&self, _: &crate::WasmMutate, ast: &Ast) -> bool {
        if !cfg!(feature="code_motion_ifs") {
            return false
        }
        ast.has_if()
    }
    
    /// Checks if this mutator can be applied to the passed `ast`
    fn get_mutation_info<'a>(&self, fidx: u32, deeplevel: u32, config: &'a crate::WasmMutate, ast: &Ast) -> Option<Vec<MutationMap>> {

        let mut results = vec![];
        let ifs = ast.get_ifs();

        for &if_ in ifs {

            let targetid: u128 = 0;
            let targetid = targetid | ((fidx as u128) << 31);
            let targetid = targetid | if_ as u128;

            let mut meta: HashMap<String, String> = HashMap::new();
            meta.insert("if_index".to_string(), format!("{}", if_));
            meta.insert("function_index".to_string(), format!("{}", fidx));

            let info = MutationMap {
                section: wasm_encoder::SectionId::Code,
                is_indexed: true,
                idx: targetid,
                how: "Invert If construction".to_string(),
                many: 1, // This happens only onece
                display: { if deeplevel > 2 { Some(ast.pretty()) } else { None } },
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
        let if_index = ast
            .get_ifs()
            .choose(config.rng())
            .expect("This mutator should check first if the AST contains at least one if");
        let writer = IfComplementWriter {
            if_to_mutate: *if_index,
        };
        writer.write(ast, ast.get_root(), &mut newfunc, operators, input_wasm)?;
        Ok(newfunc)
    }
}
