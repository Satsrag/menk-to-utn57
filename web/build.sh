#!/bin/sh
set -eu
cd "$(dirname "$0")"

required_version=0.2.125
if ! command -v wasm-bindgen >/dev/null 2>&1 || [ "$(wasm-bindgen --version)" != "wasm-bindgen $required_version" ]; then
    echo "Install the matching binding generator: cargo install wasm-bindgen-cli --version $required_version --locked" >&2
    exit 1
fi

if ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
    echo 'Install the browser target: rustup target add wasm32-unknown-unknown' >&2
    exit 1
fi

cargo build --manifest-path wasm/Cargo.toml --target-dir wasm/target \
    --target wasm32-unknown-unknown --release --locked
wasm-bindgen wasm/target/wasm32-unknown-unknown/release/menk_to_utn57_web.wasm \
    --target web --out-dir public/pkg --out-name menk_to_utn57
echo 'Built web/public — serve this directory with a local static HTTP server (e.g. python3 -m http.server -b 127.0.0.1 8080 -d web/public).'
