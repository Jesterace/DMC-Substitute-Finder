# FlossFinder GTK

A GTK4 Rust version of FlossFinder, a small DMC floss substitution finder.

## Features

- Enter a DMC colour number
- View the original DMC colour swatch
- See the closest substitute colours
- View substitute colour swatches
- Copy substitute DMC numbers
- Native GTK4 Linux interface

## Install dependencies on Linux Mint / Ubuntu

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-4-dev
```

If Rust is not installed yet:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## Run

```bash
./run_flossfinder_gtk.sh
```

Or:

```bash
cargo run --release
```

## Build Linux release archive

```bash
./build_linux_release.sh
```

The release archive will be created here:

```text
dist/FlossFinder_GTK_Linux_x86_64.tar.gz
```

## Notes

This GTK version is best for Linux. The egui version is still the better choice for Windows/macOS cross-platform builds.
