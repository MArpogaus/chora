#!/bin/bash
# Self-test of the tiling flavor scripts waybar.sh and wallpaper.sh: each
# makes chora-tiling of the runtime directory with mode 0700 under the
# usual umask 022. A 0755 one would be refused by chora (a directory that
# others can write), and with it the shaders, the theme files and the
# pids of the plugin. Fake choractl, chora-tiling and waybar stand in for
# the programs.
#
# It also runs the selftest of the clock bound (tests/clock-width.py, ti-37).
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

mkdir -p "$work/tmp"
touch "$work/stamp"
(
    umask 022
    env -u XDG_RUNTIME_DIR PATH="$work/bin:$PATH" HOME="$work/home" TMPDIR="$work/tmp" \
        timeout 20 sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >"$work/nofallback-waybar.out" 2>&1
    env -u XDG_RUNTIME_DIR PATH="$work/bin:$PATH" TMPDIR="$work/tmp" \
        sh "$flavor/wallpaper.sh" >"$work/nofallback-wallpaper.out" 2>&1 &
    script=$!
    sleep 1
    kill -TERM "$script" 2>/dev/null
    wait "$script" 2>/dev/null
)
fixed=$(find /tmp -maxdepth 1 -name 'chora-tiling*' -newer "$work/stamp" 2>/dev/null)
[ -z "$fixed" ] || fail "without XDG_RUNTIME_DIR the scripts used a fixed name in /tmp: $fixed"
unset fixed
made=$(find "$work/tmp" -mindepth 2 -maxdepth 2 -name chora-tiling -type d)
[ -n "$made" ] || fail "without XDG_RUNTIME_DIR the scripts made no chora-tiling in a mktemp dir"
for dir in $made; do
    got=$(mode_of "$(dirname "$dir")")
    [ "$got" = 700 ] || fail "the mktemp dir has mode $got, not 700"
done
find /tmp -maxdepth 1 -name 'chora-tiling*' -newer "$work/stamp" -exec rm -rf {} + 2>/dev/null

inter=${SELFTEST_INTER:-/usr/share/fonts/opentype/inter/Inter-Medium.otf}
if ! command -v python3 >/dev/null; then
    echo "skip: clock selftest, python3 is missing"
elif ! python3 -I -c 'import PIL' 2>/dev/null; then
    echo "skip: clock selftest, the Python module PIL is missing"
elif [ ! -f "$inter" ]; then
    echo "skip: clock selftest, the font Inter is missing ($inter)"
else
    python3 -I "$repo/tests/clock-width.py" --selftest >"$work/cw.log" 2>&1 || fail "clock-width selftest: $(tail -3 "$work/cw.log")"
fi

if [ -z "${SELFTEST_INNER:-}" ]; then
    skipped=$(SELFTEST_INNER=1 SELFTEST_INTER=/nonexistent/Inter-Medium.otf bash "$0" 2>&1)
    case $skipped in
    *"skip: clock selftest"*Inter*) ;;
    *) fail "without Inter the clock selftest did not print a skip line naming Inter (TF-1)" ;;
    esac
    unset skipped
fi

if [ "$failures" -gt 0 ]; then
    exit 1
fi
echo "tiling-scripts-selftest: ok"
