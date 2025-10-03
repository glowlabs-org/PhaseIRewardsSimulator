#!/usr/bin/env bash
set -euo pipefail

usage(){ echo "Usage: $0 WEEK_NUMBER" >&2; exit 1; }

[ $# -ge 1 ] || usage
WEEK="$1"
[[ "$WEEK" =~ ^[0-9]+$ ]] || { echo "ERROR: WEEK_NUMBER must be an integer" >&2; exit 1; }

BASE_URL="${REWARDS_URL:-http://127.0.0.1:35025/api/rewards-simulator?preloadGlowV1=true}"
SEP="&"
[[ "$BASE_URL" == *\?* ]] || SEP="?"
URL="${BASE_URL}${SEP}week=${WEEK}"

OUT_FILE="week${WEEK}Distribution.json"

REQ_BODY='{"cgpLeftovers":{}, "solarFarms": []}'

TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT

curl -sS -H 'content-type: application/json' -X POST -d "$REQ_BODY" "$URL" -o "$TMP" || { echo "ERROR: request failed" >&2; exit 1; }

# If server returned { "errors": [...], "output": {...} } use .output; else use body as-is.
# If an "error" field exists (e.g., 404 for missing week), fail.
if ! jq -e 'if type=="object" and has("error") then halt_error(1) else (.output // .) end' "$TMP" > "$OUT_FILE"; then
  echo "ERROR: week ${WEEK} not found or invalid response" >&2
  exit 1
fi

jq . "$OUT_FILE" > "${OUT_FILE}.tmp" && mv "${OUT_FILE}.tmp" "$OUT_FILE"
echo "Saved: $OUT_FILE"
