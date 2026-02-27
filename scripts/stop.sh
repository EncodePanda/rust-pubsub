#!/bin/sh
set -e

echo "Stopping containers..."
docker compose down
echo "Done."
