#!/bin/bash
# Self-test of the tiling flavor scripts waybar.sh and wallpaper.sh: each
# makes chora-tiling of the runtime directory with mode 0700 under the
# usual umask 022. A 0755 one would be refused by chora (a directory that
# others can write), and with it the shaders, the theme files and the
# pids of the plugin. Fake choractl, chora-tiling and waybar stand in for
# the programs.
#
# Usage: tests/tiling-scripts-selftest.sh
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

mkdir -p "$work/bin"
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

mode_of() { stat -c %a "$1"; }

fresh_runtime() {
    rm -rf "${work:?}/$1"
    mkdir -m 700 "$work/$1"
}

fresh_runtime run-waybar
(
    umask 022
    PATH=$work/bin:$PATH XDG_RUNTIME_DIR=$work/run-waybar HOME=$work/home \
        timeout 20 sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >"$work/waybar.out" 2>&1
)
[ -d "$work/run-waybar/chora-tiling" ] || fail "waybar.sh made no chora-tiling: $(cat "$work/waybar.out")"
got=$(mode_of "$work/run-waybar/chora-tiling")
[ "$got" = 700 ] || fail "waybar.sh made chora-tiling with mode $got, not 700"

fresh_runtime run-wallpaper
(
    umask 022
    PATH=$work/bin:$PATH XDG_RUNTIME_DIR=$work/run-wallpaper \
        sh "$flavor/wallpaper.sh" >"$work/wallpaper.out" 2>&1 &
    script=$!
    for _ in 1 2 3 4 5 6 7 8 9 10; do
        [ -f "$work/run-wallpaper/chora-tiling/wallpaper.pid" ] && break
        sleep 0.2
    done
    kill -TERM "$script" 2>/dev/null
    wait "$script" 2>/dev/null
)
[ -d "$work/run-wallpaper/chora-tiling" ] || fail "wallpaper.sh made no chora-tiling: $(cat "$work/wallpaper.out")"
got=$(mode_of "$work/run-wallpaper/chora-tiling")
[ "$got" = 700 ] || fail "wallpaper.sh made chora-tiling with mode $got, not 700"

if [ "$failures" -gt 0 ]; then
    exit 1
fi
echo "tiling-scripts-selftest: ok"
