DMC Substitute Finder Rust v0.1
================================

This is a separate Rust rewrite of the Python/Tkinter DMC Substitute Finder.

What it does:
- Enter a DMC colour number/code.
- Shows the original colour swatch.
- Shows closest substitution colours ranked by visual colour distance.
- Shows swatches for every substitute.
- Lets you copy a substitute DMC code.

Mint setup / first run:

1. Open Terminal in the extracted folder.
2. Install build dependencies:

sudo apt update
sudo apt install -y build-essential pkg-config libx11-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev libfontconfig1-dev curl

3. Install Rust using rustup if you do not already have cargo:

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

4. Run the app:

./run_dmc_substitute_rust.sh

Notes:
- The first run compiles the app, so it can take longer.
- Later runs launch much faster.
- The DMC colour list is embedded into the Rust app at compile time from src/dmc_colors.csv.
