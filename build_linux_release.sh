#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release

rm -rf dist/linux
mkdir -p dist/linux
cp target/release/flossfinder dist/linux/FlossFinder
chmod +x dist/linux/FlossFinder

cat > dist/linux/README-LINUX.txt <<'README_LINUX'
FlossFinder - Linux Version

To run:
1. Extract this archive.
2. Open a terminal in the extracted folder.
3. Run:

./FlossFinder

If it does not launch, make sure it is executable:

chmod +x FlossFinder
./FlossFinder
README_LINUX

cd dist
tar -czvf FlossFinder_Linux_x86_64.tar.gz linux
cd ..

echo "Built: dist/FlossFinder_Linux_x86_64.tar.gz"
