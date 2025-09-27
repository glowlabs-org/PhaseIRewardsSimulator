#!/bin/bash
script_status=0
handle_error(){ script_status=1; }
trap 'handle_error' ERR

cargo fmt
cargo build
cargo nextest run --no-tests=pass
cargo nextest run --no-tests=pass -- --ignored
cargo clippy -- -D warnings
cargo build --release

pkill -f rewards-simulator || true
while pgrep -f rewards-simulator >/dev/null; do
  sleep 0.1
done
cp target/debug/rewards-simulator .

./rewards-simulator >/dev/null 2>&1 &
RS_PID=$!
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

if wait_for_port 127.0.0.1 35025 10; then
  if command -v curl >/dev/null 2>&1; then
    for j in $(seq 1 10); do
      if curl -fsS "http://127.0.0.1:35025/index.html" >/dev/null; then
        break
      fi
      sleep 0.5
    done
  fi

  run_frontend_tests(){
    local URL="${FRONTEND_TEST_URL:-http://127.0.0.1:35025/index.html?runTests=true}"
    local VTB="${VIRTUAL_TIME_BUDGET_MS:-60000}"
    local PROC_TIMEOUT_MS=120000
    local CHROME="chromium"
    if ! command -v "$CHROME" >/dev/null 2>&1; then
      echo "ERROR: 'chromium' not found" >&2
      return 1
    fi
    local DOM_TMP; DOM_TMP="$(mktemp)"
    local DOM_NORM; DOM_NORM="$(mktemp)"
    local PROFILE_DIR; PROFILE_DIR="$(mktemp -d)"
    echo "INFO: running headless: $CHROME --dump-dom $URL (vtb=${VTB}ms)" >&2
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
      --timeout="$PROC_TIMEOUT_MS" \
      --dump-dom "$URL" >"$DOM_TMP" 2>/dev/null; then
      :
    else
      rm -f "$DOM_TMP" "$DOM_NORM"; rm -rf "$PROFILE_DIR"
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
    if [ -z "$HARNESS_OUT" ]; then
      if grep -q 'id="__TEST_OUTPUT__"' "$DOM_NORM"; then
        echo "INFO: __TEST_OUTPUT__ exists but is empty" >&2
      else
        echo "ERROR: __TEST_OUTPUT__ not found" >&2
      fi
      if [ -n "$STATUS" ]; then
        echo "INFO: data-test-status: $STATUS" >&2
      else
        echo "ERROR: No data-test-status on <html>" >&2
      fi
    fi
    rm -f "$DOM_TMP" "$DOM_NORM"; rm -rf "$PROFILE_DIR"
    if [ "$STATUS" = "passed" ]; then
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

kill "$RS_PID" 2>/dev/null || true
wait "$RS_PID" 2>/dev/null || true

exit $script_status
