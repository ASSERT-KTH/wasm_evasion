FROM ubuntu:20.04
RUN rm /bin/sh && ln -s /bin/bash /bin/sh
RUN apt-get update
RUN apt-get install -y wget jq curl unzip p7zip-full p7zip-rar build-essential gcc git python3 python3-pip

RUN curl --proto '=https' --tlsv1.2 -sSf  https://sh.rustup.rs | bash -s -- -y
RUN source "$HOME/.cargo/env"
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
RUN export PATH="$PATH:$HOME/.cargo/bin"
RUN $HOME/.cargo/bin/rustup default nightly
# Copy the source code
RUN git clone --recursive https://github.com/Jacarte/wasm_evasion.git

RUN pip3 install -r /wasm_evasion/oracles/requirements.txt
WORKDIR /wasm_evasion/crates/evasor
RUN $HOME/.cargo/bin/cargo build --features=wasm_mutate/all --release