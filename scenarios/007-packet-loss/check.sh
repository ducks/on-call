#!/usr/bin/env bash
# Must succeed 5 times in a row to confirm packet loss is gone
set -e
for i in $(seq 1 5); do
  curl -sf --max-time 2 http://localhost:8080/health > /dev/null || exit 1
done
