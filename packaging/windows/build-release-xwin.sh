#!/usr/bin/env bash
# Cross-builds the standalone Windows GUI executable from Linux with cargo-xwin.
#
# The executable uses the static MSVC CRT, so it does not need the Visual C++
# Redistributable. The GUI assets are embedded in the binary.
#
# Requirements: rustup with the x86_64-pc-windows-msvc target, cargo-xwin
# (`cargo install cargo-xwin` or a release binary) and the clang/lld/llvm
# packages. cargo-xwin downloads the MSVC CRT and Windows SDK headers on the
# first build, so that build needs network access.
#
# Output: dist/melobudsnext-gui.exe and dist/melobudsnext-gui.exe.sha256.
#
# Usage: packaging/windows/build-release-xwin.sh
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd -- "$script_dir/../.." && pwd)"
target="x86_64-pc-windows-msvc"

cd "$root"

if ! rustup target list --installed | grep -qx "$target"; then
  rustup target add "$target"
fi

export RUSTFLAGS="${RUSTFLAGS:-} -C target-feature=+crt-static"
cargo xwin build --release --locked --target "$target" --bin melobudsnext-gui

install -Dm755 "target/$target/release/melobudsnext-gui.exe" \
  "$root/dist/melobudsnext-gui.exe"

hash="$(sha256sum "$root/dist/melobudsnext-gui.exe" | cut -d' ' -f1)"
printf '%s  melobudsnext-gui.exe\n' "$hash" \
  > "$root/dist/melobudsnext-gui.exe.sha256"

printf 'Gerado: %s\n' "$root/dist/melobudsnext-gui.exe"
printf 'SHA-256: %s\n' "$hash"
