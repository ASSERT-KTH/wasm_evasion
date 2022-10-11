use crate::errors::AResult;
use paste::paste;
use wasm_mutate::mutators::codemotion::CodemotionMutator;
use wasm_mutate::mutators::peephole::PeepholeMutator;
use wasm_mutate::mutators::remove_export::RemoveExportMutator;
use wasm_mutate::mutators::Mutator;
use wasm_mutate::{
    mutators::{
        add_function::AddFunctionMutator, add_type::AddTypeMutator, custom::CustomSectionMutator,
        remove_item::RemoveItemMutator, Item,
    },
    WasmMutate,
};
pub type MutationResult = (bool, &'static str, &'static str, &'static str, Vec<u8>);
use std::fmt::Debug;

// Mutator config level
// e.g. for the peephole, "peephole", "use_global", "100...max tree depth", "0.1"
pub type MutatorWeight = (&'static str, &'static str, &'static str, f32);

macro_rules! _mutate {
    ($feature: ident, $config: ident, $take:ident, $($name: literal, $tpe: literal, $param:literal, $instance: expr)*) => {

      match $feature { $(
            ($name, $tpe, $param) => {
                let mt = $instance;
                let mut cpmut = $config.clone();
                if mt.can_mutate(&cpmut) {

                    match mt.clone().mutate(&mut cpmut) {
                        Ok(it) => {
                            // Always take the first
                            // TODO change to return a random one from here
                            let mut first = it.into_iter().take($take).map(|r| r.map(|m| m.finish()));
                            return Ok((true, $name, $tpe, $param, first.next().unwrap().unwrap()))
                        }
                        Err(e) => {
                            log::error!("Error mutating {:?}", e);
                            return Ok((false, $name, $tpe, $param,  vec![]))
                        }
                    }
                } else {
                    println!("not applicable {}", $name );
                    return Ok((false, $name, $tpe, $param, vec![]))
                }

            }
        ), *
        _ => {
            panic!("Invalid feature {:?}", $feature)
        }
    }
    }
}

pub trait MutationFactory: Debug + Clone {
    fn get_mutators_by_feature(
        self,
        config: &mut WasmMutate,
        feature: &MutatorWeight,
        take: usize,
    ) -> AResult<MutationResult>;
    fn get_available_mutations(self) -> Vec<MutatorWeight>;
    fn description(self) -> &'static str;
}

/// Statically creates the functions and the matches for the mutators
macro_rules! create_all_with_weight {
    // TODO use paste
    ($funcname: literal => $(
        $tpe: literal, $name: literal, $param:literal, $weight: literal
        ($instance: expr)
    ) *) => {

        paste::item! {

            #[derive(Debug, Clone)]
            pub struct [<MutationFactory $funcname>];

            impl MutationFactory for [<MutationFactory $funcname>] {

                fn get_mutators_by_feature(self, config: &mut WasmMutate, feature: &MutatorWeight, take: usize)  -> AResult<MutationResult> {

                    let (name, tpe, param, _weight) = *feature;
                    let tocompare = (name, tpe, param);

                    _mutate!(
                        tocompare, config, take,
                        $(
                            $tpe, $name, $param, $instance
                        )*
                    )
                }

                fn get_available_mutations(self) -> Vec<MutatorWeight> {
                    vec![
                        $(
                            ($tpe, $name, $param, $weight),
                        )*
                    ]
                }

                fn description(self) -> &'static str {
                    return $funcname;
                }
            }
        }
    };
}

// Preset collection of mutations with weights

create_all_with_weight!("Uniform" =>

    "modify", "custom_section_data", "", 0.1 (
        CustomSectionMutator
    )

    "add", "function", "", 0.1 (
        AddFunctionMutator
    )

    "remove", "type", "", 0.1 (
        RemoveItemMutator(Item::Type)
    )

    "remove", "function", "", 0.1 (
        RemoveItemMutator(Item::Function)
    )

    "remove", "global", "", 0.1 (
        RemoveItemMutator(Item::Global)
    )

    "remove", "memory", "", 0.1 (
        RemoveItemMutator(Item::Memory)
    )

    "remove", "table", "", 0.1 (
        RemoveItemMutator(Item::Table)
    )

    "remove", "data", "", 0.1 (
        RemoveItemMutator(Item::Data)
    )

    "remove", "element", "", 0.1 (
        RemoveItemMutator(Item::Element)
    )

    "remove", "tag", "", 0.1 (
        RemoveItemMutator(Item::Tag)
    )

    // small, medium and large for the same mutator
    "add", "type", "2020", 0.1 (
        AddTypeMutator {
            max_params: 20,
            max_results: 20
        }
    )

    "add", "type", "1010", 0.1 (
        AddTypeMutator {
            max_params: 10,
            max_results: 10
        }
    )

    // Large mutations are detected first !!
    //"peephole", "simple", "2", 0.1 (
    //    PeepholeMutator::new(1)
    //    // TODO Set the rules here
    //)

    "peephole", "simple", "10", 0.1 (
        PeepholeMutator::new(2)
        // TODO Set the rules here
    )
    /*"peephole", "medium", "10", 0.1 (
        PeepholeMutator::new(10)
        // TODO Set the rules here
    )

    "peephole", "large", "20", 0.1 (
        PeepholeMutator::new(20)
        // TODO Set the rules here
    )

    "codemotion", "", "", 0.1 (
        CodemotionMutator
    )

    "remove_export", "", "", 0.1 (
        RemoveExportMutator
    )*/

    // TODO add the other mutators here
);

pub fn get_by_name(name: &'static str) -> impl MutationFactory {
    match name {
        "Uniform" => MutationFactoryUniform,
        _ => panic!("Invalid mutation factory {}", name),
    }
}
