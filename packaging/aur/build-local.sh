#!/usr/bin/env bash
# Builds the Arch package from the local git checkout, without the AUR.
#
# The PKGBUILD normally downloads the tarball of the vX.Y.Z tag. While the tag
# is not published (and when validating local changes), this script creates an
# equivalent tarball with `git archive`, pins its checksum in the PKGBUILD and
# runs makepkg against that local tarball.
#
# The checksum update is left in the working tree on purpose: review it before
# committing. Pass `-si` to install the resulting package.
#
# Usage: packaging/aur/build-local.sh [-si] [makepkg options...]
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd -- "$script_dir/../.." && pwd)"
pkgbuild="$script_dir/PKGBUILD"

pkgname="$(sed -n 's/^pkgname=\(.*\)/\1/p' "$pkgbuild")"
pkgver="$(sed -n 's/^pkgver=\(.*\)/\1/p' "$pkgbuild")"
tarball="$script_dir/$pkgname-$pkgver.tar.gz"

# makepkg uses SRCDEST (the package directory by default) and skips the
# download when the file is already there.
git -C "$root" archive \
  --prefix="$pkgname-$pkgver/" \
  --format=tar.gz \
  -o "$tarball" HEAD

hash="$(sha256sum "$tarball" | cut -d' ' -f1)"
sed -i "s|^sha256sums=.*|sha256sums=('$hash')|" "$pkgbuild"

printf 'Fonte local: %s\n' "$tarball"
printf 'SHA-256: %s\n' "$hash"

cd "$script_dir"
makepkg "$@"
