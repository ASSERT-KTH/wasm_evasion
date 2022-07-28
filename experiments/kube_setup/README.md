# Experiments reproduction

### Setup

We run our experiments in a k8s cluster. This allows us to parallelize as much as possible the processing. On top of k8s we use [Argo](https://argoproj.github.io/argo-workflows/) to create a workflow for each needed experiment. Besides, we use [minio](#) to orchestrate the gathering of artifacts if the experiments results in files.

- Create your k8s cluster
- Setup a public reachable mongodb db at address `<mondodb>`
- **Setup argo and minio**: Run `bash deploy.sh`. This script will create the corresponding namespaces and allocations. Notice that the initial allocation for minio can be changed in `minio.yml`, the default value is 200Gi of storage allocation.
- **Forward ports adn create the artifacts bucket**: In separate terminals, run `kubectl port-forward -n minio service/minio-service  3434`, `kubectl port-forward -n minio service/minio-service  3435` and `argo server --auth-mode server`. Enter `localhost:3434` and create a new bucket (readwrite permissions) with the name `my-bucket`. 
- **Submit the corresponding workflows**: Locate the workflow file in the `all` or `filtered` folders. Then, run `argo submit <workflow_file_name> -p "dbconn=mongodb://<mongodb>" -p "dbpass=..." -p "dbuser=..." -p "collection=<collection_name>"`
- **Collect the artifacts**: You should be able to collect the artifacts in the minio UI forwarding, (localhost:3434)


### TIPS
- Copy from pvc minio: `kubectl cp minio/minio-deployment-8657988795-7mmpk:storage/my-bucket .` Or just create a new pod wiht the mounted pvc `kubectl apply -f -n minio dataaccess.yml`. The other options is to use the mc (minio console).

  -  ```yml
        apiVersion: v1
        kind: Pod
        metadata:
        name: dataaccess
        spec:
        containers:
        - name: alpine
            image: ubuntu:20.04
            command: [ 'bash', '-c', 'apt-get update && apt-get install -y tar' ,'sleep', 'infinity']
            volumeMounts:
            - name: mypvc
            mountPath: /data
        volumes:
        - name: mypvc
            persistentVolumeClaim:
            claimName: mypvc

        ```
    - Copy from local to minio: `kubectl cp minio/dataaccess:/data/my-bucket/my-artifacts/metas_filtered_0 metas_filtered_0`

    - `mc cp minioalias/my-bucket/... .`