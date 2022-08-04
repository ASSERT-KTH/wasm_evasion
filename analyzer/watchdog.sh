file=$1


mc alias set exp https://minio-service.minio:3434
mc --quiet cp "$1" exp/my-bucket/snapshots/ 