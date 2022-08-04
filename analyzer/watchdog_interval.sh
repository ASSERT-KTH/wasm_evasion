file=$1
interval=$2

function save() {


    mc alias set exp http://minio-service.minio:3434 

    mc --quiet cp "$1" exp/my-bucket/snapshots/ 
}

while true
do
    sleep $interval
    save $file
done