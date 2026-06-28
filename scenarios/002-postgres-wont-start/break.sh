#!/usr/bin/env bash
set -e

SCENARIO_DIR="$(cd "$(dirname "$0")" && pwd)"

# Corrupt the data directory ownership so postgres refuses to start
docker compose -f "$SCENARIO_DIR/docker-compose.yml" exec -T db \
  chown -R root:root /var/lib/postgresql/data

# Restart the container so postgres tries and fails to start
docker compose -f "$SCENARIO_DIR/docker-compose.yml" restart db
