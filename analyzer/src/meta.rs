use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meta {
    pub id: String,
    pub size: usize,
    pub tpe: String,

    // Static info
    pub tpe_section: bool,
    pub import_section: bool,
    pub export_section: bool,
    pub function_count: u32,
    pub table_section: bool,
    pub memory_count: u32,
    pub global_section: bool,
    pub start_section: bool,
    pub element_section: bool,
    pub data_section: bool,
    pub unknown_section: bool,
    pub version: u32,
    pub tag_section: bool,

    // custom data info
    #[serde(skip)]
    pub custom_sections: HashMap<String, u32>,
    pub custom_sections_count: u32,

    // code data aggregation
    pub num_instructions: u32
}

impl Meta {
    pub fn new() -> Meta {
        Meta {
            id: "unset".to_string(),
            size: 0,
            tpe: "original".to_string(),

            tpe_section: false,
            import_section: false,
            export_section: false,
            function_count: 0,
            table_section: false,
            memory_count: 0,
            global_section: false,
            start_section: false,
            element_section: false,
            data_section: false,
            unknown_section: false,
            version: 1,
            tag_section: false,
            
            custom_sections: HashMap::new(),

            custom_sections_count: 0,
            num_instructions: 0,
        }
    }
}
