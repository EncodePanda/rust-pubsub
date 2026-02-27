#!/bin/sh
set -e

echo "Building and starting containers..."
docker compose up --build -d

echo "Tailing logs (Ctrl+C to stop watching, containers keep running)..."
docker compose logs -f &
LOGS_PID=$!

echo "Waiting for API service to become ready..."
TIMEOUT=120
ELAPSED=0
until curl -s -o /dev/null -w '' http://localhost:3000/applications/health-check 2>/dev/null; do
  if [ "$ELAPSED" -ge "$TIMEOUT" ]; then
    echo "Timeout: API service not ready after ${TIMEOUT}s"
    kill $LOGS_PID 2>/dev/null || true
    exit 1
  fi
  sleep 2
  ELAPSED=$((ELAPSED + 2))
done

echo ""
echo "API service is ready at http://localhost:3000"
kill $LOGS_PID 2>/dev/null || true
