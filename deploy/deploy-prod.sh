#!/bin/bash

project_root=$(git rev-parse --show-toplevel)

if [[ "$(docker ps -aq -f name=swag)" ]]; then
    docker stop swag &>/dev/null
  if [[ "$(docker ps -aq -f name=swag)" ]]; then
    docker rm swag &>/dev/null
  fi
fi

docker_run="docker run -d \
  --cap-add=NET_ADMIN \
  --name=swag \
  --network host \
  --restart unless-stopped \
  -e PUID=1000 \
  -e PGID=1000 \
  -e TZ=US/Eastern \
  -e VALIDATION=http \
  -e FILE__URL=/config/url-secret"

if [[ -f "$project_root/deploy/config/subdomains-secret" ]]; then
  docker_run+=" -e FILE__SUBDOMAINS=/config/subdomains-secret"
fi

docker_run+=" -v $project_root/deploy/config:/config"
docker_run+=" lscr.io/linuxserver/swag:arm32v7-2.6.0"

eval "${docker_run}"

echo "Swag container started."
