
if [ -f filtered-binaries-metadata.7z ] 
then
    echo "Exist"
else
    mkdir -p filtered
    cd filtered
    wget https://github.com/sola-st/WasmBench/releases/download/v1.0/filtered-binaries-metadata.7z
    cd ..
fi


if [ -f all-binaries-metadata.7z ] 
then
    echo "Exist"
else
    mkdir -p all
    cd all
    wget https://github.com/sola-st/WasmBench/releases/download/v1.0/all-binaries-metadata.7z
    cd ..
fi




cd all
7z x all-binaries-metadata.7z
cd ..

cd filtered
7z x filtered-binaries-metadata.7z
cd ..