#!/usr/bin/env bash
SCENARIO_DIR="$(cd "$(dirname "$0")" && pwd)"

docker compose -f "$SCENARIO_DIR/docker-compose.yml" exec -T db \
  pg_isready -U postgres -d appdb > /dev/null 2>&1
