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
# pkgver takes no hyphen; pacman sorts 0.6.0rc.1 before 0.6.0.
sed "s/@VERSION@/${version//-/}/" dist/arch/PKGBUILD >"$work/PKGBUILD"
(cd "$work" && CHORA_SOURCE=$root makepkg --nodeps --noconfirm --force)
# Every file of the RPM is in the package too, but one:
# /etc/fonts/conf.d/09-chora-tex-gyre.conf. Fedora's texlive-tex-gyre puts
# the fonts in a TeX directory that fontconfig does not read, so the RPM
# adds it; Arch's tex-gyre-fonts installs them to /usr/share/fonts, where
# fontconfig finds them without a snippet.
package=$(ls "$work"/chora-*.pkg.tar.zst)
for path in usr/bin/chora usr/libexec/chora/chora-xwayland \
    usr/bin/chora-mc usr/share/chora/mc/collectors/agenda.sh \
    usr/bin/chora-voice usr/lib/systemd/user/chora-voice.service \
    usr/share/chora/voice/quadlets/gufo-llm.container \
    usr/bin/chora-construct usr/share/chora/flavors/construct/chora-construct.target \
    usr/bin/chora-notify usr/bin/chora-assistant \
    usr/share/chora/notify/assets/values.yaml usr/share/chora/huds/neutral/hud.yaml \
    usr/share/chora/flavors/xp/chora-xp-notify.service \
    usr/share/chora/flavors/xp/chora-xp-assistant.service \
    usr/share/chora/flavors/tiling/chora-tiling-assistant.service \
    usr/share/chora/flavors/construct/chora-construct-assistant.service \
    usr/share/chora/flavors/xp/assets/notify/values.yaml \
    usr/share/chora/flavors/xp/assets/notify/fonts \
    usr/share/chora/huds/xp/hud.yaml usr/share/chora/huds/xp/paperclip.svg \
    usr/share/chora/huds/xp/fonts/tahoma.ttf \
    usr/share/chora/huds/xp/fonts/LICENSE.Tahoma-Wine.LGPL-2.1 \
    usr/share/chora/huds/mc/hud.yaml usr/share/chora/huds/construct/hud.yaml \
    usr/share/wayland-sessions/chora.desktop usr/share/wayland-sessions/chora-uwsm.desktop \
    usr/share/wayland-sessions/chora-construct.desktop \
    etc/xdg/uwsm/env-chora usr/share/chora/flavors/tiling/waybar/config.jsonc \
    usr/share/chora/flavors/shared/power.sh usr/share/chora/flavors/shared/screenshot.sh \
    usr/share/chora/flavors/shared/applications/chora-chat.desktop \
    usr/share/xdg-desktop-portal/chora-portals.conf etc/xdg/xdg-desktop-portal-wlr/chora \
    usr/share/chora/flavors/xp/theme/winxp \
    usr/share/icons/ModernXP/index.theme usr/share/licenses/chora/LICENSE; do
    bsdtar -tf "$package" | grep -qx "$path/\?" || { echo "missing: $path" >&2; exit 1; }
done
want=$(find flavors/xp/theme \( -type f -o -type l \) | wc -l)
have=$(bsdtar -tf "$package" | grep -cE '^usr/share/(chora/flavors/xp/theme|icons/ModernXP)/.*[^/]$')
[ "$have" = "$want" ] || { echo "the XP theme has $have of $want files" >&2; exit 1; }
mkdir -p "$out"
mv "$package" "$out/"
(cd "$out" && sha256sum ./*.pkg.tar.zst >SHA256SUMS)
