use super::mutation_factory::MutationFactory;


type AcceptanceWasm = Vec<u8>;
// Wasm, operations to reach the Wasm, reward
type AcceptanceTuple = (
    AcceptanceWasm,
    Vec<(Vec<&'static str>, Vec<&'static str>, Vec<&'static str>)>,
    i32,
    u32 // attempt
);

pub enum MutatorAppliance {
    Direct(&'static str),
    Reverse(&'static str),
}

/// Returns the prob of that given wasm1 and wasm2, that wasm2 can be reached with mutation
pub fn qprob(
    _wasm: AcceptanceWasm,
    _next_wasm: AcceptanceWasm,
    appliance: MutatorAppliance,
    prob_weights: Vec<(String, f64)>,
) -> f32 {

    match appliance {
        // TODO, read from the prob_weights
        // TODO, read from history, keep track of the successfully applied mutators
        MutatorAppliance::Direct(name) => {
            for (f, w) in prob_weights {
                if f == name {
                    return w as f32;
                }
            }
            return 1.0;
        }
        MutatorAppliance::Reverse(_) => 0.1, // there is no reverse, however, we need to ensure ergodicity
    }
}

// TODO, make this a DSL "1 + 0.1*|f|"
// TODO, add here more options, difference in size, AST diff, etc

/// Returns the cost of the binary by adding the reward and the increate in the size of the binary
/// The formula is "1 - 3.0*delta(size) + 10.0*delta(reward)". It penalizes the increase in the size of the new binary
pub fn get_distance_reward_and_size(seed: AcceptanceTuple, wasm: AcceptanceTuple) -> f32 {
    let scale = 10.0;
    return (1.0 - 3.0 * (wasm.0.len().overflowing_sub(seed.0.len()).0 as f32)
        + scale * (wasm.2.overflowing_sub(seed.2)).0 as f32) as f32; // reward with size panlization
}

/// Returns the cost of the binary by taking into account only the reward
/// The formula is "1 + 10.0*delta(reward)". It penalizes the increase in the size of the new binary
pub fn get_distance_reward(seed: AcceptanceTuple, wasm: AcceptanceTuple) -> f32 {
    let scale = 5.0;
    return (0.0 + scale*(wasm.2.overflowing_sub(seed.2)).0 as f32) as f32 // only reward
}

/// Returns the cost of the binary by taking into account only the reward
/// The formula is "1 + 10.0*delta(reward)". It penalizes the increase in the size of the new binary
pub fn get_distance_reward_penalize_iteration(seed: AcceptanceTuple, wasm: AcceptanceTuple) -> f32 {
    let scale = 5.0;
    return (0.0 + scale*(wasm.2.overflowing_sub(seed.2)).0 as f32)/(0.5*wasm.1.len()  as f32 + 1.0) as f32 // only reward
}

/// Returns the cost of the binary by taking into account only the reward
/// The formula is "1 + 10.0*delta(reward)". It penalizes the increase in the size of the new binary
pub fn get_distance_reward_penalize_attempt(seed: AcceptanceTuple, wasm: AcceptanceTuple) -> f32 {
    let scale = 5.0;
    return (0.0 + scale*(wasm.2.overflowing_sub(seed.2)).0 as f32)/(0.3*wasm.3  as f32 + 1.0) as f32 // only reward
}

/// Assumes that the probs of getting one mutator is always the same including its reverse
pub fn get_acceptance_symmetric_prob(
    original: AcceptanceTuple,
    prev: AcceptanceTuple,
    curr: AcceptanceTuple,
    _prob_weights: impl MutationFactory,
    mut cost_func: Box<dyn FnMut(AcceptanceTuple, AcceptanceTuple) -> f32>
) -> (f32, f32) {
    // get last operation of curr
    let (_, operations, _, _) = curr.clone();
    let (_, _, _, _) = prev.clone();
    let (_, _, _, _) = original;
    // Last operation
    let _ = operations.last().unwrap();

    let cost1 = cost_func(original.clone(), curr.clone());
    let cost2 = cost_func(original, prev.clone());

    return (cost1, cost2);
}
