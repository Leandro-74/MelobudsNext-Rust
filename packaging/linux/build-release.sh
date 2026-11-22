#!/usr/bin/env bash
# Builds the Linux release artifact used by the release checklist.
#
# Usage: packaging/linux/build-release.sh [VERSION] [LABEL]
#   VERSION defaults to the version in Cargo.toml.
#   LABEL defaults to x86_64-unknown-linux-gnu and is only used in the file
#   name; the build itself targets the current machine.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd -- "$script_dir/../.." && pwd)"

version="${1:-$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$root/Cargo.toml" | head -n 1)}"
label="${2:-x86_64-unknown-linux-gnu}"
name="melobudsnext-$version-$label"
stage="$root/dist/$name"
archive="$root/dist/$name.tar.gz"

cd "$root"
cargo build --release --locked

rm -rf "$stage"
install -Dm755 target/release/melobudsnext "$stage/melobudsnext"
install -Dm755 target/release/melobudsnext-gui "$stage/melobudsnext-gui"
install -Dm644 README.md "$stage/README.md"
install -Dm644 PROTOCOLO_MelobudsNext.md "$stage/PROTOCOLO_MelobudsNext.md"
install -Dm644 LICENSE "$stage/LICENSE"
install -Dm644 packaging/melobudsnext.desktop "$stage/melobudsnext.desktop"
install -Dm644 assets/logo.svg "$stage/melobudsnext.svg"

tar -C "$root/dist" -czf "$archive" "$name"
hash="$(sha256sum "$archive" | cut -d' ' -f1)"
printf '%s  %s\n' "$hash" "$(basename "$archive")" > "$archive.sha256"

printf 'Gerado: %s\n' "$archive"
printf 'SHA-256: %s\n' "$hash"
