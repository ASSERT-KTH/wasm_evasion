FROM ubuntu:20.04

RUN apt-get update
RUN apt-get install -y wget jq curl unzip

RUN wget -O /analyzer https://github.com/Jacarte/obfuscation_wasm/releases/download/analyzer/analyzer
RUN chmod +x /analyzer

