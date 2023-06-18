docker run -d \
  --name=swag \
  --cap-add=NET_ADMIN \
  -e PUID=1000 \
  -e PGID=1000 \
  -e TZ=US/Eastern \
  -e URL=serverurl.com \
  -e VALIDATION=http \
  -e SUBDOMAINS=change-or-remove, \
  -e EMAIL=change-or-remove@no-reply.com \
  -p 443:443 \
  -p 80:80 \
  -v /deploy/config:/config \
  --restart unless-stopped \
  lscr.io/linuxserver/swag:latest
