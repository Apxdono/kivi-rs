#!/bin/bash

container_name="etcd-server"
root_pass=$(uuidgen)

create_user_cmd="etcdctl user add root:${root_pass}"

docker-compose exec $container_name $create_user_cmd 1>/dev/null
docker-compose exec $container_name etcdctl user grant root root 1>/dev/null
docker-compose exec $container_name etcdctl auth enable 1>/dev/null
echo -n "Basic $(base64 <<< root:${root_pass})" > .etcd-token
docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $container_name
