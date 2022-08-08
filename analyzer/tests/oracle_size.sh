file=$1

if [ $(wc -c $file | awk '{printf $1}') -gt 40000 ]
then
    exit 1
else
    exit 0
fi