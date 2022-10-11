use rand::{
    distributions::WeightedIndex, prelude::Distribution, rngs::SmallRng, Rng,
    SeedableRng,
};
use std::{
    collections::HashSet,
    fs,
    io::{Read, Write},
    os::unix::net::UnixListener,
    path::Path,
    process::ExitStatus,
    sync::{
        Arc, Mutex,
    },
    thread::spawn,
};
use wasm_mutate::{
    WasmMutate,
};

use crate::{
    errors::{AResult, CliError},
    send_signal_to_probes_socket,
    subcommands::mutate::{mutation_factory::get_by_name, hasting::{get_acceptance_symmetric_prob, get_distance_reward, get_distance_reward_and_size, get_distance_reward_penalize_iteration}},
    State, SOCKET_PATH,
};
use std::thread;

use self::mutation_factory::{MutationFactory, MutationResult};
pub mod mutation_factory;
pub mod hasting;

// Patch for IPC from Peephole
lazy_static! {
    static ref original_peep: Mutex<String> = Mutex::new("".into());
    static ref replacement_peep: Mutex<String> = Mutex::new("".into());
}

#[derive(Debug)]
pub enum MODE {
    SEQUENTIAL,
    BISECT(u32, u32),
    REWARD {
        mutators_weights_name: &'static str,
        use_reward: bool,
    },
}

fn open_socket() -> AResult<String> {
    let outfile = format!("probes.logs.txt");
    let mut f = fs::File::create(outfile.clone())?;

    println!("Opening probes sockets in {}", SOCKET_PATH);

    log::debug!("Opening probes sockets in {}", SOCKET_PATH);

    let socket = Path::new(SOCKET_PATH);

    if socket.exists() {
        log::debug!("File exist");
        fs::remove_file(SOCKET_PATH)?;
        // fs::unlink(&socket)?
    }

    let stream = UnixListener::bind(&socket)?;

    println!("Socket waiting for probe messages");
    log::debug!("Socket waiting for probe messages");

    'always_listening: loop {
        for (mut stream, _) in stream.accept() {
            let mut buff = String::new();
            stream.read_to_string(&mut buff)?;

            // patch to check if this is a peephole

            let pref = "Original expression \n============\n";
            let pref2 = "New expression \n============\n";
            if buff.starts_with(pref) {
                //println!("Original expression found");
                let expr = &buff[pref.len()..];
                //println!("{:?}", expr);
                *original_peep.lock().unwrap() = expr.into();
            }


            if buff.starts_with(pref2) {
                //println!("Replacement expression found");
                let expr = &buff[pref2.len()..];
                //println!("{:?}", expr);
                *replacement_peep.lock().unwrap() = expr.into();
            }
            let splat = buff.split("][").collect::<Vec<_>>();

            match splat[0] {
                "STOP" => break 'always_listening,
                "SAVE" => {
                    f.write_all(&"New variant\n".as_bytes());
                    f.write_all(&splat[1].as_bytes());
                    f.write_all(&"\n".as_bytes());
                    f.flush()?;
                }
                _ => {
                    buff.push_str(&"\n");
                    f.write_all(&buff.as_bytes())?;
                    f.flush()?;
                }
            }
        }
    }

    Ok(outfile)
}

pub fn mutate_bisect(
    state: Arc<State>,
    path: String,
    command: String,
    args: Vec<String>,
    attemps: u32,
    peek_count: u32,
    seed: u64,
    tree_size: u32,
) -> AResult<()> {
    let mut dimensions = vec![
        (0, tree_size * 2, tree_size, "tree_size"),
        (0, attemps * 2, attemps, "attempts"),
        (0, peek_count * 2, peek_count, "peek"),
    ];
    let mut finished = vec![false, false, false];
    let mut dimindex = 0;
    loop {
        if finished.iter().all(|f| *f) {
            break;
        }
        // Iterate through dimensions until all finished
        loop {
            let mut dim = &dimensions[dimindex];
            println!("dim {:?}", dim);
            match dimindex {
                0 => {
                    let (mut low, mut high, mut last_success, _) = dim;
                    if high - low <= 1 {
                        finished[dimindex] = true;
                        break;
                    }
                    let tsize = (low + high) / 2;
                    log::debug!("Going mutate for tree_size {}", tsize);
                    let statecp = state.clone();
                    let pathcp = path.clone();
                    let commandcp = command.clone();
                    let argsclone = args.clone();
                    let lastattempts = dimensions[1].2;
                    let lastpeek = dimensions[2].2;

                    let (elapsed, interesting_count) = mutate_sequential(
                        statecp,
                        pathcp,
                        commandcp,
                        argsclone,
                        lastattempts,
                        true,
                        lastpeek as u64,
                        seed,
                        tsize,
                        1,
                    )?;

                    log::debug!(
                        "Elapsed {}, interesting count {}",
                        elapsed,
                        interesting_count
                    );
                    if interesting_count == 0 {
                        // Go lower
                        let dim = ((low + high) / 2, high, last_success, "tree");
                        dimensions[dimindex] = dim;
                    } else {
                        // Go higher
                        let dim = (low, (low + high) / 2, tsize, "tree");
                        dimensions[dimindex] = dim;
                    }
                }
                1 => {
                    let (mut low, mut high, mut last_success, _) = dim;
                    if high - low <= 1 {
                        finished[dimindex] = true;
                        break;
                    }
                    let attempts = (low + high) / 2;
                    log::debug!("Going mutate for attempt {}", attempts);
                    let statecp = state.clone();
                    let pathcp = path.clone();
                    let commandcp = command.clone();
                    let argsclone = args.clone();
                    let lastsize = dimensions[0].2;
                    let lastpeek = dimensions[2].2;

                    let (elapsed, interesting_count) = mutate_sequential(
                        statecp,
                        pathcp,
                        commandcp,
                        argsclone,
                        attempts,
                        true,
                        lastpeek as u64,
                        seed,
                        lastsize,
                        1,
                    )?;

                    log::debug!(
                        "Elapsed {}, interesting count {}",
                        elapsed,
                        interesting_count
                    );
                    if interesting_count == 0 {
                        // Go lower
                        let dim = ((low + high) / 2, high, last_success, "attempts");
                        dimensions[dimindex] = dim;
                    } else {
                        // Go higher
                        let dim = (low, (low + high) / 2, attempts, "attempts");
                        dimensions[dimindex] = dim;
                    }
                }
                2 => {
                    let (mut low, mut high, mut last_success, _) = dim;
                    if high - low <= 1 {
                        finished[dimindex] = true;
                        break;
                    }
                    let peeks = (low + high) / 2;
                    log::debug!("Going mutate for peeks {}", peeks);
                    let statecp = state.clone();
                    let pathcp = path.clone();
                    let commandcp = command.clone();
                    let argsclone = args.clone();
                    let lastsize = dimensions[0].2;
                    let lastattemps = dimensions[1].2;

                    let (elapsed, interesting_count) = mutate_sequential(
                        statecp,
                        pathcp,
                        commandcp,
                        argsclone,
                        lastattemps,
                        true,
                        peeks as u64,
                        seed,
                        lastsize,
                        1,
                    )?;

                    log::debug!(
                        "Elapsed {}, interesting count {}",
                        elapsed,
                        interesting_count
                    );
                    if interesting_count == 0 {
                        // Go lower
                        let dim = ((low + high) / 2, high, last_success, "peek");
                        dimensions[dimindex] = dim;
                    } else {
                        // Go higher
                        let dim = (low, (low + high) / 2, peeks, "peek");
                        dimensions[dimindex] = dim;
                    }
                }
                _ => {
                    panic!("Invalid dimension")
                }
            }
        }
        dimindex = (dimindex + 1) % dimensions.len();
    }
    println!("Minimum config {:?}", dimensions);
    Ok(())
}

pub fn mutate_sequential(
    state: Arc<State>,
    path: String,
    command: String,
    args: Vec<String>,
    attemps: u32,
    exit_on_found: bool,
    peek_count: u64,
    seed: u64,
    tree_size: u32,
    bulk_limit: usize,
) -> AResult<(u32, u32)> {
    log::debug!("Mutating binary {}", path);
    let th = spawn(move || open_socket());
    loop {
        if Path::new("probes.logs.txt").exists() {
            log::debug!("Probes file exists. Waiting for setle down 30s");
            thread::sleep(std::time::Duration::from_secs(30));
            break;
        }
    }

    let mut file = fs::File::open(path.clone())?;
    let session_folder = format!(
        "{}/{}_{}_a{}_p{}_ts{}",
        state.dbclient.as_ref().unwrap().f,
        command.replace("/", "_"),
        args.iter()
            .map(|f| f.replace("/", "_"))
            .collect::<Vec<_>>()
            .join("_"),
        attemps,
        peek_count,
        tree_size
    );
    fs::create_dir(session_folder.clone());
    log::debug!("Saving session in {}", session_folder.clone());

    // Filter first the header to check for Wasm
    let mut buf = [0; 4];
    let r = &file.read_exact(&mut buf)?;

    let mut bin = match &buf {
        // Filter first the header to check for Wasm
        b"\0asm" => fs::read(path.clone())?,
        _ => return Err(CliError::Any("Invalid Wasm header".into())),
    };

    let mut elapsed = 0;
    let mut gn = SmallRng::seed_from_u64(seed);

    let mut seen: HashSet<blake3::Hash> = HashSet::new();
    let mut collision_count = 0;
    let mut interesting_count = 0;
    let mut parent = String::new();
    let mut buffer = vec![];

    'attempts: while true {
        // mutated = m
        let s = gn.gen();
        let mut config = WasmMutate::default();
        config.preserve_semantics(true);
        config.peephole_size(tree_size);
        config.seed(s);

        let cp = bin.clone();

        let m = config.run(&cp);

        let mut worklist = vec![];
        match m {
            Err(e) => {
                log::error!("{}", e)
            }
            Ok(it) => {
                for (idx, b) in it.enumerate().take(peek_count as usize) {
                    match b {
                        Err(e) => {
                            log::error!("{}", e)
                        }
                        Ok(b) => {
                            // FIXME, Prevent to save a previous seen binary
                            // TODO, validate as well
                            let hash = blake3::hash(&b.clone());

                            if !seen.contains(&hash) {
                                worklist.push((b.clone(), idx));
                                seen.insert(hash);
                            } else {
                                log::debug!("Binary already seen");
                                collision_count += 1;
                            }
                        }
                    }
                }
            }
        }
        // log::debug!("Worklist size {}", worklist.len());
        if worklist.len() == 0 {
            // Reset the probes file
            elapsed += 1;
            send_signal_to_probes_socket(format!("No mutation"));
            continue;
        }

        while let Some((newbin, idx)) = worklist.pop() {
            // TODO Move this to parallel execution
            // TODO move this to bulk execution
            buffer.push((newbin.clone(), idx, s));
            log::debug!("Size of bulk {}", buffer.len());
            swap(&mut bin, newbin.clone());
            if buffer.len() >= bulk_limit {
                let results = check_binary(
                    buffer.clone(),
                    command.clone(),
                    args.clone(),
                    bulk_limit > 1,
                    true,
                )?;

                log::debug!("Size of results {}", results.len());
                for ((newbin, idx, s), result) in buffer.iter().zip(results) {
                    let (r, stdout, stderr) = result;
                    let (interesting, out) = if r.success() {
                        let fname = format!("{session_folder}/non_interesting");
                        fs::create_dir(fname.clone());
                        (false, fname)
                    } else {
                        interesting_count += 1;
                        let fname = format!("{session_folder}/interesting");
                        fs::create_dir(fname.clone());
                        (true, fname)
                    };

                    let fname = format!("{out}/e{:0width$}_s{}_i{}", elapsed, s, idx, width = 10);
                    fs::create_dir(fname.clone());
                    fs::write(format!("{}/stderr.txt", fname.clone()), &stderr)?;

                    let mut f = fs::File::create(format!("{}/iteration_info.txt", fname.clone()))?;
                    f.write_all(format!("seed: {}\n", s).as_bytes())?;
                    f.write_all(format!("attempts: {}\n", attemps).as_bytes())?;
                    f.write_all(format!("elapsed: {}\n", elapsed).as_bytes())?;
                    f.write_all(format!("idx: {}\n", idx).as_bytes())?;
                    f.write_all(format!("interesting: {}\n", interesting).as_bytes())?;
                    f.write_all(format!("variant_size: {}\n", newbin.len()).as_bytes())?;
                    f.write_all(format!("parent: {}\n", parent).as_bytes())?;
                    // TODO Add Meta info of the variant ?

                    fs::write(format!("{}/stdout.txt", fname.clone()), &stdout)?;
                    fs::write(format!("{}/variant.wasm", fname), &newbin)?;

                    send_signal_to_probes_socket(format!("SAVE][{}/probes.logs.txt", fname));
                    // Send filena name
                    parent = fname;

                    if exit_on_found && interesting {
                        elapsed += 1;
                        break 'attempts;
                    }

                    elapsed += 1;

                    if elapsed >= attemps as usize {
                        break 'attempts;
                    }

                    if elapsed % 10 == 9 {
                        println!(
                            "Elapsed {}/{}. Collision count {}. Interesting count {}",
                            elapsed, attemps, collision_count, interesting_count
                        );
                    }
                }

                buffer.clear();
            }
        }
    }

    println!(
        "Elapsed {}/{}. Collision count {}. Interesting count {}",
        elapsed, attemps, collision_count, interesting_count
    );

    // Now save the session to a folder ?
    send_signal_to_probes_socket("STOP".into());
    let outfile = th.join().unwrap()?;
    Ok((elapsed as u32, interesting_count))
}

pub fn get_random_mutators(
    config: &mut WasmMutate,
    rng: &mut SmallRng,
    prob_weights: impl MutationFactory,
    uniform: bool,
) -> AResult<MutationResult> {
    let cn = prob_weights.clone();
    let mutations = prob_weights.get_available_mutations();
    if uniform {
        let name = &mutations[rng.gen_range(0..mutations.len())];
        println!("{:?}", name);
        let mt = cn.get_mutators_by_feature(config, name, 1)?;
        return Ok(mt);
    } else {
        let dist2 = WeightedIndex::new(mutations.iter().map(|item| item.3)).unwrap();
        let name = &mutations[dist2.sample(rng)];
        println!("{:?}", name);
        let mt = cn.get_mutators_by_feature(config, name, 1)?;

        return Ok(mt);
    }
}

pub fn mutate_with_reward(
    state: Arc<State>,
    path: String,
    command: String,
    args: Vec<String>,
    attemps: u32,
    exit_on_found: bool,
    peek_count: u64,
    seed: u64,
    tree_size: u32,
    prob_weights_name: &'static str,
    use_reward: bool,
) -> AResult<(u32, u32)> {
    log::debug!("Mutating binary {}", path);
    let prob_weights = get_by_name(prob_weights_name);

    let th = spawn(move || open_socket());
    loop {
        if Path::new("probes.logs.txt").exists() {
            log::debug!("Probes file exists. Waiting for setle down 30s");
            thread::sleep(std::time::Duration::from_secs(5));
            break;
        }
    }

    println!("Starting mutate reward");
    let mut file = fs::File::open(path.clone())?;
    let session_folder = format!(
        "{}/{}_{}_a{}_p{}_ts{}",
        state.dbclient.as_ref().unwrap().f,
        command.replace("/", "_"),
        args.iter()
            .map(|f| f.replace("/", "_"))
            .collect::<Vec<_>>()
            .join("_"),
        attemps,
        peek_count,
        tree_size
    );
    fs::create_dir(session_folder.clone());
    log::debug!("Saving session in {}", session_folder.clone());

    // Filter first the header to check for Wasm
    let mut buf = [0; 4];
    let r = &file.read_exact(&mut buf)?;

    let mut original = match &buf {
        // Filter first the header to check for Wasm
        b"\0asm" => fs::read(path.clone())?,
        _ => return Err(CliError::Any("Invalid Wasm header".into())),
    };

    let mut elapsed = 0;
    let mut gn = SmallRng::seed_from_u64(seed);
    let mut gn2 = SmallRng::seed_from_u64(gn.gen());
    let mut rng = SmallRng::from_seed(gn.gen());

    let mut interesting_count = 0;
    let mut parent = String::new();
    let mut buffer = vec![];
    let mut missed: u32 = 0;
    let mut all: u32 = 0;
    let mut bin = original.clone();
    let mut operations: Vec<(&str, &str, &str)> = vec![];
    let mut reward = 0;
    let mut original_reward = 0;
    let mut number_of_oracle_calls = 0;

    if use_reward {
        // get the original reward by calling the oracle
        number_of_oracle_calls += 1;
        let results = check_binary(
            vec![(original.clone(), 0, seed)],
            command.clone(),
            args.clone(),
            false,
            true,
        )?;

        for result in results {
            let (_, _, stderr) = result;
            //let parsed = stderr::parse::<i32>();
            let stderrstr = String::from_utf8(stderr).unwrap();
            println!("!sterr '{stderrstr}'");
            let parsed: i32 = stderrstr.parse().expect("not a number");
            original_reward = parsed;
            reward = original_reward;
        }
    }

    let mut mutationlogfile = fs::File::create(format!("{session_folder}/mutation_log.txt"))?;

    'attempts: loop {
        if elapsed >= attemps {
            println!("Elapsed {}", elapsed);
            break;
        }
        // mutated = m
        let s = gn.gen();
        let mut config = WasmMutate::default();
        config.preserve_semantics(true);
        config.peephole_size(tree_size);
        config.seed(s);

        let cp = bin.clone();
        config.setup(&cp).unwrap();
        // Get random mutators here
        let (mutated, mutator_tpe, mutator_name, mutator_param, mutated_bin) =
            get_random_mutators(&mut config, &mut gn2, prob_weights.clone(), use_reward)?;
        all += 1;

        if mutated {
            println!(
                "{} Mutated with {}:{}:{}",
                number_of_oracle_calls, mutator_tpe, mutator_name, mutator_param
            );

            let hash = blake3::hash(&mutated_bin.clone());
            // Check if this is by reward, otherwise just, random mutate based on a weight map
            if use_reward {
                number_of_oracle_calls += 1;
                println!("Calling oracle");
                let results = check_binary(
                    vec![(mutated_bin.clone(), 0, s)],
                    command.clone(),
                    args.clone(),
                    false,
                    true,
                )?;

                println!("Oracle returns");

                for result in results {
                    let (r, stdout, stderr) = result;
                    // The reward in these cases come from the stderr

                    let stderrstr = String::from_utf8(stderr.clone());

                    let newr = if let Ok(str) = stderrstr {
                        if let Ok(parsed) = str.parse() {
                            parsed
                        } else {
                            // Not correctly parsed
                            -2
                        }
                    } else {
                        // Incorrect stderr
                        -1
                    };
                    // if the exit code is zero, then report, since it is a zero reward, which means...breaking
                    if r.success() {
                        let fname = format!("{session_folder}/interesting");
                        fs::create_dir(fname.clone());

                        let fname =
                            format!("{fname}/e{:0width$}_s{}_i{}", elapsed, s, 0, width = 10);
                        fs::create_dir(fname.clone());
                        fs::write(format!("{}/stderr.txt", fname.clone()), &stderr)?;

                        let mut f =
                            fs::File::create(format!("{}/iteration_info.txt", fname.clone()))?;
                        f.write_all(format!("seed: {}\n", s).as_bytes())?;
                        f.write_all(format!("attempts: {}\n", attemps).as_bytes())?;
                        f.write_all(format!("elapsed: {}\n", elapsed).as_bytes())?;
                        f.write_all(format!("interesting: {}\n", true).as_bytes())?;
                        //f.write_all(format!("variant_size: {}\n", mutated_bin.len()).as_bytes())?;
                        f.write_all(format!("parent: {}\n", parent).as_bytes())?;
                        f.write_all(
                            format!(
                                "mutation: {}|{}|{}\n",
                                mutator_tpe, mutator_name, mutator_param
                            )
                            .as_bytes(),
                        )?;

                        fs::write(format!("{}/stdout.txt", fname.clone()), &stdout)?;
                        fs::write(format!("{}/variant.wasm", fname), &mutated_bin)?;

                        send_signal_to_probes_socket(format!("SAVE][{}/probes.logs.txt", fname));

                        mutationlogfile.write(
                            format!(
                                "{}|{}|{}|{}| {}:{}:{} {}|{}|{}|{:?}|{:?}\n",
                                all,
                                number_of_oracle_calls,
                                newr,
                                0,
                                mutator_tpe,
                                mutator_name,
                                mutator_param,
                                "",
                                hash,
                                mutated_bin.len(),
                                original_peep.lock().unwrap(),
                                replacement_peep.lock().unwrap()
                            )
                            .as_bytes(),
                        )?;

                        *original_peep.lock().unwrap() = "".into();
                        *replacement_peep.lock().unwrap() = "".into();

                        println!("stderr {:?}", stderr);
                        println!("BINGO!");
                        break 'attempts;
                    } else {
                        let fname = format!("{session_folder}/non_interesting");
                        fs::create_dir(fname.clone());

                        let fname =
                            format!("{fname}/e{:0width$}_s{}_i{}", elapsed, s, 0, width = 10);
                        fs::create_dir(fname.clone());
                        fs::write(format!("{}/stderr.txt", fname.clone()), &stderr)?;

                        let mut f =
                            fs::File::create(format!("{}/iteration_info.txt", fname.clone()))?;
                        f.write_all(format!("seed: {}\n", s).as_bytes())?;
                        f.write_all(format!("attempts: {}\n", attemps).as_bytes())?;
                        f.write_all(format!("elapsed: {}\n", elapsed).as_bytes())?;
                        f.write_all(format!("interesting: {}\n", true).as_bytes())?;
                        //f.write_all(format!("variant_size: {}\n", mutated_bin.len()).as_bytes())?;
                        f.write_all(format!("parent: {}\n", parent).as_bytes())?;
                        f.write_all(
                            format!(
                                "mutation: {}|{}|{}\n",
                                mutator_tpe, mutator_name, mutator_param
                            )
                            .as_bytes(),
                        )?;

                        fs::write(format!("{}/stdout.txt", fname.clone()), &stdout)?;
                        fs::write(format!("{}/variant.wasm", fname), &mutated_bin)?;

                        send_signal_to_probes_socket(format!("SAVE][{}/probes.logs.txt", fname));

                        // Do completely random
                        let rn: f32 = rng.gen();

                        println!("New reward {newr}");

                        let origclone = original.clone();
                        let opsclone = operations.clone();
                        let mutatedcp = mutated_bin.clone();
                        let binclone = bin.clone();
                        let probs1clone = prob_weights.clone();

                        let (cost1, cost2, beta) = get_acceptance_symmetric_prob(
                            (origclone, vec![], original_reward),
                            (binclone, opsclone.clone(), reward),
                            (
                                mutatedcp,
                                vec![opsclone, vec![(mutator_tpe, mutator_name, mutator_param)]]
                                    .concat(),
                                newr,
                            ),
                            probs1clone,
                            Box::new(get_distance_reward_penalize_iteration)
                        );

                        let lg = rn.log(2.7) / beta;
                        let reduction = if mutated_bin.len() < bin.len() {
                            "reduction"
                        } else {
                            ""
                        };

                        //println!("Hash {}", hash);
                        if cost2 < cost1 + lg {
                            println!("Accepting with {} < {} + ({}) ", cost2, cost1, lg);

                            println!(
                                "{}|{}|{}|{}| {}:{}:{} {}|{}|{}|{:?}|{:?}\n",
                                all,
                                number_of_oracle_calls,
                                newr,
                                lg,
                                mutator_tpe,
                                mutator_name,
                                mutator_param,
                                reduction,
                                hash,
                                mutated_bin.len(),
                                original_peep.lock().unwrap(),
                                replacement_peep.lock().unwrap()
                            );
                            mutationlogfile.write(
                                format!(
                                    "{}|{}|{}|{}| {}:{}:{} {}|{}|{}|{:?}|{:?}\n",
                                    all,
                                    number_of_oracle_calls,
                                    newr,
                                    lg,
                                    mutator_tpe,
                                    mutator_name,
                                    mutator_param,
                                    reduction,
                                    hash,
                                    mutated_bin.len(),
                                    original_peep.lock().unwrap(),
                                    replacement_peep.lock().unwrap()
                                )
                                .as_bytes(),
                            )?;

                            *original_peep.lock().unwrap() = "".into();
                            *replacement_peep.lock().unwrap() = "".into();

                            swap(&mut bin, mutated_bin.clone());
                            operations.push((mutator_tpe, mutator_name, mutator_param));
                            reward = newr;
                            // TODO update reward
                            // Add the mutation to the current list
                        } else {
                            println!("Rejecting with {} < {} + ({}) ", cost2, cost1, lg);

                            println!(
                                "{}|{}|{}|{}| {}:{}:{} {}|{}|{}|{:?}|{:?}| but not moved\n",
                                all,
                                number_of_oracle_calls,
                                newr,
                                lg,
                                mutator_tpe,
                                mutator_name,
                                mutator_param,
                                reduction,
                                hash,
                                mutated_bin.len(),
                                original_peep.lock().unwrap(),
                                replacement_peep.lock().unwrap()
                            );
                            mutationlogfile.write(
                                format!(
                                    "{}|{}|{}|{}| {}:{}:{} {}|{}|{}|{:?}|{:?}| but not moved\n",
                                    all,
                                    number_of_oracle_calls,
                                    newr,
                                    lg,
                                    mutator_tpe,
                                    mutator_name,
                                    mutator_param,
                                    reduction,
                                    hash,
                                    mutated_bin.len(),
                                    original_peep.lock().unwrap(),
                                    replacement_peep.lock().unwrap()
                                )
                                .as_bytes(),
                            )?;

                            *original_peep.lock().unwrap() = "".into();
                            *replacement_peep.lock().unwrap() = "".into();
                            //println!("Rejecting with acceptance {} > {}", rn, lg);
                        }
                    };
                }
            } else {
                mutationlogfile.write(
                    format!(
                        "{}|{}| {}:{}:{}\n",
                        all, number_of_oracle_calls, mutator_tpe, mutator_name, mutator_param
                    )
                    .as_bytes(),
                )?;
                // Check the oracle as usual
                // The prob of mutating to the new binary is always 1
                buffer.push((mutated_bin.clone(), 0, s));
                // Always swap and advance
                swap(&mut bin, mutated_bin.clone());
                number_of_oracle_calls += 1;
                let results =
                    check_binary(buffer.clone(), command.clone(), args.clone(), false, true)?;
                log::debug!("Size of results {}", results.len());
                for ((newbin, idx, s), result) in buffer.iter().zip(results) {
                    let (r, stdout, stderr) = result;
                    println!("result reward {:?}", r);
                    println!("result stdout {:?}", String::from_utf8(stdout.clone()));
                    println!("result stderr {:?}", String::from_utf8(stderr.clone()));

                    let (interesting, out) = if r.success() {
                        let fname = format!("{session_folder}/interesting");
                        fs::create_dir(fname.clone());
                        (true, fname)
                    } else {
                        interesting_count += 1;
                        let fname = format!("{session_folder}/non_interesting");
                        fs::create_dir(fname.clone());
                        (false, fname)
                    };

                    let fname = format!("{out}/e{:0width$}_s{}_i{}", elapsed, s, idx, width = 10);
                    fs::create_dir(fname.clone());
                    fs::write(format!("{}/stderr.txt", fname.clone()), &stderr)?;

                    let mut f = fs::File::create(format!("{}/iteration_info.txt", fname.clone()))?;
                    f.write_all(format!("seed: {}\n", s).as_bytes())?;
                    f.write_all(format!("attempts: {}\n", attemps).as_bytes())?;
                    f.write_all(format!("elapsed: {}\n", elapsed).as_bytes())?;
                    f.write_all(format!("idx: {}\n", idx).as_bytes())?;
                    f.write_all(format!("interesting: {}\n", interesting).as_bytes())?;
                    f.write_all(format!("variant_size: {}\n", newbin.len()).as_bytes())?;
                    f.write_all(format!("parent: {}\n", parent).as_bytes())?;
                    f.write_all(
                        format!(
                            "mutation: {}|{}|{}\n",
                            mutator_tpe, mutator_name, mutator_param
                        )
                        .as_bytes(),
                    )?;
                    // TODO Add Meta info of the variant ?

                    // Mutation applied

                    fs::write(format!("{}/stdout.txt", fname.clone()), &stdout)?;
                    fs::write(format!("{}/variant.wasm", fname), &newbin)?;

                    send_signal_to_probes_socket(format!("SAVE][{}/probes.logs.txt", fname));
                    // Send filena name
                    parent = fname;

                    if exit_on_found && interesting {
                        elapsed += 1;
                        break 'attempts;
                    }

                    elapsed += 1;

                    if elapsed >= attemps {
                        break 'attempts;
                    }
                }

                buffer.clear();
            }
        } else {
            missed += 1;
            if missed % 100 == 99 {
                println!(
                    "Missed {}/{} ({:.2}%) times.",
                    missed,
                    all,
                    100.0 * (missed as f32) / (all as f32)
                );
            }
        }

        elapsed += 1;
    }

    send_signal_to_probes_socket("STOP".into());
    Ok((elapsed as u32, interesting_count))
}

pub fn mutate(
    state: Arc<State>,
    path: String,
    command: String,
    args: Vec<String>,
    attemps: u32,
    exit_on_found: bool,
    peek_count: u64,
    seed: u64,
    tree_size: u32,
    mode: MODE,
    bulk_limit: usize,
) -> AResult<()> {
    match mode {
        MODE::SEQUENTIAL => {
            mutate_sequential(
                state,
                path,
                command,
                args,
                attemps,
                exit_on_found,
                peek_count,
                seed,
                tree_size,
                bulk_limit,
            )?;
        }
        MODE::BISECT(_, _) => {
            mutate_bisect(
                state,
                path,
                command,
                args,
                attemps,
                peek_count as u32,
                seed,
                tree_size,
            )?;
        }
        MODE::REWARD {
            mutators_weights_name,
            use_reward,
        } => {
            mutate_with_reward(
                state,
                path,
                command,
                args,
                attemps,
                true,
                1,
                seed,
                tree_size,
                mutators_weights_name,
                use_reward,
            )?;
        }
    };

    Ok(())
}

/// Return bulk calling of the oracle each one with the binary passed
fn check_binary(
    bins: Vec<(Vec<u8>, usize, u64)>,
    command: String,
    args: Vec<String>,
    bulk: bool,
    return_as_it_it: bool,
) -> AResult<Vec<(ExitStatus, Vec<u8>, Vec<u8>)>> {
    // Write file to tmparg

    let mut r = vec![];
    let mut results = vec![];
    for (i, (bin, _, _)) in bins.iter().enumerate() {
        let fname = format!("tmparg{}.wasm", i);
        fs::write(fname.clone(), bin)?;

        if !bulk {
            let output = std::process::Command::new(&command)
                .args(args.clone())
                .arg(fname)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
                .map_err(|e| {
                    CliError::Any(format!(
                        "Failed to run command {} args {:?}. Error {}",
                        command, args, e
                    ))
                })?;

            results.push((output.status, output.stdout, output.stderr));

            if return_as_it_it || !output.status.success() {
                return Ok(results);
            }
        } else {
            r.push(fname);
        }
    }

    if bulk {
        let output = std::process::Command::new(&command)
            .args(args.clone())
            .args(r)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .map_err(|e| {
                CliError::Any(format!(
                    "Failed to run command {} args {:?}. Error {}",
                    command, args, e
                ))
            })?;

        let cp = output.clone();
        let mp = bins
            .iter()
            .map(move |_| (output.status, cp.stdout.clone(), cp.stderr.clone()))
            .collect::<Vec<_>>();
        // Copy the result and set to all

        return Ok(mp);
    }

    // Return all ok
    return Ok(results);
}

fn swap(a: &mut Vec<u8>, b: Vec<u8>) {
    *a = b;
}

#[cfg(test)]
pub mod tests {
    use std::sync::{
        atomic::{AtomicBool, AtomicU32},
        Arc,
    };

    use crate::{db::DB, State};

    use super::{
        mutate,
        // mutation_factory::{get_by_name, MutationFactory},
        MODE,
    };

    #[test]
    pub fn test1() {
        let state = State {
            dbclient: Some(DB::new("test_db/t1", 10000).unwrap()),
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            out_folder: None,
            save_logs: false,
            finish: AtomicBool::new(false),
            depth: 2,
            sample_ratio: 1,
            patch_metadata: false,
            seed: 0,
            timeout: 5,
            snapshot: None,
            snapshot_time: None,
        };

        mutate(
            Arc::new(state),
            "tests/1.wasm".into(),
            "/bin/bash".into(),
            vec!["tests/oracle_size.sh".into()],
            5,
            false,
            1,
            0,
            100,
            MODE::SEQUENTIAL,
            3,
        )
        .unwrap()
    }

    #[test]
    pub fn test2() {
        let state = State {
            dbclient: Some(DB::new("test_db/t2", 10000).unwrap()),
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            out_folder: None,
            save_logs: false,
            finish: AtomicBool::new(false),
            depth: 2,
            sample_ratio: 1,
            patch_metadata: false,
            seed: 0,
            timeout: 5,
            snapshot: None,
            snapshot_time: None,
        };

        mutate(
            Arc::new(state),
            "tests/1.wasm".into(),
            "/bin/bash".into(),
            vec!["tests/oracle_size.sh".into()],
            7,
            true,
            1,
            0,
            100,
            MODE::SEQUENTIAL,
            3,
        )
        .unwrap()
    }

    #[test]
    pub fn test3() {
        let state = State {
            dbclient: Some(DB::new("test_db/t3", 10000).unwrap()),
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            out_folder: None,
            save_logs: false,
            finish: AtomicBool::new(false),
            depth: 2,
            sample_ratio: 1,
            patch_metadata: false,
            seed: 0,
            timeout: 5,
            snapshot: None,
            snapshot_time: None,
        };

        mutate(
            Arc::new(state),
            "tests/1.wasm".into(),
            "/bin/bash".into(),
            vec!["tests/oracle_size.sh".into()],
            7,
            true,
            1,
            0,
            100,
            MODE::SEQUENTIAL,
            1,
        )
        .unwrap()
    }

    #[test]
    pub fn test_feature_by_str1() {
        let mut dbclient = DB::new("test_db/tfeatures", 10000).unwrap();
        dbclient.open().unwrap();

        let state = State {
            dbclient: Some(dbclient),
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            out_folder: None,
            save_logs: false,
            finish: AtomicBool::new(false),
            depth: 2,
            sample_ratio: 1,
            patch_metadata: false,
            seed: 0,
            timeout: 5,
            snapshot: None,
            snapshot_time: None,
        };

        mutate(
            Arc::new(state),
            "tests/1.wasm".into(),
            "/bin/bash".into(),
            vec!["tests/oracle_number_of_types.sh".into()],
            1000,
            true,
            1,
            0,
            100,
            MODE::REWARD {
                mutators_weights_name: "Uniform",
                use_reward: false,
            },
            1,
        )
        .unwrap()
    }

    #[test]
    pub fn test_feature_by_str_oracle() {
        let mut dbclient = DB::new("test_db/tfeatures5", 10000).unwrap();
        dbclient.open().unwrap();

        let state = State {
            dbclient: Some(dbclient),
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            out_folder: None,
            save_logs: false,
            finish: AtomicBool::new(false),
            depth: 2,
            sample_ratio: 1,
            patch_metadata: false,
            seed: 1,
            timeout: 5,
            snapshot: None,
            snapshot_time: None,
        };

        mutate(
            Arc::new(state),
            "tests/47d299593572faf8941351f3ef8e46bc18eb684f679d87f9194bb635dd8aabc0.wasm".into(),
            "python3".into(),
            vec![
                "../../oracles/vt_custom_chrome/vt_oracle_count_reward.py".into(),
                "http://0.0.0.0:5600/".into(),
                "test".into(),
                "admin".into(),
                "admin".into(),
                "test1".into(),
            ],
            1000,
            true,
            1,
            0,
            100,
            MODE::REWARD {
                mutators_weights_name: "Uniform",
                use_reward: true,
            },
            1,
        )
        .unwrap()
    }

    #[test]
    pub fn test_feature_by_str4() {
        let mut dbclient = DB::new("test_db/tfeatures4", 10000).unwrap();
        dbclient.open().unwrap();

        let state = State {
            dbclient: Some(dbclient),
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            out_folder: None,
            save_logs: false,
            finish: AtomicBool::new(false),
            depth: 2,
            sample_ratio: 1,
            patch_metadata: false,
            seed: 1,
            timeout: 5,
            snapshot: None,
            snapshot_time: None,
        };

        mutate(
            Arc::new(state),
            "tests/jazecminer.wasm".into(),
            "/bin/bash".into(),
            vec!["tests/oracle_number_of_types_and_funcs.sh".into()],
            5000,
            true,
            1,
            0,
            100,
            MODE::REWARD {
                mutators_weights_name: "Uniform",
                use_reward: true,
            },
            1,
        )
        .unwrap()
    }

    #[test]
    pub fn test_feature_by_str2() {
        let mut dbclient = DB::new("test_db/tfeatures2", 10000).unwrap();
        dbclient.open().unwrap();

        let state = State {
            dbclient: Some(dbclient),
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            out_folder: None,
            save_logs: false,
            finish: AtomicBool::new(false),
            depth: 2,
            sample_ratio: 1,
            patch_metadata: false,
            seed: 0,
            timeout: 5,
            snapshot: None,
            snapshot_time: None,
        };

        mutate(
            Arc::new(state),
            "tests/1.wasm".into(),
            "/bin/bash".into(),
            vec!["tests/oracle_number_of_types.sh".into()],
            5000,
            true,
            1,
            0,
            100,
            MODE::REWARD {
                mutators_weights_name: "Uniform",
                use_reward: true,
            },
            1,
        )
        .unwrap()
    }

    #[test]
    pub fn test_feature_by_str3() {
        let mut dbclient = DB::new("test_db/tfeatures3", 10000).unwrap();
        dbclient.open().unwrap();

        let state = State {
            dbclient: Some(dbclient),
            process: AtomicU32::new(0),
            error: AtomicU32::new(0),
            parsing_error: AtomicU32::new(0),
            out_folder: None,
            save_logs: false,
            finish: AtomicBool::new(false),
            depth: 2,
            sample_ratio: 1,
            patch_metadata: false,
            seed: 0,
            timeout: 5,
            snapshot: None,
            snapshot_time: None,
        };

        mutate(
            Arc::new(state),
            "tests/1.wasm".into(),
            "/bin/bash".into(),
            vec!["tests/oracle_number_of_types.sh".into()],
            4000,
            true,
            1,
            0,
            100,
            MODE::REWARD {
                mutators_weights_name: "Uniform",
                use_reward: false,
            },
            1,
        )
        .unwrap()
    }
}
