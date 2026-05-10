#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

echo "Installing Windows cross-compile tools..."
sudo apt update
sudo apt install -y mingw-w64 zip

if ! command -v cargo >/dev/null 2>&1; then
  echo "Cargo/Rust is not installed yet."
  echo "Install Rust first: https://rustup.rs"
  exit 1
fi

echo "Adding Rust Windows target..."
rustup target add x86_64-pc-windows-gnu

mkdir -p .cargo
cat > .cargo/config.toml <<'CARGO_CONFIG'
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"
CARGO_CONFIG

echo "Building Windows .exe..."
cargo build --release --target x86_64-pc-windows-gnu

mkdir -p dist/windows
cp target/x86_64-pc-windows-gnu/release/dmc-substitute-finder.exe dist/windows/DMC_Substitute_Finder.exe

cat > dist/windows/README-WINDOWS.txt <<'README_WINDOWS'
DMC Substitute Finder - Windows Version

To run:
1. Double-click DMC_Substitute_Finder.exe
2. Enter a DMC colour number such as 310, 823, B5200, Blanc, or Ecru.
3. Click Find Substitutes.

If Windows SmartScreen warns you, choose More info > Run anyway.
That happens because this is a homemade app and is not code-signed.
README_WINDOWS

cd dist/windows
zip -r ../DMC_Substitute_Finder_Windows.zip .
cd ../..

echo "Done."
echo "Windows exe: dist/windows/DMC_Substitute_Finder.exe"
echo "Windows zip: dist/DMC_Substitute_Finder_Windows.zip"
