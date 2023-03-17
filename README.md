## Obfuscation analysis for Wasm

[![Build and deploy](https://github.com/Jacarte/obfuscation_wasm/actions/workflows/build_and_deploy.yml/badge.svg)](https://github.com/Jacarte/obfuscation_wasm/actions/workflows/build_and_deploy.yml) [![Build docker evasor](https://github.com/Jacarte/obfuscation_wasm/actions/workflows/build_docker_image.yml/badge.svg)](https://github.com/Jacarte/obfuscation_wasm/actions/workflows/build_docker_image.yml) [![Build docker oracle](https://github.com/Jacarte/wasm_evasion/actions/workflows/build_docker_image_oracles.yml/badge.svg)](https://github.com/Jacarte/wasm_evasion/actions/workflows/build_docker_image_oracles.yml)

This repo contains the tooling and the reproduction of our experiments on Wasm obfuscation.

### Setup & requirements
- Clone this repo and its submodules `git clone --recursive`
- Install Rust in your computer
    - Set nightly as the version `rustup default nightly`
    - Compile the analyzer tool `cd crates/evasor && cargo build`

- As an alternative, you can download the [ubuntu release binary](https://github.com/Jacarte/obfuscation_wasm/releases/download/0.1.0/analyzer) `wget -O analyzer https://github.com/Jacarte/obfuscation_wasm/releases/download/0.1.0/evasor_linux_64amd`

## Evasor

The `evasor` bin perform the evasion of a passed oracle. The oracle can be set with the `--oracle` option. The oracle argument should be another executable script or binary that receives a Wasm program as the first argument. The oracle binary should return exit code 0 if the binary evades, otherwise the exit code is used by the evasor as the numeric value returned by the fitness function. For example, to perform the evasion of VirusTotal, the exit code of the script is the number of bypassed vendors.

### Examples

- Run the baseline evasion over the MINOS oracle: `RUST_BACKTRACE=1 RUST_LOG=evasor=debug ./target/release/evasor --dbconn "datas/minos" mutate --seed 0 -s 10 -e --attempts 1000 -p 1  --input <input.wasm> --oracle python3 ../../oracles/minos/minio.py `


- Run the basesline evasion over the VirusTotal oracle. This example assumes that out VirusTotal oracle is runnin on `http://127.0.0.1:4000`. Follow the [instructions](/oracles/vt_custom_chrome) to deploy our VirusTotal wrapper: `RUST_BACKTRACE=1 RUST_LOG=evasor=debug ./target/release/evasor --dbconn "datas/all" mutate --seed 0 --bulk-size 50 -s 10 -e --attempts 1000 -p 1  --input /input.wasm --oracle python3 ../../oracles/vt_custom_chrome/vt_oracle_count.py http://127.0.0.1:4000 vt vt vt123 malware_file_1`

- Run the mcmc evasion over VirusTotal (assume the VirusTotal wrapper of the previous example): `RUST_LOG=evasor=debug ./target/release/evasor --dbconn "datas/all" mutate --use-reward --seed 0 --beta 0.3 --peek_count 2 -e --attempts 1000 --input /input.wasm --oracle python3 ../../oracles/vt_custom_chrome/vt_oracle_count_reward.py http://127.0.0.1:4000 vt vt vt123 multiple_steps_malware_file`

### Evasor CLI

To access the help lines of the tool, run `./evasor --help`.

### Tests
- Run `cargo test --features <wasm-mutate features>`

## Reproducing our experiments

Our experiments run as an [Argo workflow](https://argoproj.github.io/), the main reason is that the evasion pipeline can escalate horizontally, i.e., how job per malware. To fully reproduce our experiments a Kubernetes cluster is needed (minikube is an option as well for local testing). Once with the kubernetes cluster set, run the [install script](/kube/deploy.sh). The later script will create the services for argo and the artifact storage in MINIO. Thus, all jobs of evasion will collect data in the same storage layer, and you can collect them later.

Once the deploy script ran, submit each experiment as an argo job `argo submit <job.yml>`. Check the job scripts if you find incongruences with the docker images used by them.
