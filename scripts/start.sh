#!/bin/sh
set -e

echo "Building and starting containers..."
docker compose up --build -d

echo "Tailing logs (Ctrl+C to stop watching, containers keep running)..."
docker compose logs -f &
LOGS_PID=$!
