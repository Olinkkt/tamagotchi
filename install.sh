#!/bin/sh
# Installs the newest tama for 64-bit Linux: no Rust needed.
#   curl -fsSL https://raw.githubusercontent.com/Olinkkt/tamagotchi/master/install.sh | sh
set -eu

if [ "$(uname -s)" != Linux ] || [ "$(uname -m)" != x86_64 ]; then
    echo "The download is for 64-bit Linux only. With Rust: cargo install --git https://github.com/Olinkkt/tamagotchi"
    exit 1
fi
command -v curl >/dev/null || { echo "Please install curl first."; exit 1; }

dir="$HOME/.local/bin"
mkdir -p "$dir"
curl -fsSL -o "$dir/tama.new" https://github.com/Olinkkt/tamagotchi/releases/latest/download/tama-linux-x86_64
chmod +x "$dir/tama.new"
mv "$dir/tama.new" "$dir/tama"
"$dir/tama" --install-desktop
case ":$PATH:" in
    *":$dir:"*) ;;
    *) echo "To run it from a terminal, add $dir to your PATH." ;;
esac
