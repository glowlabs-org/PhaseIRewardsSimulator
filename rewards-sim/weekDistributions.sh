#!/usr/bin/env bash
set -euo pipefail

# weekDistributions.sh
# Usage: ./weekDistributions.sh WEEK_NUMBER
#
# Calls the rewards-simulator API, extracts the distribution object for the
# provided week, and writes it to "weekNDistribution.json" (pretty JSON).
#
# Requirements:
# - curl
# - jq
#
# Environment variables:
# - REWARDS_URL: Override the API URL (defaults to basic endpoint with preload).
#   Example: REWARDS_URL="http://127.0.0.1:35025/api/rewards-simulator?preloadGlowV1=true"

usage() {
  echo "Usage: $0 WEEK_NUMBER" >&2
  echo "Example: $0 97" >&2
}

die() {
  echo "ERROR: $*" >&2
  exit 1
}

command -v curl >/dev/null 2>&1 || die "curl is required but not found"
command -v jq >/dev/null 2>&1 || die "jq is required but not found"

if [ $# -lt 1 ]; then
  usage
  exit 1
fi

WEEK="$1"
if ! [[ "$WEEK" =~ ^[0-9]+$ ]]; then
  die "WEEK_NUMBER must be a positive integer (got: $WEEK)"
fi

API_URL="${REWARDS_URL:-http://127.0.0.1:35025/api/rewards-simulator?preloadGlowV1=true}"
OUT_FILE="week${WEEK}Distribution.json"  # intentional spelling per specification

TMP_BODY="$(mktemp)"
TMP_HDRS="$(mktemp)"
trap 'rm -f "$TMP_BODY" "$TMP_HDRS"' EXIT

# Minimal body; with preloadGlowV1=true, server will merge in v1-data if available.
REQ_BODY='{"cgpLeftovers": {}, "solarFarms": []}'

# Perform request
# 200 OK returns the public output object directly (map keyed by week as string).
# 422 Unprocessable Entity returns { "errors": [...], "output": { ... } }.
curl -sS -D "$TMP_HDRS" -H 'content-type: application/json' -X POST -d "$REQ_BODY" "$API_URL" -o "$TMP_BODY" || die "request failed (network error)"

# Extract HTTP status
STATUS_CODE="$(awk '/^HTTP\// {code=$2} END {print code}' "$TMP_HDRS" || true)"
[ -n "$STATUS_CODE" ] || die "failed to determine HTTP status from response"

case "$STATUS_CODE" in
  200|422) ;;
  *)
    echo "Unexpected HTTP status: $STATUS_CODE" >&2
    echo "Response body:" >&2
    cat "$TMP_BODY" >&2 || true
    exit 1
    ;;
esac

# Extract the output map and select the requested week by key, writing pretty JSON.
if ! jq -e 'if (type=="object" and has("output")) then .output else . end | .["'"$WEEK"'"]' "$TMP_BODY" > "$OUT_FILE"; then
  die "week $WEEK not found in response or invalid response structure"
fi

# Normalize formatting (pretty print)
tmp_pretty="$(mktemp)"
jq . "$OUT_FILE" > "$tmp_pretty" && mv "$tmp_pretty" "$OUT_FILE"

echo "Saved week $WEEK distribution to: $OUT_FILE"
