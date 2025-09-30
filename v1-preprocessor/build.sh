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

cp target/debug/v1-preprocessor .
./v1-preprocessor
cp assets/v2-configuration.json ../rewards-simulator/v2-configuration.json

exit $script_status
