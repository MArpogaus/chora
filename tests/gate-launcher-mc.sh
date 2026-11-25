#!/bin/bash
# The launcher of mc opens, in the light and in the dark theme, by a click on
# Programs in the column and by Super+Space, with the hands of a persona
# (tests/persona/act.py), and closes by Escape (RC shared bar: a visible way to
# start a program; rtr1-3: in tour v9 a click on Programs opened no launcher).
#
# mc starts the launcher as `chora-mc --launch fuzzel --config FILE`, a fuzzel
# config that it writes from the colours of its theme (rpx1-10). A fuzzel that
# does not know a key of that file (1.9.2 has no `prompt`) exits at once, and
# the launcher was a process that showed nothing. Each theme therefore runs
# with two fuzzel: the one of this machine (at least 1.11, which need_fuzzel
# checks) and a stand-in that answers `--version` with 1.9.2 and rejects the
# config of mc as 1.9.2 does, with the err line of fuzzel, and otherwise runs
# the fuzzel of this machine. In all four sessions the launcher must be a fuzzel
# that is on the screen: a process, and pixels in the box where it stands,
# after the click on Programs and after Super+Space.
#
# Usage: tests/gate-launcher-mc.sh   BIN: directory with the binaries
#   CELLS  a regex of the cells (light, dark; default all)
# shellcheck disable=SC2016
set -u
. "$(dirname "$0")/gate-lib.sh"
need python3
need_fuzzel 1.11
need jq
need_bins chora choractl chora-mc chora-assistant chora-probe

real_fuzzel=$(command -v fuzzel)

act() { PERSONA_ENV=$dir/session.env python3 -I "$repo/tests/persona/act.py" "$@"; }
admin() {
    CHORA_SOCKET=$(sed -n 's/^CHORA_SOCKET=//p' "$dir/session.env") \
        XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$dir/session.env") "$bin/choractl" "$@"
}
launcher_up() {
    XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$dir/session.env") session_pids -x fuzzel >/dev/null
}
launcher_down() { ! launcher_up; }
spid=
dir=
stop_session() {
    [ -z "$spid" ] || {
        "$repo/tests/persona/session.sh" --stop "$dir/session.env" >/dev/null 2>&1
        wait "$spid" 2>/dev/null
        spid=
    }
}
trap 'status=$?; stop_session; gate_cleanup $status' EXIT

old_fuzzel() {
    mkdir -p "$1"
    cat >"$1/fuzzel" <<EOF
#!/bin/sh
case " \$* " in
*" --version "*)
    echo "fuzzel version: 1.9.2 +cairo +png +svg(nanosvg) -assertions"
    exit 0
    ;;
esac
prev=
for arg in "\$@"; do
    if [ "\$prev" = --config ] && grep -q '^prompt=' "\$arg"; then
        echo "info: config.c:1475: loading configuration from \$arg" >&2
        echo " err: config.c:866: \$arg:5: [colors].prompt: not a valid option: prompt" >&2
        exit 1
    fi
    prev=\$arg
done
exec "$real_fuzzel" "\$@"
EOF
    chmod +x "$1/fuzzel"
}

programs_door() {
    admin query "owner:mc kind:rect" | jq -r '[.[] | select(.visible and (.id | endswith("-doors-programs-hit"))
        and (.rects | length) > 0) | .rects | to_entries[0].value][0] // empty | "\(.[0] + .[2] / 2 | floor) \(.[1] + .[3] / 2 | floor)"'
}

on_screen() {
    local after=$dir/after.png n
    admin capture "$after" >/dev/null || return 1
    n=$(python3 -I "$repo/tests/capture-diff.py" "$1" "$after" 440 220 840 580)
    [ "$n" != 0 ] && [ "$n" != size ]
}

shows() {
    local what=$1 how=$2 base=$dir/base.png
    admin capture "$base" >/dev/null || fail "$what: capture before"
    case $how in
    mouse)
        local door
        door=$(programs_door)
        [ -n "$door" ] || fail "$what: no Programs door in the column"
        # shellcheck disable=SC2086
        act click $door >/dev/null || fail "$what: click on Programs"
        ;;
    keys) act key Super+space >/dev/null || fail "$what: Super+space" ;;
    esac
    until_ok launcher_up 80 || fail "$what: no launcher after $how (a fuzzel process; the log of the session says: $(grep -ah 'launcher\|err:' "$dir.err" "$dir/chora.log" 2>/dev/null | tail -3 | tr '\n' '|'))"
    until_ok 'on_screen "$base"' 40 || fail "$what: the launcher process runs but nothing is drawn where it stands after $how"
    act key Escape >/dev/null || fail "$what: Escape"
    until_ok launcher_down 40 || fail "$what: the launcher stays open after Escape"
}

cell() {
    local theme=$1 fuzzel=$2 what
    what="mc $theme, $fuzzel fuzzel"
    dir=$(mktemp -d /tmp/launcher-mc.XXXXXX)
    tmp_dirs+=("$dir")
    mkdir -p "$dir/data/applications" "$dir/config/chora"
    printf 'theme: %s\ntest_accept_injected_keys: true\n' "$theme" >"$dir/config/chora/chora-mc.yaml"
    printf '[Desktop Entry]\nType=Application\nName=Aa Alpha\nExec=foot -a alpha\n' >"$dir/data/applications/alpha.desktop"
    local front=$PATH
    if [ "$fuzzel" = old ]; then
        old_fuzzel "$dir/old"
        front=$dir/old:$PATH
    fi
    (
        export XDG_DATA_DIRS=$dir/none
        export XDG_CACHE_HOME=$dir/cache
        export PATH=$front
        PERSONA_DIR=$dir CHORA_BIN=$bin CHORA_TEST_FLAGS=--test-injected-press-is-real exec "$repo/tests/persona/session.sh" mc >"$dir.out" 2>"$dir.err"
    ) &
    spid=$!
    until_ok "grep -q '^PERSONA_ENV=' '$dir.out'" 200 || fail "$what: the session did not start: $(cat "$dir.err")"
    admin grant choractl screen.capture >/dev/null || fail "$what: grant of the capture"
    until_ok '[ -n "$(programs_door)" ]' 120 || fail "$what: the column shows no Programs door"
    pace 1
    shows "$what" mouse
    pace 1
    shows "$what" keys
    ok "$what: the launcher opens by a click on Programs and by Super+Space, and closes by Escape"
    stop_session
}

for theme in light dark; do
    for fuzzel in new old; do
        [[ "$theme" =~ ${CELLS:-.} ]] || continue
        cell "$theme" "$fuzzel"
    done
done
