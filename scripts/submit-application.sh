#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 2 ]; then
  echo "Usage: $0 <user_id> <amount> [currency]" >&2
  echo "  e.g. $0 u-7712 15000 PLN" >&2
  exit 1
fi

USER_ID="$1"
AMOUNT="$2"
CURRENCY="${3:-PLN}"

curl -s -X POST http://localhost:3000/applications \
  -H "Content-Type: application/json" \
  -d "{\"user_id\": \"${USER_ID}\", \"amount\": ${AMOUNT}, \"currency\": \"${CURRENCY}\"}" | jq .
