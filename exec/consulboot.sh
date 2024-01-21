#!/bin/bash

aclboot=$(docker compose exec consul-server consul acl bootstrap)
acc_token=$(echo "${aclboot}" | grep "SecretID" | cut -d":" -f2 | tr -d '[:space:]')
docker compose exec consul-server consul kv import -token="${acc_token}" @/consul/dumps/kv_dump.json
echo "${acc_token}" > ./tokens/.consul-token
export CONSUL_HTTP_TOKEN=$(cat ./tokens/.consul-token)
export CONSUL_HTTP_ADDR="http://127.0.0.1:8500"
