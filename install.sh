#!/bin/sh
# Installs the newest tama: no Rust needed. For 64-bit Linux and for macOS (Apple silicon or Intel).
#   curl -fsSL https://raw.githubusercontent.com/Olinkkt/tamagotchi/main/install.sh | sh
set -eu

case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) asset=tama-linux-x86_64 ;;
    Darwin-arm64) asset=tama-macos-arm64 ;;
    Darwin-x86_64) asset=tama-macos-x86_64 ;;
    *)
        echo "There is no download for this system. With Rust: cargo install --git https://github.com/Olinkkt/tamagotchi"
        exit 1
        ;;
esac
command -v curl >/dev/null || { echo "Please install curl first."; exit 1; }

dir="$HOME/.local/bin"
mkdir -p "$dir"
curl -fsSL -o "$dir/tama.new" "https://github.com/Olinkkt/tamagotchi/releases/latest/download/$asset"
chmod +x "$dir/tama.new"
mv "$dir/tama.new" "$dir/tama"
# The app-menu entry is a Linux thing.
if [ "$(uname -s)" = Linux ]; then
    "$dir/tama" --install-desktop
fi
case ":$PATH:" in
    *":$dir:"*) ;;
    *) echo "To run it from a terminal, add $dir to your PATH." ;;
esac
