use std::{collections::HashMap, io::Write, sync::{atomic::{AtomicU32, Ordering, AtomicBool}, Arc}, fs::File, time};

use clap::{ArgMatches, value_t};

use crate::{State, errors::CliError, meta::Meta, arg_or_error, arge, NO_WORKERS, db::DB};


pub fn create_chunk(record: Meta, level: u32) -> String {

    let mut str_result = String::new();
    
    str_result.push_str(
        &format!("{}", record.id)
    );


    match level {
        1 => {

            if record.mutations.len() > 0 {

                str_result.push_str(
                    &format!(",1\n")
                );
            } else{

                str_result.push_str(
                    &format!(",0\n")
                );
            }

        }
        2 => {

            if record.mutations.len() > 0 {

                let first = record.mutations.get(0).unwrap();

                if record.mutations.len() > 1 {
                    log::warn!("More than one possible mutator, check this {}", record.mutations.len())
                }
                str_result.push_str(
                    &format!(",{},{},{},{},{},{},{},{},{},{}\n", 
                    record.num_tags,
                    record.function_count,
                    record.num_globals,
                    record.num_tables,
                    record.num_elements,
                    record.num_data,
                    record.num_tpes,
                    record.memory_count,
                    record.num_instructions,
                    first.generic_map.as_ref().unwrap().len())
                );
            } else{

                str_result.push_str(
                    &format!(",{},{},{},{},{},{},{},{},{},{}\n", 
                    record.num_tags,
                    record.function_count,
                    record.num_globals,
                    record.num_tables,
                    record.num_elements,
                    record.num_data,
                    record.num_tpes,
                    record.memory_count, 
                    record.num_instructions,0)
                );
            }
        }
        _ => {
            todo!("Level above 1 is not implemented yet")
        }
        
    }

    str_result.clone()
    
}

pub fn export(matches: &ArgMatches, args: &ArgMatches, dbclient: DB<'static>, state: Arc<State>) -> Result<(), CliError> {

    if args.is_present("list") {
        // TODO

    } else {
        log::debug!("Exporting {}", &arg_or_error!(matches, "dbconn"));

        // If JSON do this
        if args.is_present("csv") {

            
            let mut outfile = std::fs::File::create(arg_or_error!(args, "out")).unwrap();
            // Write headers

            let level = value_t!(args.value_of("level"), u32).unwrap();
            // Write header
            match level {
                1 => {

                    outfile.write_all(
                        "id,mutable_count\n".as_bytes()
                     ).unwrap();
                }
                2 => {

                    outfile.write_all(
                        "id,num_tags,num_functions,num_globals,num_tables,num_elements,num_data,num_types,num_memory,num_instructions,mutable_count\n".as_bytes()
                     ).unwrap();
                }
                _ => {

                    todo!("Level above 1 is not implemented yet")
                }
            }
            
            let mut c = 0;
            let mut buff = String::new();
            let TH = 1024*10;
            for item in dbclient.get_all()? {
                let r = create_chunk(item, level);
                
                buff.push_str(&r);
                c += 1;

                if c %100 == 99 {
                    log::debug!("Exported {} records", c);
                }

                if buff.len() > TH {
                    outfile.write_all(buff.as_bytes());
                    buff = String::new();
                }
            }

            outfile.write_all(buff.as_bytes());
            buff = String::new();

            println!("Done {} records", c);

            // Call workers and then append return to CSV file

        } else {



                let mut outfile = std::fs::File::create(arg_or_error!(args, "out")).unwrap();

                let mut all: Vec<Meta> = vec![];

                for record in dbclient.get_all()? {
                    all.push(record);
                }

                outfile
                    .write_all(serde_json::to_string_pretty(&all).unwrap().as_bytes())
                    .unwrap();
            }
        }
    Ok(())
}
