#!/bin/bash
# Self-test of the runtime directory rule of the flavor scripts
# (flavors/shared/runtime.sh): without XDG_RUNTIME_DIR, wallpaper.sh and
# waybar.sh both use ${TMPDIR:-/tmp}/chora-UID, so that a second run finds
# the pid of the first; a directory of another owner or one that is not
# mode 700 is refused with one line on stderr and exit 1.
#
# Usage: tests/runtime-dir-selftest.sh
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
flavor=$repo/flavors/tiling
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0
fail() {
    echo "FAIL: $*"
    failures=$((failures + 1))
}

mkdir -p "$work/bin" "$work/tmp" "$work/home"
cat >"$work/bin/choractl" <<'FAKE'
#!/bin/sh
case $1 in
call) echo '{"outputs":[{"name":"A"}]}' ;;
outputs) echo '[{"name":"A"}]' ;;
esac
FAKE
printf '#!/bin/sh\nexit 0\n' >"$work/bin/chora-tiling"
printf '#!/bin/sh\nexit 0\n' >"$work/bin/waybar"
chmod +x "$work/bin/"*
want=$work/tmp/chora-$(id -u)

run() {
    local name=$1 pidfile=$2
    shift 2
    (
        unset XDG_RUNTIME_DIR WAYLAND_DISPLAY
        PATH=$work/bin:$PATH TMPDIR=$work/tmp HOME=$work/home \
            timeout 20 sh "$@" >"$work/$name.out" 2>&1 &
        script=$!
        for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
            [ -f "$pidfile" ] && break
            kill -0 "$script" 2>/dev/null || break
            sleep 0.2
        done
        kill "$script" 2>/dev/null
        wait "$script" 2>/dev/null
    )
}

command -v jq >/dev/null 2>&1 || echo "note: jq missing, waybar.sh is not covered"
for round in 1 2; do
    run "wallpaper$round" "$want/chora-tiling/wallpaper.pid" "$flavor/wallpaper.sh"
    [ -f "$want/chora-tiling/wallpaper.pid" ] || fail "wallpaper.sh run $round did not use $want: $(cat "$work/wallpaper$round.out")"
    rm -f "$want/chora-tiling/wallpaper.pid"
done
if command -v jq >/dev/null 2>&1; then
    for round in 1 2; do
        run "waybar$round" "$want/chora-tiling/panel.pid" "$flavor/waybar/waybar.sh" "$flavor/waybar"
        [ -f "$want/chora-tiling/panel.pid" ] || fail "waybar.sh run $round did not use $want: $(cat "$work/waybar$round.out")"
        rm -f "$want/chora-tiling/panel.pid"
    done
fi
[ "$(stat -c %a "$want" 2>/dev/null)" = 700 ] || fail "$want is not mode 700"
[ "$(find "$work/tmp" -mindepth 1 -maxdepth 1 | wc -l)" = 1 ] || fail "more than one directory in TMPDIR: $(ls "$work/tmp")"

refuse() {
    local what=$1 script=$2
    shift 2
    local out rc
    out=$(unset XDG_RUNTIME_DIR WAYLAND_DISPLAY; PATH=$work/bin:$PATH TMPDIR=$work/tmp HOME=$work/home timeout 20 sh "$script" "$@" 2>&1 >/dev/null)
    rc=$?
    [ "$rc" = 1 ] || fail "$what: exit $rc, not 1"
    [ "$(printf '%s\n' "$out" | wc -l)" = 1 ] && [ -n "$out" ] || fail "$what: not one line on stderr: $out"
}
[ -d "$want" ] || mkdir -m 700 "$want"
chmod 755 "$want"
refuse "mode 755 (wallpaper)" "$flavor/wallpaper.sh"
command -v jq >/dev/null 2>&1 && refuse "mode 755 (waybar)" "$flavor/waybar/waybar.sh" "$flavor/waybar"
chmod 700 "$want"
if [ "$(id -u)" = 0 ] && chown 65534 "$want" 2>/dev/null; then
    refuse "foreign owner (wallpaper)" "$flavor/wallpaper.sh"
    command -v jq >/dev/null 2>&1 && refuse "foreign owner (waybar)" "$flavor/waybar/waybar.sh" "$flavor/waybar"
else
    echo "note: cannot chown, foreign owner not covered"
fi

[ "$failures" = 0 ] && echo "runtime-dir-selftest: ok"
exit $((failures > 0))
