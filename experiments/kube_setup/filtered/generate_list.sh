for i in $(seq 1 43)
do
    echo  - { folder: "filtered$i", url: "https://github.com/Jacarte/obfuscation_wasm/releases/download/analyzer/filtered$i.zip", path: "filtered$i/all-binaries-metadata/filtered$i" }
done