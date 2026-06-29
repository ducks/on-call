#!/usr/bin/env bash
# Add 40% packet loss on the app container's eth0
CONTAINER=$(docker compose ps -q app)
docker exec "$CONTAINER" tc qdisc add dev eth0 root netem loss 40%
