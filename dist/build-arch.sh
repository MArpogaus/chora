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
# Every file of the RPM is in the package too.
package=$(ls "$work"/chora-*.pkg.tar.zst)
for path in usr/bin/chora usr/libexec/chora/chora-xwayland \
    usr/share/wayland-sessions/chora.desktop usr/share/chora/flavors/xp/theme/winxp \
    usr/share/icons/ModernXP/index.theme usr/share/licenses/chora/LICENSE; do
    bsdtar -tf "$package" | grep -qx "$path/\?" || { echo "missing: $path" >&2; exit 1; }
done
want=$(find flavors/xp/theme \( -type f -o -type l \) | wc -l)
have=$(bsdtar -tf "$package" | grep -cE '^usr/share/(chora/flavors/xp/theme|icons/ModernXP)/.*[^/]$')
[ "$have" = "$want" ] || { echo "the XP theme has $have of $want files" >&2; exit 1; }
mkdir -p "$out"
mv "$package" "$out/"
(cd "$out" && sha256sum ./*.pkg.tar.zst >SHA256SUMS)
