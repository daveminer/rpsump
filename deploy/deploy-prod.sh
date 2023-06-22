#!/bin/bash

project_root=$(git rev-parse --show-toplevel)

# Load the env file to ensure required vars are set.
source $project_root/deploy/.env.swag

if [[ -z "${URL}" ]]; then
  echo "The URL env var is not set; set it and try again."
  exit 1
fi

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
  -e FILE__URL=$project_root/deploy/.env.swag"

if [[ -n "${SUBDOMAINS}" ]]; then
  docker_run+=" -e FILE__SUBDOMAINS=$project_root/deploy/.env.swag"
fi

docker_run+=" -v $project_root/deploy/config:/config"
docker_run+=" lscr.io/linuxserver/swag:arm32v7-2.6.0"

eval "${docker_run}"

echo "Swag container started."
