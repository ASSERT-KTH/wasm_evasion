use std::{path::PathBuf, rc::Rc, ops::Range};

use wasm_mutate::{WasmMutate, mutators::{Mutator,  custom::CustomSectionMutator, peephole::PeepholeMutator, remove_export::RemoveExportMutator, rename_export::RenameExportMutator, snip_function::SnipMutator}};
use wasmparser::{Chunk, Parser, Payload};
use std::collections::HashMap;
use crate::meta::{Meta, MutationInfo, MutationMap as MM, MutationType};

pub struct InfoExtractor;

macro_rules! get_info {
    ($mutation: expr, $config: ident, $meta: ident, $prettyname: literal, $description: literal, $reduce: literal, $tpe: expr) => {
        { if $mutation.can_mutate(&$config) {

            let info = $mutation.get_mutation_info(&$config);

            let mut idxsmap: HashMap<String, Vec<MM>> = HashMap::new();

            if let Some(info) = info {
                
                for origm in info.iter() {
                    // Group by idx
                    let k = format!("{}", &origm.idx);
                    if !idxsmap.contains_key(&k) {
                        idxsmap.insert(k.clone(), vec![]);
                    }
                    let mdto = MM {
                        section: origm.section.into(),
                        is_indexed: origm.is_indexed,
                        idx: origm.idx,
                        how: origm.how.clone(),
                        many: origm.many,
                        display: origm.display.clone()
                    };        
                    

                    idxsmap.get_mut(&k).unwrap().push(mdto);
                }
            }
            
            $meta.mutations.push(
                MutationInfo{ class_name: format!("{}",stringify!($mutation)), pretty_name:$prettyname.to_string(), desccription: $description.to_string(), map: idxsmap, can_reduce: $reduce, tpe: $tpe.get_val() }
            );
        }
    }
    }
}

impl InfoExtractor {
    pub fn get_info(binary_data: &[u8], meta: &mut Meta) -> crate::errors::AResult<Meta> {
        let mut parser = Parser::new(0);
        let mut wasm = binary_data;
        let mut prev = 0;
        loop {
            let (payload, consumed) = match parser.parse(wasm, true)? {
                Chunk::NeedMoreData(hint) => {
                    panic!("Invalid Wasm module {:?}", hint);
                }
                Chunk::Parsed { consumed, payload } => (payload, consumed),
            };
            match payload {
                Payload::CodeSectionStart {
                    count: _,
                    range,
                    size: _,
                } => {
                    //parser.skip_section();
                    // update slice, bypass the section
                    //wasm = &binary_data[range.end..];
                    //continue;
                }
                Payload::TypeSection(mut reader) => {
                    meta.tpe_section = Some(Range { start: prev, end: prev + consumed});
                }
                Payload::ImportSection(mut reader) => {
                    meta.import_section = Some(Range { start: prev, end: prev + consumed});;
                }
                Payload::FunctionSection(mut reader) => {
                    meta.function_count = reader.get_count();
                }
                Payload::TableSection(mut reader) => {
                    meta.table_section = Some(Range { start: prev, end: prev + consumed});;
                }
                Payload::MemorySection(mut reader) => {
                    meta.memory_count = reader.get_count();
                }
                Payload::GlobalSection(mut reader) => {
                    meta.global_section = Some(Range { start: prev, end: prev + consumed});;
                }
                Payload::ExportSection(mut reader) => {
                    meta.export_section = Some(Range { start: prev, end: prev + consumed});;
                }
                Payload::StartSection { func, range } => {
                    meta.start_section = Some(Range { start: prev, end: prev + consumed});;
                }
                Payload::ElementSection(reader) => {
                    meta.element_section = Some(Range { start: prev, end: prev + consumed});;
                }
                Payload::DataSection(reader) => {
                    meta.data_section = Some(Range { start: prev, end: prev + consumed});;
                }
                Payload::CustomSection (reader) => {
                    meta.custom_sections_count += 1;
                    let name = reader.name();
                    let data = reader.data();
                    meta.custom_sections.insert(name.to_string(), data.len() as u32);
                }
                Payload::UnknownSection {
                    id,
                    contents: _,
                    range,
                } => {
                    meta.unknown_section = Some(Range { start: prev, end: prev + consumed});;
                }
                Payload::DataCountSection { count: _, range } => {
                    meta.data_section = Some(Range { start: prev, end: prev + consumed});;
                }
                Payload::CodeSectionEntry(r) => {

                    // TODO, add mutation info
                    meta.num_instructions += r.get_operators_reader().into_iter().count() as u32;
                }
                Payload::Version { num, .. } => {
                    meta.version = num;

                }
                Payload::InstanceSection(_) => {}
                Payload::TagSection(..) => {
                    meta.tag_section = Some(Range { start: prev, end: prev + consumed});;
                }
                Payload::End { .. } => {
                    break;
                }
                _ => todo!("{:?} not implemented", payload),
            }
            wasm = &wasm[consumed..];
            prev += consumed;
        }

        Ok(meta.clone())
    }

    pub fn get_mutable_info(meta: &mut Meta, config: WasmMutate) -> crate::errors::AResult<Meta> {

        // Check all mutators `can_mutate`, if true, creates a new entry for that mutator and where it can be applied
        let Add = MutationType::Add;
        let Edit = MutationType::Edit;
        let Delete = MutationType::Delete;

        get_info!(PeepholeMutator::new(2), config, meta, "Change custom section", "Changes a custom section. It can be applied ot any custom section in the binary. Usually they are only used to store debug info, such as function names. This mutator can mutate the section name or the data of the section", true, Add|Edit|Delete);

        get_info!(RemoveExportMutator, config, meta, "Change custom section", "Changes a custom section. It can be applied ot any custom section in the binary. Usually they are only used to store debug info, such as function names. This mutator can mutate the section name or the data of the section", true, Add|Edit|Delete);

        get_info!(RenameExportMutator{ max_name_size: 100 }, config, meta, "Change custom section", "Changes a custom section. It can be applied ot any custom section in the binary. Usually they are only used to store debug info, such as function names. This mutator can mutate the section name or the data of the section", true, Add|Edit|Delete);

        get_info!(SnipMutator, config, meta, "Change custom section", "Changes a custom section. It can be applied ot any custom section in the binary. Usually they are only used to store debug info, such as function names. This mutator can mutate the section name or the data of the section", true, Add|Edit|Delete);

        get_info!(CustomSectionMutator, config, meta, "Change custom section", "Changes a custom section. It can be applied ot any custom section in the binary. Usually they are only used to store debug info, such as function names. This mutator can mutate the section name or the data of the section", true, Add|Edit|Delete);


        Ok(meta.clone())
    }
}

#[cfg(test)]
pub mod tests {

    use std::{fs, rc::Rc};

    use crate::meta::Meta;

    use super::InfoExtractor;

    #[test]
    pub fn test_parsing() {
        let content = fs::read("tests/1.wasm").unwrap();

        let info = InfoExtractor::get_info(&content, &mut Meta::new());
    }
}
