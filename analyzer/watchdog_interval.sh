file=$1
interval=$2

MINIO_ROOT_USER=minio
MINIO_ROOT_PASSWORD=minio123

function save() {

    mc --quiet cp "$1" exp/my-bucket/snapshots/ 
}

function save_mem() {
    ps -o pid,user,%mem,command ax > $1.mem.log
    mc --quiet cp $1.mem.log exp/my-bucket/snapshots/ 
}


mc config host add exp http://minio-service.minio:3434 minio minio123


while true
do
    sleep $interval
    save $file
    save_mem $file
done