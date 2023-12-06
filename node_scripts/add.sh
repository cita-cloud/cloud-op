#! /bin/bash

source ./env.sh

running=$(kubectl get pods -l app.kubernetes.io/chain-node=$STS_NAME --no-headers -n$NAME_SPACE 2>/dev/null | wc -l)
echo "$STS_NAME: $running pod(s) is running"

if [ "$running" -eq 0 ]; then
    echo exit
    exit
fi
stop_scale=$((running - 1))
backup_node="$STS_NAME-$stop_scale"
export BACKUP_NODE=$backup_node
new_node="$STS_NAME-$running"
export NEW_NODE=$new_node
echo "backup $backup_node to $new_node"

kubectl scale sts $STS_NAME --replicas=$stop_scale -n$NAME_SPACE >/dev/null

# check backup_node stopped
while true; do
    if kubectl get pod $backup_node -n$NAME_SPACE >/dev/null 2>&1; then
        echo "waiting $backup_node stop..."
        sleep 5
    else
        echo "$backup_node stopped"
        break
    fi
done

# todo: update yamls
python update_yamls.py
echo "yamls updated"

# backup
kubectl apply -f ./yamls/backup_pvc.yaml -n$NAME_SPACE >/dev/null
kubectl apply -f ./yamls/backup_job.yaml -n$NAME_SPACE >/dev/null
echo "waitting $backup_node backup..."
kubectl wait --for=condition=complete --timeout=-1s job/backup-job -n $NAME_SPACE
kubectl scale sts $STS_NAME --replicas=$running -n$NAME_SPACE >/dev/null

# recover
kubectl apply -f ./yamls/node_pvc.yaml -n$NAME_SPACE >/dev/null
kubectl apply -f ./yamls/recover_job.yaml -n$NAME_SPACE >/dev/null
echo "waitting $new_node recover..."
kubectl wait --for=condition=complete --timeout=-1s job/recover-job -n $NAME_SPACE

add_scale=$((running + 1))
kubectl scale sts $STS_NAME --replicas=$add_scale -n$NAME_SPACE >/dev/null

# delete
kubectl delete job backup-job -n$NAME_SPACE >/dev/null
kubectl delete job recover-job -n$NAME_SPACE >/dev/null
kubectl delete pvc cloud-op-backup -n$NAME_SPACE >/dev/null

echo "add $new_node done!"
