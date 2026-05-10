#!/usr/bin/env bash
set -e
cargo build --release
mkdir -p dist/linux
cp target/release/flossfinder-gtk dist/linux/FlossFinder-GTK
chmod +x dist/linux/FlossFinder-GTK
cat > dist/linux/README_LINUX.txt <<'README'
FlossFinder GTK - Linux Version

To run:

./FlossFinder-GTK

If it does not launch, make sure it is executable:

chmod +x FlossFinder-GTK
./FlossFinder-GTK
README
cd dist
tar -czvf FlossFinder_GTK_Linux_x86_64.tar.gz linux
cd ..
echo "Built: dist/FlossFinder_GTK_Linux_x86_64.tar.gz"
