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
  -e ONLY_SUBDOMAINS=true \
  -e PUID=1000 \
  -e PGID=1000 \
  -e TZ=US/Eastern \
  -e VALIDATION=dns \
  -e DNSPLUGIN=cloudflare \
  -e FILE__URL=/swag_config/url-secret \
  -e EMAIL=dave@halyard.systems \
  -v $project_root/swag_config:/swag_config"

if [[ -f "$project_root/swag_config/subdomains-secret" ]]; then
  docker_run+=" -e FILE__SUBDOMAINS=/swag_config/subdomains-secret"
fi

docker_run+=" -v $project_root/deploy/config:/config"

if [ -z "$1" ]; then
  docker_run+=" lscr.io/linuxserver/swag:2.10.0"
else
  docker_run+=" lscr.io/linuxserver/swag:arm32v7-2.6.0"
fi

eval "${docker_run}"

echo "Swag container started."
