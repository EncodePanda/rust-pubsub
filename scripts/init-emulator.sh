#!/bin/sh
set -e

EMULATOR_HOST="${PUBSUB_EMULATOR_HOST:-pubsub-emulator:8085}"
PROJECT_ID="local-project"

MAX_RETRIES=15
RETRY_INTERVAL=2
ATTEMPT=0

echo "Waiting for Pub/Sub emulator to be ready..."
until curl -sf "http://${EMULATOR_HOST}/v1/projects/${PROJECT_ID}/topics" > /dev/null 2>&1; do
  ATTEMPT=$((ATTEMPT + 1))
  if [ "$ATTEMPT" -ge "$MAX_RETRIES" ]; then
    echo "ERROR: Pub/Sub emulator not ready after $((MAX_RETRIES * RETRY_INTERVAL))s. Giving up."
    exit 1
  fi
  echo "  emulator not ready, retrying in ${RETRY_INTERVAL}s... (attempt ${ATTEMPT}/${MAX_RETRIES})"
  sleep "$RETRY_INTERVAL"
done
echo "Emulator is ready."

echo "Creating topic 'loan-applications'..."
curl -s -X PUT "http://${EMULATOR_HOST}/v1/projects/${PROJECT_ID}/topics/loan-applications"
echo ""

echo "Creating topic 'loan-decisions'..."
curl -s -X PUT "http://${EMULATOR_HOST}/v1/projects/${PROJECT_ID}/topics/loan-decisions"
echo ""

echo "Creating subscription 'loan-applications-sub'..."
curl -s -X PUT "http://${EMULATOR_HOST}/v1/projects/${PROJECT_ID}/subscriptions/loan-applications-sub" \
  -H "Content-Type: application/json" \
  -d "{\"topic\": \"projects/${PROJECT_ID}/topics/loan-applications\"}"
echo ""

echo "Creating subscription 'loan-decisions-sub'..."
curl -s -X PUT "http://${EMULATOR_HOST}/v1/projects/${PROJECT_ID}/subscriptions/loan-decisions-sub" \
  -H "Content-Type: application/json" \
  -d "{\"topic\": \"projects/${PROJECT_ID}/topics/loan-decisions\"}"
echo ""

echo "Emulator initialization complete."
