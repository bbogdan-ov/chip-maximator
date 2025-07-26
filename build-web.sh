#!/usr/bin/bash

# Build the project using this script and run the project using any http server
# Example:
#     ./build-web.sh --release && basic-http-server web/

set -ex

BIN_NAME="chip-maximator.wasm"

if [ "$1" == "release" ] || [ "$1" == "--release" ]; then
	cargo build --release --target wasm32-unknown-unknown
	cp target/wasm32-unknown-unknown/release/$BIN_NAME web/
else
	cargo build --target wasm32-unknown-unknown
	cp target/wasm32-unknown-unknown/debug/$BIN_NAME web/
fi
