#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: $0 <application_id>" >&2
  echo "  e.g. $0 loan-12345" >&2
  exit 1
fi

curl -s http://localhost:3000/applications/"$1" | jq .
