#!/usr/bin/env bash
set -e
cd "$(dirname "$0")"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Cargo/Rust is not installed yet."
  echo "Install Rust first, then run this script again."
  exit 1
fi

cargo run --release
