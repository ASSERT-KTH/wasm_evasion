## Obfuscation analysis for Wasm

This repo contains the tooling and the reproduction of our experiments on Wasm obfuscation.

### Setup & requirements
- Clone this repo and its submodules `git clone --recursive` 
- Install Rust in your computer
    - Set nightly as the version `rustup default nightly`
    - Compile the analyzer tool `cd analyzer && cargo build`

- As an alternative, you can download the [ubuntu release binary](https://github.com/Jacarte/obfuscation_wasm/releases/download/0.1.0/analyzer) `wget -O analyzer https://github.com/Jacarte/obfuscation_wasm/releases/download/0.1.0/analyzer`
- Run the analysis on a binary or a folder of Wasm binaries `RUST_LOG=analyzer,wasm-mutate=debug ./target/debug/analyzer --dbconn "datas/database" extract -d 4 --input "binary.wasm"  `

### Analyzer CLI

The tool provides a cli to perform mutation analysis on binaries. To access the help lines of the tool, run `./analyzer --help`. The analysis tool provides 6 subcommands:

### Notebooks

We continuously update our experiments insights using the notebooks [here](./notebooks). Our experiments are based on the [wasmbench](todo) dataset.

### Featurized wasm-mutate

TODO

### Simple KV database to restore the experiment's data