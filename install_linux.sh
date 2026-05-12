#!/usr/bin/env bash
set -e

APP_NAME="DMC Substitute Finder"
BIN_NAME="flossfinder"
INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
DESKTOP_FILE="$DESKTOP_DIR/dmc-substitute-finder.desktop"

echo "Building $APP_NAME..."
cargo build --release

echo "Installing binary..."
mkdir -p "$INSTALL_DIR"
cp "target/release/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
chmod +x "$INSTALL_DIR/$BIN_NAME"

echo "Creating desktop launcher..."
mkdir -p "$DESKTOP_DIR"

cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Name=DMC Substitute Finder
Comment=Find close DMC floss substitutes
Exec=$INSTALL_DIR/$BIN_NAME
Icon=applications-graphics
Terminal=false
Type=Application
Categories=Graphics;Utility;
Keywords=DMC;floss;cross stitch;substitute;thread;
EOF

chmod +x "$DESKTOP_FILE"

if command -v kbuildsycoca6 >/dev/null 2>&1; then
    kbuildsycoca6 >/dev/null 2>&1 || true
elif command -v kbuildsycoca5 >/dev/null 2>&1; then
    kbuildsycoca5 >/dev/null 2>&1 || true
fi

echo
echo "Installed!"
echo "You can now launch it from your app menu as: $APP_NAME"
echo "Or run it from terminal with: $BIN_NAME"
