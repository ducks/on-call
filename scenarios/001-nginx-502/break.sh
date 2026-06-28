#!/usr/bin/env bash
set -e

# Change the upstream port in the running nginx config to the wrong port
docker compose -f "$(dirname "$0")/docker-compose.yml" exec -T nginx \
  sed -i 's/app:3000/app:3001/g' /etc/nginx/nginx.conf

docker compose -f "$(dirname "$0")/docker-compose.yml" exec -T nginx \
  nginx -s reload
