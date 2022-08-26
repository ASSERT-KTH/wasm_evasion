//! Mutate custom sections.
#![allow(missing_docs)]

use std::sync::{Arc, atomic::AtomicBool};

use super::{Mutator, MutationMap};
use rand::{seq::SliceRandom, Rng};
use wasm_encoder::{CodeSection, SectionId};
use crate::{probe, send_signal_to_probes_socket};

#[derive(Clone, Copy)]
pub struct CustomSectionMutator;

impl Mutator for CustomSectionMutator {
    fn can_mutate(&self, config: &crate::WasmMutate) -> bool {

        if cfg!(feature="modify_custom_section") {
            config.info().has_custom_section()
        } else {
            false
        }
    }

    fn get_mutation_info(&self, config: &crate::WasmMutate, deeplevel: u32, seed: u64, sample_ratio: u32, stopsignal: Arc<AtomicBool>) -> crate::Result<Option<Vec<super::MutationMap>>> {

        let mut r = vec![];

        let custom_section_indices: Vec<_> = config
            .info()
            .raw_sections
            .iter()
            .enumerate()
            .filter(|(_i, s)| s.id == wasm_encoder::SectionId::Custom as u8)
            .map(|(i, _s)| (i, _s))
            .collect();

        for (idx, s) in custom_section_indices {
            if cfg!(feature="modify_custom_section_name") {
                r.push(MutationMap { 
                    section: SectionId::Custom, 
                    // It is indexed regarding all sections
                    is_indexed: true, idx:  idx as u128, how: "Change the name of the custom section.".to_string(), 
                    many: 1, display: None, meta: None });
            }

            if cfg!(feature="modify_custom_section_data") {
                r.push(MutationMap { 
                    section: SectionId::Custom, 
                    // It is indexed regarding all sections
                    is_indexed: true, idx:  idx as u128, how: "Change the data of the custom section.".to_string(), 
                    many: 1,
                    display: None, meta: None })
            }
        }

        Ok(Some(r))
    }

    fn mutate<'a>(
        self,
        config: &'a mut crate::WasmMutate,
    ) -> crate::Result<Box<dyn Iterator<Item = crate::Result<wasm_encoder::Module>> + 'a>> {
        let custom_section_indices: Vec<_> = config
            .info()
            .raw_sections
            .iter()
            .enumerate()
            .filter(|(_i, s)| s.id == wasm_encoder::SectionId::Custom as u8)
            .map(|(i, _s)| i)
            .collect();
        assert!(!custom_section_indices.is_empty());

        let custom_section_index = *custom_section_indices.choose(config.rng()).unwrap();
        let old_custom_section = &config.info().raw_sections[custom_section_index];
        let old_custom_section =
            wasmparser::CustomSectionReader::new(old_custom_section.data, 0).unwrap();

        let name_string;
        let data_vec;
        let mut name = old_custom_section.name();
        let mut data = old_custom_section.data();

        if cfg!(feature="modify_custom_section_name") {
            // Mutate the custom section's name.
            let mut new_name = name.to_string().into_bytes();
            config.raw_mutate(
                &mut new_name,
                if config.reduce {
                    name.len().saturating_sub(1)
                } else {
                    std::cmp::max(name.len() * 2, 32)
                },
            )?;
            name_string = String::from_utf8_lossy(&new_name).to_string();
            name = &name_string;
        }
        if cfg!(feature="modify_custom_section_data") {
            // Mutate the custom section's data.
            let mut new_data = data.to_vec();
            config.raw_mutate(
                &mut new_data,
                if config.reduce {
                    data.len().saturating_sub(1)
                } else {
                    std::cmp::max(data.len() * 2, 32)
                },
            )?;
            data_vec = new_data;
            data = &data_vec;
        };

        Ok(Box::new(std::iter::once(Ok(config
            .info()
            .replace_section(
                custom_section_index,
                &wasm_encoder::CustomSection { name, data },
            )))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grow_custom_section() {
        crate::mutators::match_mutation(
            r#"
                (module
                    (@custom "name" "data")
                )
            "#,
            CustomSectionMutator,
            r#"
                (module
                    (@custom "name" "datadata")
                )
            "#,
        );
    }

    #[test]
    fn test_shrink_custom_section() {
        crate::mutators::match_mutation(
            r#"
                (module
                    (@custom "name" "data")
                )
            "#,
            CustomSectionMutator,
            r#"
                (module
                    (@custom "name" "d")
                )
            "#,
        );
    }

    #[test]
    fn test_mutate_custom_section() {
        crate::mutators::match_mutation(
            r#"
                (module
                    (@custom "name" "data")
                )
            "#,
            CustomSectionMutator,
            r#"
                (module
                    (@custom "name" "aaaa")
                )
            "#,
        );
    }

    #[test]
    fn test_mutate_custom_section_name() {
        crate::mutators::match_mutation(
            r#"
                (module
                    (@custom "name" "data")
                )
            "#,
            CustomSectionMutator,
            r#"
                (module
                    (@custom "n" "data")
                )
            "#,
        );
    }
}
