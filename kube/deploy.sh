kubectl delete cm workflow-controller-configmap -n argo
kubectl create ns argo
kubectl apply -n argo -f https://github.com/argoproj/argo-workflows/releases/download/v3.3.8/install.yaml
kubectl apply -f roles.yml 
kubectl create rolebinding default-admin --clusterrole=admin --serviceaccount=default:default
kubectl create rolebinding arg-dev-binding --clusterrole=argo-dev --serviceaccount=argo:argo -n argo

# kubectl patch configmap workflow-controller-configmap --patch '{"data":{"containerRuntimeExecutor":"pns"}}'

# restart argo
kubectl scale deploy argo-server --replicas 0 -n argo 
kubectl scale deploy workflow-controller --replicas 0 -n argo

kubectl scale deploy argo-server --replicas 1 -n argo
kubectl scale deploy workflow-controller --replicas 1 -n argo 


kubectl create ns minio
kubectl apply -n minio -f minio.yml 

kubectl patch cm -n argo workflow-controller-configmap --type merge --patch "$(cat config.yml)"
kubectl describe configmap workflow-controller-configmap -n argo


kubectl logs $(kubectl get po -n argo | grep 'argo-server' | awk '{print $1}') -n argo
kubectl logs $(kubectl get po -n argo | grep 'workflow-controller' | awk '{print $1}') -n argo | grep pns

kubectl create secret generic argo-artifacts --from-literal=accesskey="minio" --from-literal=secretkey="minio123"
kubectl apply -f https://raw.githubusercontent.com/kubernetes/dashboard/v2.0.0-beta8/aio/deploy/recommended.yaml
