Copy/paste this on Linux Mint:

cd ~/Downloads
unzip FlossFinder_GTK_Rust_v0_1.zip
cd FlossFinder_GTK_Rust_v0_1
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-4-dev
./run_flossfinder_gtk.sh

To build a Linux release tar.gz:

./build_linux_release.sh

To upload the Linux GTK release to GitHub:

gh release upload v0.1 dist/FlossFinder_GTK_Linux_x86_64.tar.gz
