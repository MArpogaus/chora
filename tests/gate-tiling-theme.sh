#!/bin/bash
# The theme of tiling (tiling-v2, items 11 and the decision Q3): the theme
# follows the colour scheme of the system and is Ink when the system says
# nothing; Paper is the light palette; one change of accent in the yaml
# recolours the focus border, the panel colours, the launcher and the lock
# screen after one reload. The system is a stand-in of busctl.
#
# Usage: tests/gate-tiling-theme.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-tiling chora-probe
fake=$(mktemp -d /tmp/gate-theme.XXXXXX)
tmp_dirs+=("$fake")
# shellcheck disable=SC2016 # the stand-in expands its own arguments
printf '#!/bin/sh\nfor a in "$@"; do case $a in color-scheme) cat "%s/scheme" 2>/dev/null; exit 0 ;; esac; done\nexit 1\n' "$fake" >"$fake/busctl"
chmod +x "$fake/busctl"
export PATH=$fake:$bin:$PATH
temp_config
session_start 1280x800 -- "$bin/chora-tiling"
grant screen.capture
theme=$XDG_RUNTIME_DIR/chora-tiling/theme
shot=$XDG_CONFIG_HOME/theme.png

[ "$(cat "$theme/scheme")" = ink ] || fail "without an answer of the system the theme is not Ink"
grep -q "@define-color surface #1a1d27" "$theme/colors.css" || fail "colors.css is not Ink"
ok "the system says nothing: Ink"

stop() { for p in "${pids[@]}"; do kill "$p" 2>/dev/null; done; }
echo "v u 2" >"$fake/scheme"
mkdir -p "$XDG_CONFIG_HOME/chora"
"$bin/chora-tiling" --print-config >"$XDG_CONFIG_HOME/chora/chora-tiling.yaml"
ctl call tiling settings.reload >/dev/null || fail "settings.reload"
sleep 1
[ "$(cat "$theme/scheme")" = paper ] || fail "the system asks for light and the theme is $(cat "$theme/scheme")"
grep -q "@define-color surface #ffffff" "$theme/colors.css" || fail "colors.css is not Paper"
grep -q "background=ffffffff" "$theme/fuzzel.ini" || fail "fuzzel.ini is not Paper"
grep -q 'stroke="#1b1e27"' "$theme/icons/keys.svg" || fail "the icons are not Paper"
ok "the system asks for light: Paper, in the panel, launcher and icons"

sed -i 's/^  theme: system/  theme: ink/' "$XDG_CONFIG_HOME/chora/chora-tiling.yaml"
ctl call tiling settings.reload >/dev/null || fail "settings.reload"
sleep 1
[ "$(cat "$theme/scheme")" = ink ] || fail "theme: ink does not beat the system"
ok "a fixed theme beats the system"

sed -i "s/^  palette: {}/  palette:\n    accent: '#f59e0b'\n    accent_2: '#f59e0b'/" "$XDG_CONFIG_HOME/chora/chora-tiling.yaml"
started=$(ctl run "$bin/chora-probe" window 0 0 0) || fail "the run of a window"
pid=$(sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p' <<<"$started")
run_pid "$pid"
ctl call tiling settings.reload >/dev/null || fail "settings.reload"
sleep 3
ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
[ "$(pixel "$shot" 12 400 | cut -d' ' -f1-3)" = "245 158 11" ] || fail "the focus border is not the new accent: $(pixel "$shot" 12 400)"
grep -q "@define-color accent #f59e0b" "$theme/colors.css" || fail "colors.css kept the old accent"
grep -q "match=f59e0bff" "$theme/fuzzel.ini" || fail "fuzzel.ini kept the old accent"
grep -q "key-hl-color=f59e0b" "$theme/swaylock.conf" || fail "swaylock.conf kept the old accent"
grep -q "regular5=f59e0b" "$theme/foot.ini" || fail "foot.ini kept the old accent"
ok "one change of accent recolours border, panel, launcher, lock and terminal"

sed -i 's/^  theme: ink/  theme: system/' "$XDG_CONFIG_HOME/chora/chora-tiling.yaml"
echo "v u 1" >"$fake/scheme"
ctl call tiling settings.reload >/dev/null || fail "settings.reload"
sleep 1
[ "$(cat "$theme/scheme")" = ink ] || fail "the system asks for dark and the theme is $(cat "$theme/scheme")"
echo "v u 2" >"$fake/scheme"
for _ in $(seq 16); do
    [ "$(cat "$theme/scheme")" = paper ] && break
    sleep 0.5
done
[ "$(cat "$theme/scheme")" = paper ] || fail "the theme did not follow the system within 8 s"
ok "the theme follows a change of the system without a reload"

export CHORA_FLAVOR_DIR=$repo/flavors/tiling
"$bin/chora-tiling" --launch look dark || fail "launch look dark"
sleep 1
[ "$(cat "$theme/scheme")" = ink ] || fail "Dark look does not give Ink while the system says light: $(cat "$theme/scheme")"
grep -q '^  theme: ink' "$XDG_CONFIG_HOME/chora/chora-tiling.yaml" || fail "Dark look is not in the file"
"$bin/chora-tiling" --launch look light || fail "launch look light"
sleep 1
[ "$(cat "$theme/scheme")" = paper ] || fail "Light look does not give Paper"
echo "v u 1" >"$fake/scheme"
"$bin/chora-tiling" --launch look system || fail "launch look system"
sleep 1
[ "$(cat "$theme/scheme")" = ink ] || fail "System look does not follow the system again"
! grep -qE '^  theme: (ink|paper)' "$XDG_CONFIG_HOME/chora/chora-tiling.yaml" || fail "System look leaves a fixed theme in the file"
ok "Light look, Dark look and System look set the theme at once and go back to the default"
echo "v u 2" >"$fake/scheme"

rm -rf "$fake/out"
"$bin/chora-tiling" --apply-theme --theme-dir "$fake/out" || fail "--apply-theme"
[ "$(cat "$fake/out/scheme")" = paper ] || fail "--apply-theme did not write the scheme of the system"
ok "chora-tiling --apply-theme writes the files"
stop
