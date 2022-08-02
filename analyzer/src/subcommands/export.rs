use std::{collections::HashMap, io::Write, sync::{atomic::{AtomicU32, Ordering, AtomicBool}, Arc}, fs::File};

use clap::{ArgMatches, value_t};

use crate::{State, errors::CliError, meta::Meta, arg_or_error, arge, NO_WORKERS, db::DB};


pub fn create_chunk(records: Vec<Meta>, level: u32, counter: Arc<AtomicU32>, file_locker: Arc<AtomicBool>, file: &mut File) -> () {

    let mut str_result = String::new();

    for item in records {
        str_result.push_str(
            &format!("{}", item.id)
        );
    
    
        match level {
            1 => {
    
               if item.mutations.len() > 0 {
    
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
    
                str_result.push_str(
                    &format!(",{}", item.num_instructions)
                );
                if item.mutations.len() > 0 {
    
                    let first = item.mutations.get(0).unwrap();
    
                    if item.mutations.len() > 1 {
                        log::warn!("More than one possible mutator, check this {}", item.mutations.len())
                    }
                    str_result.push_str(
                        &format!("{},{},{},{},{},{},{},{},{}\n", 
                        item.num_tags,
                        item.function_count,
                        item.num_globals,
                        item.num_tables,
                        item.num_elements,
                        item.num_data,
                        item.num_tpes,
                        item.memory_count,
                        first.generic_map.as_ref().unwrap().len())
                    );
                } else{
    
                    str_result.push_str(
                        &format!("{},{},{},{},{},{},{},{},{}\n", 
                        item.num_tags,
                        item.function_count,
                        item.num_globals,
                        item.num_tables,
                        item.num_elements,
                        item.num_data,
                        item.num_tpes,
                        item.memory_count, 0)
                    );
                }
            }
            _ => {
                todo!("Level above 1 is not implemented yet")
            }
            
        }

        let c = counter.fetch_add(1, Ordering::SeqCst);
        if c % 99 == 0 {
            log::debug!("{}", c)
        }
    }
    

    log::debug!("Appending to file");
    let lock = file_locker.load(Ordering::Acquire);
    file.write_all(str_result.as_bytes());
    let lock = file_locker.store(true, Ordering::Release);
    
}

pub fn export(matches: &ArgMatches, args: &ArgMatches, dbclient: DB<'static>) -> Result<(), CliError> {

    if args.is_present("list") {
        // TODO

    } else {
        log::debug!("Exporting {}", &arg_or_error!(matches, "collection_name"));



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
            let mut workers = vec![vec![];NO_WORKERS];
            let records: Vec<Meta> = dbclient.get_all().unwrap();
            for (idx, record) in records.iter().enumerate() {
                let item = record;
                workers[idx%NO_WORKERS].push(item.clone());
            }

            let mut jobs = vec![];
            let counter = Arc::new(AtomicU32::new(0));
            let locker = Arc::new(AtomicBool::new(true));
            for i in 0..NO_WORKERS {
                
                let pc = workers[i].clone();
                let countercp = counter.clone();
                let mut outcp = outfile.try_clone()?;
                let lockercp = locker.clone();

                let th = std::thread::Builder::new()
                    .name(format!("exporter{}", i))
                    .stack_size(32 * 1024 * 1024 * 1024) // 320 MB
                    .spawn(move || {
                        create_chunk(pc, level, countercp, lockercp, &mut outcp);
                    }
                    )?;
            
                    jobs.push(th);
            }

            for j in jobs {
                let r = j.join().unwrap();
            }


            let c = counter.fetch_add(1, Ordering::SeqCst);
            println!("Done {} records", c);

            // Call workers and then append return to CSV file

        } else {



                let records: Vec<Meta> = dbclient.get_all().unwrap();
                let mut outfile = std::fs::File::create(arg_or_error!(args, "out")).unwrap();

                let mut all = vec![];

                for record in records {
                    all.push(record);
                }

                outfile
                    .write_all(serde_json::to_string_pretty(&all).unwrap().as_bytes())
                    .unwrap();
            }
        }
    Ok(())
}
