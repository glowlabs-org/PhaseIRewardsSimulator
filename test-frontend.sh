#!/usr/bin/env bash
# ------------------------------------------------------------------------------
# Frontend test runner for a pure HTML/JS/CSS app already served at:
#   http://127.0.0.1:35025/
#
# Behavior:
#   - Loads "?test=1" in headless Chromium
#   - Dumps the DOM to a temp file (never printed)
#   - Prints ONLY <pre id="__TEST_OUTPUT__">...</pre> to stdout
#   - Reads <html data-test-status="passed|failed"> and exits 0/1
#
# Stdout is STRICTLY the harness output. All diagnostics go to stderr.
#
# Env overrides:
#   FRONTEND_TEST_URL      default: http://127.0.0.1:35025/?test=1
#   VIRTUAL_TIME_BUDGET_MS default: 60000
#   CHROME_BIN             path to chromium/chrome binary
#   KEEP_DUMP              if set (non-empty), keeps the DOM dump path printed to stderr
#   DEBUG                  if set (non-empty), preserves browser stderr (don’t /dev/null it)
# ------------------------------------------------------------------------------

set -euo pipefail

URL="${FRONTEND_TEST_URL:-http://127.0.0.1:35025/?test=1}"
VTB="${VIRTUAL_TIME_BUDGET_MS:-60000}"
PROC_TIMEOUT_MS=120000

# Locate chromium
CHROME="${CHROME_BIN:-}"
if [[ -z "${CHROME}" ]]; then
  for c in chromium chromium-browser google-chrome; do
    if command -v "$c" >/dev/null 2>&1; then CHROME="$c"; break; fi
  done
fi
if [[ -z "${CHROME}" ]]; then
  echo "ERROR: chromium not found. Install on Debian: sudo apt-get update && sudo apt-get install -y chromium" >&2
  exit 1
fi

# Quick port wait (in case server was just started earlier in the pipeline)
if [[ "$URL" =~ ^https?://(127\.0\.0\.1|localhost):([0-9]+)/ ]]; then
  host="${BASH_REMATCH[1]}"; port="${BASH_REMATCH[2]}"
  for _ in {1..50}; do
    if (exec 3<>"/dev/tcp/${host}/${port}") 2>/dev/null; then exec 3>&-; break; fi
    sleep 0.1
  done
fi

DOM_TMP="$(mktemp)"
DOM_NORM="$(mktemp)"
cleanup() {
  if [[ -z "${KEEP_DUMP:-}" ]]; then
    rm -f "$DOM_TMP" "$DOM_NORM" 2>/dev/null || true
  else
    echo "INFO: kept dumps: raw=$DOM_TMP norm=$DOM_NORM" >&2
  fi
}
trap cleanup EXIT

# Run headless and dump DOM
echo "INFO: running headless: $CHROME --dump-dom $URL (vtb=${VTB}ms)" >&2
set +e
if [[ -n "${DEBUG:-}" ]]; then
  "$CHROME" --headless --disable-gpu --no-sandbox \
    --run-all-compositor-stages-before-draw \
    --virtual-time-budget="$VTB" \
    --timeout="$PROC_TIMEOUT_MS" \
    --dump-dom "$URL" >"$DOM_TMP"
else
  "$CHROME" --headless --disable-gpu --no-sandbox \
    --run-all-compositor-stages-before-draw \
    --virtual-time-budget="$VTB" \
    --timeout="$PROC_TIMEOUT_MS" \
    --dump-dom "$URL" >"$DOM_TMP" 2>/dev/null
fi
CHROME_RC=$?
set -e

if [[ $CHROME_RC -ne 0 ]]; then
  echo "ERROR: headless browser failed (rc=$CHROME_RC). URL: $URL" >&2
  exit 1
fi

# Normalize quotes to simplify parsing (handles single/double)
tr "'" '"' < "$DOM_TMP" > "$DOM_NORM"

# Extract harness sink to STDOUT (and only that)
HARNESS_OUT="$(awk '
  BEGIN { inpre=0 }
  /<pre[^>]*id="__TEST_OUTPUT__"[^>]*>/ { inpre=1; next }
  /<\/pre>/ { if (inpre) { inpre=0; next } }
  { if (inpre) print }
' "$DOM_NORM")"

# Print ONLY the harness output to stdout
if [[ -n "$HARNESS_OUT" ]]; then
  printf "%s\n" "$HARNESS_OUT"
fi

# Determine status for exit code
STATUS="$(grep -o 'data-test-status="[a-z]*"' "$DOM_NORM" | tail -n1 | cut -d'"' -f2 || true)"

# Diagnostics to stderr when harness output was empty
if [[ -z "$HARNESS_OUT" ]]; then
  # Was the harness element present at all?
  if grep -q 'id="__TEST_OUTPUT__"' "$DOM_NORM"; then
    echo "INFO: __TEST_OUTPUT__ exists but contains no lines. Did any tests call harness.test(...)?" >&2
  else
    echo "ERROR: __TEST_OUTPUT__ not found in DOM." >&2
    echo "  • Ensure your HTML conditionally injects tests/harness.js + tests/tests.js when ?test=1" >&2
    echo "  • Confirm you loaded the correct page: $URL" >&2
  fi

  # If there is a status, report it for debugging (to stderr only)
  if [[ -n "$STATUS" ]]; then
    echo "INFO: data-test-status detected: $STATUS" >&2
  else
    echo "ERROR: No data-test-status on <html>. Harness may not have run or did not finish." >&2
    echo "      Consider increasing VIRTUAL_TIME_BUDGET_MS (currently $VTB)." >&2
  fi
fi

# Exit code
if [[ "$STATUS" == "passed" ]]; then
  exit 0
elif [[ "$STATUS" == "failed" ]]; then
  exit 1
else
  # No status: treat as failure for CI
  exit 1
fi
