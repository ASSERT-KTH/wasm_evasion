files=$@

for file in $files
do
    echo $file
    C1=$($WASM2WAT --enable-all --verbose $file -o /dev/null 2>&1  | grep OnFuncType | wc -l)
    CC=$(($C1))
    echo $CC
    if [ $CC -gt 300 ]
    then
        echo "Large"
        echo -n "0" > /dev/stderr
        exit 0
    else

        echo -n "$CC" > /dev/stderr
        exit $CC
    fi
done
