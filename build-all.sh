#!/usr/bin/env bash
# builds the linux binary + the windows .exe and drops both in ./dist
# windows build needs mingw: sudo pacman -S mingw-w64-gcc
set -e

cd "$(dirname "$0")"
mkdir -p dist

echo ">> linux"
cargo build --release
cp target/release/dexor dist/dexor-linux-x86_64

echo ">> windows"
if ! command -v x86_64-w64-mingw32-gcc >/dev/null; then
  echo "!! mingw not found. run: sudo pacman -S mingw-w64-gcc"
  echo "   (linux binary is still in dist/)"
  exit 1
fi
rustup target add x86_64-pc-windows-gnu >/dev/null 2>&1 || true
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/dexor.exe dist/dexor-windows-x86_64.exe

# bundle the quick-start next to the binaries (kept in version control)
cp QUICKSTART.txt dist/ 2>/dev/null || true

echo ">> done. files in ./dist:"
ls -lh dist
