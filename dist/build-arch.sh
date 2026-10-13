#!/bin/bash
# Build the Arch package of chora from this source tree.
# Usage, from the repository root, as a user that may run makepkg:
#   dist/build-arch.sh [OUTDIR]
# Needs base-devel and the depends and makedepends of dist/arch/PKGBUILD.
set -euo pipefail
root=$(pwd)
out=$(realpath -m "${1:-target/arch}")
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
sed "s/@VERSION@/$version/" dist/arch/PKGBUILD >"$work/PKGBUILD"
(cd "$work" && CHORA_SOURCE=$root makepkg --nodeps --noconfirm --force)
mkdir -p "$out"
mv "$work"/chora-*.pkg.tar.zst "$out/"
(cd "$out" && sha256sum ./*.pkg.tar.zst >SHA256SUMS)
