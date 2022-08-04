file=$1


mc alias set exp minio-service.minio:3434

mc --quiet cp "$file" exp/my-bucket/snapshots/ 