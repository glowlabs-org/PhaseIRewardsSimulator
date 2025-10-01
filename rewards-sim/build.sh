#!/bin/bash
BINARY="rewards-simulator"
PORT=35025

script_status=0
handle_error(){ script_status=1; }
trap 'handle_error' ERR

cargo fmt
cargo build
cargo nextest run --no-tests=pass
cargo nextest run --no-tests=pass -- --ignored
cargo clippy -- -D warnings
cargo build --release

pkill -f "$BINARY" || true
while pgrep -f "$BINARY" >/dev/null; do
  sleep 0.1
done
cp "target/debug/$BINARY" .

PORT="$PORT" "./$BINARY" >/dev/null 2>&1 &
wait_for_port(){
  local host="$1"
  local port="$2"
  local timeout="${3:-10}"
  local i
  for i in $(seq 1 $((timeout*10))); do
    if (exec 3<>"/dev/tcp/${host}/${port}") 2>/dev/null; then
      exec 3>&-
      return 0
    fi
    sleep 0.1
  done
  return 1
}

if wait_for_port 127.0.0.1 "$PORT" 10; then
  if command -v curl >/dev/null 2>&1; then
    for j in $(seq 1 10); do
      if curl -fsS "http://127.0.0.1:${PORT}/index.html" >/dev/null; then
        break
      fi
      sleep 0.5
    done
  fi

  run_frontend_tests(){
    local URL="http://127.0.0.1:${PORT}/index.html?runTests=true"
    local VTB=60000
    local PROC_TIMEOUT_MS=120000
    local CHROME="chromium"
    if ! command -v "$CHROME" >/dev/null 2>&1; then
      echo "ERROR: 'chromium' not found" >&2
      return 1
    fi
    local DOM_TMP; DOM_TMP="$(mktemp)"
    local DOM_NORM; DOM_NORM="$(mktemp)"
    local PROFILE_DIR; PROFILE_DIR="$(mktemp -d)"
    local LOG_TMP; LOG_TMP="$(mktemp)"
    if "$CHROME" \
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
      --enable-logging=stderr \
      --timeout="$PROC_TIMEOUT_MS" \
      --dump-dom "$URL" >"$DOM_TMP" 2>"$LOG_TMP"; then
      :
    else
      rm -f "$DOM_TMP" "$DOM_NORM" "$LOG_TMP"; rm -rf "$PROFILE_DIR"
      return 1
    fi
    tr "'" '"' < "$DOM_TMP" > "$DOM_NORM"
    local HARNESS_OUT
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
    if [ -n "$HARNESS_OUT" ]; then
      printf "%s\n" "$HARNESS_OUT"
    fi
    local STATUS
    STATUS="$(grep -o 'data-test-status="[a-z]*"' "$DOM_NORM" | tail -n1 | cut -d'"' -f2 || true)"

    local ORIGIN="http://127.0.0.1:${PORT}"
    local RAW_ERRS
    RAW_ERRS="$(grep -E 'ERROR:CONSOLE|Uncaught|Unhandled promise rejection' "$LOG_TMP" 2>/dev/null || true)"
    local ORIGIN_ERRS
    ORIGIN_ERRS="$(printf "%s\n" "$RAW_ERRS" | grep -F "$ORIGIN" 2>/dev/null || true)"
    local FILTERED_ERRS
    FILTERED_ERRS="$(printf "%s\n" "$ORIGIN_ERRS" | grep -Ev 'Failed to load resource: net::|ERR_BLOCKED_BY_CLIENT' 2>/dev/null || true)"
    local ERR_COUNT=0
    if [ -n "$FILTERED_ERRS" ]; then
      ERR_COUNT="$(printf "%s\n" "$FILTERED_ERRS" | grep -c . 2>/dev/null || true)"
    fi
    local CONSOLE_OK=1
    if [ "$ERR_COUNT" -gt 0 ]; then
      echo "ERROR: Browser console errors detected during test (count=$ERR_COUNT):" >&2
      printf "%s\n" "$FILTERED_ERRS" | head -n 50 >&2
      CONSOLE_OK=0
    fi

    rm -f "$DOM_TMP" "$DOM_NORM" "$LOG_TMP"; rm -rf "$PROFILE_DIR"

    if [ "$STATUS" = "passed" ] && [ "$CONSOLE_OK" -eq 1 ]; then
      return 0
    else
      return 1
    fi
  }

  if ! run_frontend_tests; then
    script_status=1
  fi
else
  script_status=1
fi

exit $script_status
