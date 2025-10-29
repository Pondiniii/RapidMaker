#!/usr/bin/env bash
set -euo pipefail

API_URL="http://127.0.0.1:8080/api/quote"
FILE="test-cube.stl"
if [ ! -f "$FILE" ]; then
  echo "missing $FILE" >&2
  exit 1
fi

curl -sS -X POST "$API_URL" \
  -F "file=@$FILE" \
  -F "material_id=pla" | jq
