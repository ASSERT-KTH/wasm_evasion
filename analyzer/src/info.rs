use std::{path::PathBuf, rc::Rc};

use wasmparser::{Chunk, Parser, Payload};

use crate::meta::Meta;

pub struct InfoExtractor;

impl InfoExtractor {
    pub fn get_info(binary_data: &[u8], meta: &mut Meta) -> crate::errors::AResult<Meta> {
        let mut parser = Parser::new(0);
        let mut wasm = binary_data;

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
                    meta.tpe_section = true;
                }
                Payload::ImportSection(mut reader) => {
                    meta.import_section = true;
                }
                Payload::FunctionSection(mut reader) => {
                    meta.function_count = reader.get_count();
                }
                Payload::TableSection(mut reader) => {
                    meta.table_section = true;
                }
                Payload::MemorySection(mut reader) => {
                    meta.memory_count = reader.get_count();
                }
                Payload::GlobalSection(mut reader) => {
                    meta.global_section = true;
                }
                Payload::ExportSection(mut reader) => {
                    meta.export_section = true;
                }
                Payload::StartSection { func, range } => {
                    meta.start_section = true;
                }
                Payload::ElementSection(reader) => {
                    meta.element_section = true;
                }
                Payload::DataSection(reader) => {
                    meta.data_section = true;
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
                    meta.unknown_section = true;
                }
                Payload::DataCountSection { count: _, range } => {
                    meta.data_section = true;
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
                    meta.tag_section = true;
                }
                Payload::End { .. } => {
                    break;
                }
                _ => todo!("{:?} not implemented", payload),
            }
            wasm = &wasm[consumed..];
        }

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
