#!/bin/bash

echo "==> Building bench apps"

docker compose -f docker-compose.bench.yml build

echo "✔ Finished builds."