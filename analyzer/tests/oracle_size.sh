files=$@

for file in $files
do
    echo $(wc -c $file | awk '{printf $1}')
    if [ $(wc -c $file | awk '{printf $1}') -gt 40000 ]
    then
        exit 1
    fi
done

exit 0