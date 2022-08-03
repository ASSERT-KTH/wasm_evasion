use std::cell::RefCell;
use std::hash::Hash;
use std::ops::Range;

use crate::meta::{Meta, MutationInfo, MutationMap as MM, MutationType};
use crate::State;
use std::collections::HashMap;
use wasm_mutate::mutators::codemotion::CodemotionMutator;
use wasm_mutate::mutators::remove_item::RemoveItemMutator;
use wasm_mutate::{
    mutators::{
        add_function::AddFunctionMutator, add_type::AddTypeMutator, custom::CustomSectionMutator,
        function_body_unreachable::FunctionBodyUnreachable,
        modify_init_exprs::InitExpressionMutator, peephole::PeepholeMutator,
        remove_export::RemoveExportMutator, remove_section::RemoveSection,
        rename_export::RenameExportMutator, snip_function::SnipMutator, Item, Mutator,
    },
    WasmMutate,
};
use wasmparser::{Chunk, Parser, Payload};
pub struct InfoExtractor;

macro_rules! get_info {
    ($mutation: expr, $config: ident, $state: ident, $meta: ident, $prettyname: literal, $description: literal, $reduce: literal, $tpe: expr, $affects_execution: literal, $rs: ident, $seed: ident, $sample_ratio: ident) => {
        { if $config.is_some() && $mutation.can_mutate(&$config) {

            let mut idxsmap: HashMap<String, Vec<MM>> = HashMap::new();
            if $state > 0 {
                // The can mutate needs to be more deep, the code motio for example is returning true, when it is not checking for code motion
                let info = $mutation.get_mutation_info(&$config, $state, $seed, $sample_ratio);


                // TODO, get the seed to reach a mutation over the specific target

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
                            idx: origm.idx.to_be_bytes().to_vec(),
                            how: origm.how.clone(),
                            many: origm.many,
                            display: origm.display.clone(),
                            meta: origm.meta.clone()
                        };


                        idxsmap.get_mut(&k).unwrap().push(mdto);
                    }
                }
            }
            $rs.push(
                (MutationInfo{ class_name: format!("{}",stringify!($mutation)), pretty_name:$prettyname.to_string(), desccription: $description.to_string(), map: (0, "".into()), can_reduce: $reduce, tpe: $tpe.get_val(), affects_execution:$affects_execution, generic_map: None }, idxsmap)
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
                    range: _,
                    size: _,
                } => {
                    //parser.skip_section();
                    // update slice, bypass the section
                    //wasm = &binary_data[range.end..];
                    //continue;
                }
                Payload::TypeSection(reader) => {
                    meta.tpe_section = Some(Range {
                        start: prev,
                        end: prev + consumed,
                    });
                    meta.num_tpes = reader.get_count();
                }
                Payload::ImportSection(reader) => {
                    meta.import_section = Some(Range {
                        start: prev,
                        end: prev + consumed,
                    });
                    meta.num_imports = reader.get_count();
                }
                Payload::FunctionSection(reader) => {
                    meta.function_count = reader.get_count();
                }
                Payload::TableSection(reader) => {
                    meta.table_section = Some(Range {
                        start: prev,
                        end: prev + consumed,
                    });
                    meta.num_tables = reader.get_count();
                }
                Payload::MemorySection(reader) => {
                    meta.memory_count = reader.get_count();
                }
                Payload::GlobalSection(reader) => {
                    meta.global_section = Some(Range {
                        start: prev,
                        end: prev + consumed,
                    });
                    meta.num_globals = reader.get_count();
                }
                Payload::ExportSection(reader) => {
                    meta.export_section = Some(Range {
                        start: prev,
                        end: prev + consumed,
                    });
                    meta.num_exports = reader.get_count();
                }
                Payload::StartSection { func: _, range: _ } => {
                    meta.start_section = Some(Range {
                        start: prev,
                        end: prev + consumed,
                    });
                }
                Payload::ElementSection(reader) => {
                    meta.element_section = Some(Range {
                        start: prev,
                        end: prev + consumed,
                    });
                    meta.num_elements = reader.get_count();
                }
                Payload::DataSection(reader) => {
                    meta.data_section = Some(Range {
                        start: prev,
                        end: prev + consumed,
                    });
                    meta.num_data_segments = reader.get_count();
                }
                Payload::CustomSection(reader) => {
                    meta.custom_sections_count += 1;
                    let name = reader.name();
                    let data = reader.data();
                    meta.custom_sections
                        .insert(name.to_string(), (0, data.len() as u32));
                }
                Payload::UnknownSection {
                    id: _,
                    contents: _,
                    range: _,
                } => {
                    meta.unknown_section = Some(Range {
                        start: prev,
                        end: prev + consumed,
                    });
                }
                Payload::DataCountSection { count: c, range: _ } => {
                    meta.data_section = Some(Range {
                        start: prev,
                        end: prev + consumed,
                    });

                    meta.num_data = c;
                }
                Payload::CodeSectionEntry(r) => {
                    // TODO, add mutation info
                    let reader = r.get_operators_reader()?;
                    meta.num_instructions += reader.into_iter().count() as u32;
                }
                Payload::Version { num, .. } => {
                    meta.version = num;
                }
                Payload::InstanceSection(_) => {}
                Payload::TagSection(reader) => {
                    meta.tag_section = Some(Range {
                        start: prev,
                        end: prev + consumed,
                    });
                    meta.num_tags = reader.get_count();
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

    pub fn get_mutable_info(
        meta: &mut Meta,
        config: WasmMutate,
        state: u32,
        seed: u64,
        sample_ratio: u32,
    ) -> crate::errors::AResult<(Meta, Vec<(MutationInfo, HashMap<String, Vec<MM>>)>)> {
        // Check all mutators `can_mutate`, if true, creates a new entry for that mutator and where it can be applied
        let Add = MutationType::Add;
        let Edit = MutationType::Edit;
        let Delete = MutationType::Delete;
        let mut rs = vec![];

        // iterate through the heights if depth > 5

        if state > 4 {
            for d in 1..=state {
                get_info!(
                    PeepholeMutator::new(d), // This will just see if the egraph can be contructed from it, then we sould iteratively increase this. 
                    config,
                    state,
                    meta,
                    "Apply a peephole mutation",
                    "Changes a function to the peephole level. It uses an egraphs to create the mutations",
                    true,
                    Add | Edit | Delete,
                    true, rs, seed, sample_ratio
                );
            }
        } else {
            get_info!(
                PeepholeMutator::new(2 /* 2 is the defalt value used by wasm-mutate */), // This will just see if the egraph can be contructed from it, then we sould iteratively increase this. 
                config,
                state,
                meta,
                "Apply a peephole mutation",
                "Changes a function to the peephole level. It uses an egraphs to create the mutations",
                true,
                Add | Edit | Delete,
                true, rs, seed, sample_ratio
            );
        }

        get_info!(
            RemoveExportMutator,
            config,
            state,
            meta,
            "Remove an export",
            "Remove an export",
            true,
            Delete,
            true,
            rs,
            seed,
            sample_ratio
        );
        get_info!(
            RenameExportMutator { max_name_size: 100 },
            config,
            state,
            meta,
            "Rename an export",
            "Renames an export",
            true,
            Edit,
            false,
            rs,
            seed,
            sample_ratio
        );
        get_info!(SnipMutator, config,
            state, meta, "Snip a function body", "Removes the body of a function and replaces its body by a default value given the type of the function", true, Delete, true, rs, seed, sample_ratio);

        // Split into the two types of current mutators
        get_info!(
            CodemotionMutator,
            config,
            state,
            meta,
            "Code motion mutator",
            "Changes the cfg of the function body",
            false,
            Edit,
            true,
            rs,
            seed,
            sample_ratio
        );

        get_info!(
            FunctionBodyUnreachable,
            config,
            state,
            meta,
            "Set function to unreachable",
            "Replaces a function body by unreachable",
            true,
            Delete | Edit,
            true,
            rs,
            seed,
            sample_ratio
        );
        get_info!(
            AddTypeMutator {
                max_params: 20,
                max_results: 20,
            },
            config,
            state,
            meta,
            "Add type",
            "Adds a new random type declaration to the binary",
            false,
            Add,
            false,
            rs,
            seed,
            sample_ratio
        );

        get_info!(
            AddFunctionMutator,
            config,
            state,
            meta,
            "Add function",
            "Adds a custom random created function to the binary",
            false,
            Add,
            false,
            rs,
            seed,
            sample_ratio
        );
        get_info!(
            RemoveSection::Custom,
            config,
            state,
            meta,
            "Remove custom section",
            "Removes a custom section",
            true,
            Delete,
            false,
            rs,
            seed,
            sample_ratio
        );
        get_info!(
            RemoveSection::Empty,
            config,
            state,
            meta,
            "Remove empty section",
            "Removes empty section",
            true,
            Delete,
            false,
            rs,
            seed,
            sample_ratio
        );

        get_info!(
            InitExpressionMutator::Global,
            config,
            state,
            meta,
            "Init expression mutator",
            "Mutates the initial expression of a global",
            true,
            Edit,
            true,
            rs,
            seed,
            sample_ratio
        );

        get_info!(
            InitExpressionMutator::ElementOffset,
            config,
            state,
            meta,
            "Element offset mutation",
            "Mutate the init expression of the element offset",
            true,
            Edit,
            true,
            rs,
            seed,
            sample_ratio
        );

        get_info!(
            InitExpressionMutator::ElementFunc,
            config,
            state,
            meta,
            "Element func mutation",
            "Mutate the init expression of the element func",
            true,
            Edit,
            true,
            rs,
            seed,
            sample_ratio
        );

        get_info!(
            RemoveItemMutator(Item::Function),
            config,
            state,
            meta,
            "Removes function",
            "Removes a ramdon function",
            true,
            Delete,
            true,
            rs,
            seed,
            sample_ratio
        );
        get_info!(
            RemoveItemMutator(Item::Global),
            config,
            state,
            meta,
            "Remove global",
            "Removes a random global",
            true,
            Delete,
            true,
            rs,
            seed,
            sample_ratio
        );
        get_info!(
            RemoveItemMutator(Item::Memory),
            config,
            state,
            meta,
            "Remove memory",
            "Removes a memory element",
            true,
            Delete,
            true,
            rs,
            seed,
            sample_ratio
        );
        get_info!(
            RemoveItemMutator(Item::Table),
            config,
            state,
            meta,
            "Remove table",
            "Removes a table",
            true,
            Delete,
            true,
            rs,
            seed,
            sample_ratio
        );
        get_info!(
            RemoveItemMutator(Item::Type),
            config,
            state,
            meta,
            "Remove type",
            "Removes a type",
            true,
            Delete,
            false,
            rs,
            seed,
            sample_ratio
        );
        get_info!(
            RemoveItemMutator(Item::Data),
            config,
            state,
            meta,
            "Remove data",
            "Remove data segment",
            true,
            Delete,
            true,
            rs,
            seed,
            sample_ratio
        );
        get_info!(
            RemoveItemMutator(Item::Element),
            config,
            state,
            meta,
            "Remove elemen",
            "Removes element",
            true,
            Delete,
            true,
            rs,
            seed,
            sample_ratio
        );
        get_info!(
            RemoveItemMutator(Item::Tag),
            config,
            state,
            meta,
            "Remove tahg",
            "Remove tag",
            true,
            Delete,
            true,
            rs,
            seed,
            sample_ratio
        );

        get_info!(CustomSectionMutator, config,
            state, meta, "Change custom section", "Changes a custom section. It can be applied ot any custom section in the binary. Usually they are only used to store debug info, such as function names. This mutator can mutate the section name or the data of the section", true, Edit, false, rs, seed, sample_ratio);

        Ok((meta.clone(), rs))
    }
}

#[cfg(test)]
pub mod tests {

    use std::fs;

    use crate::meta::Meta;

    use super::InfoExtractor;

    #[test]
    pub fn test_parsing() {
        let content = fs::read("tests/1.wasm").unwrap();

        let _info = InfoExtractor::get_info(&content, &mut Meta::new());
    }
}
