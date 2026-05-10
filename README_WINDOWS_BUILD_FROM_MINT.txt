DMC Substitute Finder Rust - Build Windows Version From Linux Mint

Run this from the project folder:

./build_windows_on_mint.sh

What it does:
- Installs mingw-w64, the Windows cross-compile toolchain.
- Adds the Rust x86_64-pc-windows-gnu target.
- Builds a 64-bit Windows .exe.
- Copies it to dist/windows/DMC_Substitute_Finder.exe.
- Creates dist/DMC_Substitute_Finder_Windows.zip for sharing.

If the build succeeds, copy this file to your Windows machine:

dist/DMC_Substitute_Finder_Windows.zip

Then unzip it and double-click:

DMC_Substitute_Finder.exe

Note:
Windows may show a SmartScreen warning because the app is homemade and not code-signed.
