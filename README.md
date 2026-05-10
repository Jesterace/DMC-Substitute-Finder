# FlossFinder

FlossFinder is a small DMC embroidery floss substitution finder.

Enter a DMC color number and FlossFinder will show the closest matching substitute colors. It can also use your own stash list, so it only suggests colors you actually have.

## Features

- Search by DMC color number
- View the original color swatch
- View closest substitute color swatches
- Copy substitute DMC numbers
- My Stash mode
- Supports stash quantities
- Cross-platform Rust app
- Native GTK Linux version included separately


## Running on Linux Mint

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libx11-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev libfontconfig1-dev curl
./run_flossfinder.sh
```

## My Stash mode

Paste your owned DMC colours into the stash box.

Plain codes mean quantity 1:

```text
310
666
823
```

Quantities are also supported:

```text
310 x2
666=1
823:3
B5200 x1
3812, 3810, 3847
```

Then check **My Stash only** and search for the missing colour. The results will show an **Owned** column with quantities like `x2`.

## Building Windows from Linux Mint

```bash
./build_windows_on_mint.sh
```

The Windows zip will be created at:

```text
dist/FlossFinder_Windows_x86_64.zip
```

## Building Linux release

```bash
./build_linux_release.sh
```

The Linux archive will be created at:

```text
dist/FlossFinder_Linux_x86_64.tar.gz
```

## License

This project is licensed under the MIT License.

DMC is a thread/floss brand owned by its respective owner. This project is not affiliated with or endorsed by DMC.
