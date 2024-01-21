#!/bin/bash
container_name="etcd-server"
root_pass=$(uuidgen)

create_user_cmd="etcdctl user add root:${root_pass}"

docker-compose exec $container_name $create_user_cmd 1>/dev/null
docker-compose exec $container_name etcdctl user grant root root 1>/dev/null
docker-compose exec $container_name etcdctl auth enable 1>/dev/null

echo "root:${root_pass}" | base64 > ./tokens/.etcd-token
etcd_token=$(cat ./tokens/.etcd-token | base64 -d)

docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $container_name

cmds=$(cat docker-config/kv_dump.json | jq -r --arg etcd_token "$etcd_token" \
 '.[] | ["docker", "compose", "exec", "etcd-server", "etcdctl", "--user=\($etcd_token)", "put", .key , (.value | @base64d | tojson), ";"] | join(" ") | tostring')
eval $cmds
