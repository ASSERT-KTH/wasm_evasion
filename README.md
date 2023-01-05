## Obfuscation analysis for Wasm 

[![Build and deploy](https://github.com/Jacarte/obfuscation_wasm/actions/workflows/build_and_deploy.yml/badge.svg)](https://github.com/Jacarte/obfuscation_wasm/actions/workflows/build_and_deploy.yml) [![Build docker evasor](https://github.com/Jacarte/obfuscation_wasm/actions/workflows/build_docker_image.yml/badge.svg)](https://github.com/Jacarte/obfuscation_wasm/actions/workflows/build_docker_image.yml) [![Build docker oracle](https://github.com/Jacarte/wasm_evasion/actions/workflows/build_docker_image_oracles.yml/badge.svg)](https://github.com/Jacarte/wasm_evasion/actions/workflows/build_docker_image_oracles.yml)

This repo contains the tooling and the reproduction of our experiments on Wasm obfuscation.

### Setup & requirements
- Clone this repo and its submodules `git clone --recursive` 
- Install Rust in your computer
    - Set nightly as the version `rustup default nightly`
    - Compile the analyzer tool `cd crates/evasor && cargo build`

- As an alternative, you can download the [ubuntu release binary](https://github.com/Jacarte/obfuscation_wasm/releases/download/0.1.0/analyzer) `wget -O analyzer https://github.com/Jacarte/obfuscation_wasm/releases/download/0.1.0/evasor_linux_64amd`
- Run the analysis on a binary or a folder of Wasm binaries `RUST_LOG=evasor,wasm-mutate=debug ./target/debug/evasor --dbconn "datas/database" extract -d 4 --input "binary.wasm"  `

### Evasor CLI

The tool provides a cli to perform mutation analysis on binaries. To access the help lines of the tool, run `./evasor --help`. 

### Tests
- Run `cargo test --features <wasm-mutate features>`
