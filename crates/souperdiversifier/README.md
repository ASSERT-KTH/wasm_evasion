# MVP Souperify pass for Wasm binaries for superoptimization and superdiversification

## Requirements

Copy the [souper](TODO_oursouper_version) build folder into the `libs/souper` folder

## How to run the superoptimizer

```
cargo build --release
target/release/souperdiversifier --superoptimize input.wasm -o out.wasm
```

## How to run the superdiversifier

```
cargo build --release
target/release/souperdiversifier --diversify input.wasm -o out_folder
```

## How does it work?

We use the `ffi` library to call the `bridge` component inside the Souper implementation, which is merely a shared library. This bridge provides two functions: one, for superoptimizing a passed SouperIR string and another to diversify a passed SouperIR string.

- `superoptimize(souperIR, callback)`
- `superdiversify(souperIR, callback)`

These two functions also receive a callback which is called every time a replacement is found for the peephole string. 
- For the superoptimizer, each found peephole string (for each function) is parsed and transformed to the underlying egraph used by wasm-mutate, which replaces the current peephole by the new "better" replacement.
- In the case of the superdiversifier, we feed the egraph with all found replacements as the rewriting rules. We then call `wasm-mutate` to create as much as possible combinations of all replacements found by Souper.


Roadmap
=

 - [x] Connect Souper and Rust through a ffi bridge
 - [ ] Superoptimize peepholes based in the egraph root for each instruction on each function of an input Wasm.
 - [ ] Superdiversify peepholes.
 - [ ] Connect possible options of Souper from the Rust call.