file=$1
interval=$2
folder=$3

MINIO_ROOT_USER=minio
MINIO_ROOT_PASSWORD=minio123

function save() {

    fname=$(basename $1)
    t=$(date +"%T")
    mc --quiet cp "$1" exp/my-bucket/snapshots/$2/$t${fname} 
}


mc config host add exp http://minio-service.minio:3434 minio minio123


while true
do
    sleep $interval
    save $file $folder
done