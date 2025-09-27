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

exit $script_status
