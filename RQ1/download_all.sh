
if [ test -f filtered-binaries-metadata.7z ] 
then
    echo "Exist"
else
    wget https://github.com/sola-st/WasmBench/releases/download/v1.0/filtered-binaries-metadata.7z
fi


if [ test -f all-binaries-metadata.7z ] 
then
    echo "Exist"
else
    wget https://github.com/sola-st/WasmBench/releases/download/v1.0/all-binaries-metadata.7z
fi





7z x all-binaries-metadata.7z
7z x filtered-binaries-metadata.7z