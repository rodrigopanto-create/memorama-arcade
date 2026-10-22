#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

for tool in cargo rustup; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "Falta $tool. Instala Rust con rustup para compilar." >&2
        exit 1
    fi
done

if ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
    rustup target add wasm32-unknown-unknown
fi

cargo build --locked --release --target wasm32-unknown-unknown --bin memorama_arcade

cargo run --locked --example build_web
