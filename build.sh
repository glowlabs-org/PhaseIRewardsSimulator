#!/bin/bash

script_status=0
handle_error() {
    script_status=1
}
trap 'handle_error' ERR

cargo fmt
cargo build

cargo nextest run --no-tests=pass
cargo nextest run --no-tests=pass -- --ignored

cargo clippy -- -D warnings
cargo build --release

pkill -f rewards-simulator || true
while pgrep -f rewards-simulator > /dev/null; do
	sleep 0.1
done
cp target/debug/rewards-simulator .
./rewards-simulator >/dev/null 2>&1 &

RS_PID=$!
wait_for_port() {
    local host="$1" port="$2" timeout="${3:-10}"
    for _ in $(seq 1 $((timeout * 10))); do
        if (exec 3<>"/dev/tcp/${host}/${port}") 2>/dev/null; then
            exec 3>&-
            return 0
        fi
        sleep 0.1
    done
    return 1
}
if ! wait_for_port 127.0.0.1 35025 10; then
    echo "ERROR: rewards-simulator did not open 127.0.0.1:35025 within timeout" >&2
    script_status=1
else
    FRONTEND_TEST_URL="http://127.0.0.1:35025/index.html?runTests=true" \
    ./frontend-build.sh || script_status=1
fi

exit $script_status
