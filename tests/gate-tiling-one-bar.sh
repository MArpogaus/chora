#!/bin/bash
# One top bar (ti-47): a tiling session has exactly one waybar on the config
# of its runtime directory, however it starts, and none is left when the
# session is stopped. Three ways in:
#   the product: chora-tiling starts the panel itself (no --no-companions);
#   the unit too: the panel unit (chora-tiling-waybar.service) runs
#     waybar.sh beside the panel that chora-tiling started, as a login
#     session whose target wants both does;
#   the persona harness: tests/persona/session.sh tiling, which starts the
#     companions of the flavor itself and gives the core CHORA_FLAVOR_DIR
#     (a4badd8e), so that the core finds waybar.sh as well.
# A second bar is not a count of one that settles: the cell samples for 10 s
# after the first bar is up, and fails on any sample of two.
#
# Functions: one_bar NAME RUNTIME (one bar comes up and stays one for 10 s),
# none_left NAME RUNTIME (no waybar of that runtime after the stop),
# stop_session (ends the headless session that session_start made last).
# The cells run in the order product, unit (the unit starts after
# chora-tiling.service), persona.
#
# The lock itself (rtr-1, rtr-3), the cell "lock", with no session (a fake
# waybar says whether descriptor 9 of the lock is open in it; the second
# start has a PATH with every tool of the script but flock): waybar
# must not inherit the descriptor of panel.lock (a child of waybar that
# outlives it would keep the lock and the next start would end with no bar),
# and a machine without flock says so on stderr once instead of skipping the
# lock silently.
#
# The cell "fork", with no session: waybar forks a shell for each of its custom
# modules, and the child keeps the name waybar until it execs (a loaded host
# stretches that to milliseconds, sh-41: "3 waybar processes on one config" with
# one bar). bars() counts the waybar processes whose parent is no waybar: a
# fake waybar with a forked child that keeps its name counts as one bar, and
# a second waybar beside it (a parent that is no waybar) makes two.
#
# Usage: tests/gate-tiling-one-bar.sh [lock|fork|product|unit|persona ...]
#   BIN: the binaries (default target/debug); the cells to run (default all)
set -u
. "$(dirname "$0")/gate-lib.sh"
need waybar
need jq
need setsid
need dbus-run-session
need dbus-daemon
need_bins chora choractl chora-tiling chora-probe chora-assistant
flavor=$repo/flavors/tiling

bars() {
    local pid ppid n=0
    for pid in $(XDG_RUNTIME_DIR=$1 session_pids -x waybar); do
        ppid=$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d ' ')
        [ -n "$ppid" ] || continue
        [ "$(ps -o comm= -p "$ppid" 2>/dev/null)" = waybar ] || n=$((n + 1))
    done
    echo "$n"
}

one_bar() {
    local name=$1 runtime=$2 n max=0
    until_ok "[ \"\$(bars $runtime)\" -ge 1 ]" 80 || fail "$name: no bar came up on $runtime"
    for _ in $(seq 20); do
        n=$(bars "$runtime")
        [ "$n" -le "$max" ] || max=$n
        pace 0.5
    done
    [ "$max" = 1 ] || fail "$name: $max waybar processes on one config: $(XDG_RUNTIME_DIR=$runtime session_pids -x waybar | tr '\n' ' ')"
    ok "$name: one waybar ($(bars "$runtime")) for 10 s"
}

none_left() {
    until_ok "[ \"\$(bars $2)\" = 0 ]" 40 || fail "$1: $(bars "$2") waybar processes left after the stop"
    ok "$1: none left after the stop"
}

stop_session() {
    local pid=${pids[-1]}
    kill -TERM "$pid" 2>/dev/null
    until_ok "! kill -0 $pid 2>/dev/null" 80 || kill -KILL "$pid" 2>/dev/null
    wait "$pid" 2>/dev/null
    pids=()
}

cells=("$@")
[ ${#cells[@]} -gt 0 ] || cells=(lock fork product unit persona)
want() { [[ " ${cells[*]} " == *" $1 "* ]]; }

temp_config

lock_cell() {
    local d fakebin out tool
    d=$(mktemp -d /tmp/gate-one-bar-lock.XXXXXX)
    tmp_dirs+=("$d")
    fakebin=$d/bin
    mkdir -p "$fakebin" "$d/run"
    chmod 700 "$d/run"
    cat >"$fakebin/waybar" <<'FAKE'
#!/bin/sh
if [ -e "/proc/$$/fd/9" ]; then echo open >"$LOCK_OUT"; else echo closed >"$LOCK_OUT"; fi
exec tail -f /dev/null
FAKE
    cat >"$fakebin/choractl" <<'FAKE'
#!/bin/sh
case $1 in
call) echo '{"outputs": [{"name": "X"}]}' ;;
outputs) echo '[{"name": "X"}]' ;;
esac
FAKE
    chmod +x "$fakebin/waybar" "$fakebin/choractl"
    out=$d/fd9
    LOCK_OUT=$out XDG_RUNTIME_DIR=$d/run PATH=$fakebin:$PATH setsid sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >"$d/log" 2>&1 &
    run_pid $!
    until_ok "[ -s $out ]" 40 || fail "lock: the fake waybar did not start: $(cat "$d/log")"
    [ "$(cat "$out")" = closed ] || fail "lock: waybar inherits the descriptor of panel.lock (9), so a child that outlives it keeps the lock (rtr-1)"
    ok "lock: waybar does not inherit the lock descriptor"
    mkdir -p "$d/noflock" "$d/run2"
    chmod 700 "$d/run2"
    for tool in sh jq id stat mkdir cp cat dirname kill sed tail sleep; do
        ln -s "$(command -v "$tool")" "$d/noflock/$tool"
    done
    cp "$fakebin/waybar" "$fakebin/choractl" "$d/noflock/"
    rm -f "$out"
    LOCK_OUT=$out XDG_RUNTIME_DIR=$d/run2 setsid env PATH="$d/noflock" "$d/noflock/sh" "$flavor/waybar/waybar.sh" "$flavor/waybar" >"$d/log2" 2>&1 &
    run_pid $!
    until_ok "[ -s $out ]" 40 || fail "lock: with no flock the panel did not start: $(cat "$d/log2")"
    [ "$(grep -c flock "$d/log2")" = 1 ] || fail "lock: with no flock the lock is skipped silently, not with one line on stderr (rtr-3): $(cat "$d/log2")"
    ok "lock: without flock one line on stderr says the lock is skipped"
}

if want lock; then
    lock_cell
fi

fork_cell() {
    local d fakebin
    d=$(mktemp -d /tmp/gate-one-bar-fork.XXXXXX)
    tmp_dirs+=("$d")
    fakebin=$d/bin
    mkdir -p "$fakebin" "$d/run"
    chmod 700 "$d/run"
    cat >"$fakebin/waybar" <<'FAKE'
#!/bin/sh
( tail -f /dev/null; true ) &
tail -f /dev/null
FAKE
    chmod +x "$fakebin/waybar"
    XDG_RUNTIME_DIR=$d/run setsid "$fakebin/waybar" >"$d/log" 2>&1 &
    run_pid $!
    until_ok "[ \"\$(XDG_RUNTIME_DIR=$d/run session_pids -x waybar | wc -l)\" -ge 2 ]" 40 ||
        fail "fork: the fake waybar did not fork a child named waybar: $(XDG_RUNTIME_DIR=$d/run session_pids -x waybar | tr '\n' ' ')"
    [ "$(bars "$d/run")" = 1 ] || fail "fork: a waybar with a forked child of its name counts as $(bars "$d/run") bars, not 1 (sh-41)"
    XDG_RUNTIME_DIR=$d/run setsid "$fakebin/waybar" >"$d/log2" 2>&1 &
    run_pid $!
    until_ok "[ \"\$(bars $d/run)\" -ge 2 ]" 40 || fail "fork: a second waybar that is no child of a waybar does not count: $(bars "$d/run")"
    ok "fork: a forked child named waybar is no second bar, a second waybar is"
}

if want fork; then
    fork_cell
fi

if want product || want unit; then
    session_start 1280x800 -- "$bin/chora-tiling"
    runtime=$XDG_RUNTIME_DIR
    export CHORA_FLAVOR_DIR=$flavor
    one_bar "chora-tiling alone" "$runtime"
    if want unit; then
        PATH=$bin:$PATH setsid dbus-run-session -- sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >/dev/null 2>&1 &
        run_pid $!
        pace 3
        one_bar "chora-tiling and the panel unit" "$runtime"
    fi
    stop_session
    none_left "product session" "$runtime"
    unset CHORA_FLAVOR_DIR
fi

if want persona; then
    pdir=$(mktemp -d /tmp/gate-one-bar.XXXXXX)
    tmp_dirs+=("$pdir" "$pdir.out")
    PERSONA_DIR=$pdir CHORA_BIN=$bin "$repo/tests/persona/session.sh" tiling >"$pdir.out" 2>&1 &
    pids+=($!)
    until_ok "grep -q '^PERSONA_ENV=' $pdir.out" 240 || fail "persona session did not start: $(tail -n 3 "$pdir.out")"
    pruntime=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$pdir/session.env")
    [ -d "$pruntime" ] || fail "no runtime directory of the persona session in $pdir/session.env"
    one_bar "persona session.sh tiling" "$pruntime"
    "$repo/tests/persona/session.sh" --stop "$pdir/session.env" >/dev/null 2>&1
    none_left "persona session" "$pruntime"
fi
