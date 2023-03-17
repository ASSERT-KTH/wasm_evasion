 folder=$1

echo $(find $folder -type f -name "*.wasm" | wc -l)
sleep 60

for w in $(find $folder -type f -name "*.wasm")
do
    abspath=$(realpath $w)
    echo "$abspath"
    curl -X POST --user 'admin:admin' -F "file=@$abspath" http://0.0.0.0:4000/details/details

done