#!/usr/bin/env bash
# Frontend harness runner for http://127.0.0.1:35025/
# - Prints ONLY the harness sink (<pre id="__TEST_OUTPUT__">...</pre>) to stdout.
# - Exits 0/1 based on <html data-test-status="...">.
# - Uses a fresh Chromium profile and disables caches for determinism.

set -euo pipefail

URL="${FRONTEND_TEST_URL:-http://127.0.0.1:35025/?test=1}"
VTB="${VIRTUAL_TIME_BUDGET_MS:-60000}"
WAIT_SECS="${WAIT_UP_TO_SEC:-5}"
PROC_TIMEOUT_MS=120000
CHROME="chromium"

if ! command -v "$CHROME" >/dev/null 2>&1; then
  echo "ERROR: 'chromium' not found. Install: sudo apt-get update && sudo apt-get install -y chromium" >&2
  exit 1
fi

# Wait briefly for the port if URL is localhost:PORT/...
if [[ "$URL" =~ ^https?://(127\.0\.0\.1|localhost):([0-9]+)/ ]]; then
  host="${BASH_REMATCH[1]}"; port="${BASH_REMATCH[2]}"
  for _ in $(seq 1 $((WAIT_SECS * 10))); do
    if (exec 3<>"/dev/tcp/${host}/${port}") 2>/dev/null; then exec 3>&-; break; fi
    sleep 0.1
  done
fi

DOM_TMP="$(mktemp)"
DOM_NORM="$(mktemp)"
PROFILE_DIR="$(mktemp -d)"
cleanup() {
  rm -f "$DOM_TMP" "$DOM_NORM" 2>/dev/null || true
  rm -rf "$PROFILE_DIR" 2>/dev/null || true
}
trap cleanup EXIT

echo "INFO: running headless: $CHROME --dump-dom $URL (vtb=${VTB}ms)" >&2
set +e
"$CHROME" \
  --headless \
  --disable-gpu \
  --no-sandbox \
  --user-data-dir="$PROFILE_DIR" \
  --disable-application-cache \
  --disk-cache-size=0 \
  --media-cache-size=0 \
  --disable-cache \
  --run-all-compositor-stages-before-draw \
  --virtual-time-budget="$VTB" \
  --timeout="$PROC_TIMEOUT_MS" \
  --dump-dom "$URL" >"$DOM_TMP" 2>/dev/null
CHROME_RC=$?
set -e

if [[ $CHROME_RC -ne 0 ]]; then
  echo "ERROR: headless browser failed (rc=$CHROME_RC). URL: $URL" >&2
  exit 1
fi

# Normalize quotes for simpler grep/awk
tr "'" '"' < "$DOM_TMP" > "$DOM_NORM"

# Robust extractor: handle text on same line as the opening <pre> and/or before the closing </pre>.
HARNESS_OUT="$(awk '
  BEGIN { inpre=0 }
  {
    line=$0
    if (!inpre) {
      if (line ~ /<pre[^>]*id="__TEST_OUTPUT__"[^>]*>/) {
        inpre=1
        sub(/.*<pre[^>]*id="__TEST_OUTPUT__"[^>]*>/, "", line)
      } else {
        next
      }
    }
    if (inpre) {
      if (line ~ /<\/pre>/) {
        sub(/<\/pre>.*/, "", line)
        if (length(line)) print line
        inpre=0
        next
      } else {
        print line
      }
    }
  }
' "$DOM_NORM")"

# Print ONLY harness lines to stdout
if [[ -n "$HARNESS_OUT" ]]; then
  printf "%s\n" "$HARNESS_OUT"
fi

# Determine pass/fail from <html data-test-status="...">
STATUS="$(grep -o 'data-test-status="[a-z]*"' "$DOM_NORM" | tail -n1 | cut -d'"' -f2 || true)"

# Minimal diagnostics to stderr if nothing was captured
if [[ -z "$HARNESS_OUT" ]]; then
  if grep -q 'id="__TEST_OUTPUT__"' "$DOM_NORM"; then
    echo "INFO: __TEST_OUTPUT__ exists but is empty. Did tests run/write output?" >&2
  else
    echo "ERROR: __TEST_OUTPUT__ not found. Ensure ?test=1 (or runTests=true) loads harness/tests." >&2
  fi
  if [[ -n "$STATUS" ]]; then
    echo "INFO: data-test-status: $STATUS" >&2
  else
    echo "ERROR: No data-test-status on <html>. Consider increasing VIRTUAL_TIME_BUDGET_MS ($VTB)." >&2
  fi
fi

# Exit code by status
if [[ "$STATUS" == "passed" ]]; then
  exit 0
elif [[ "$STATUS" == "failed" ]]; then
  exit 1
else
  exit 1
fi
