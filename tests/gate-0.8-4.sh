#!/bin/bash
# Gate check 4 of 0.8: injected text "Grüße äöüß" appears unchanged in
# foot on a US layout. This needs a nested session on sway with foot;
# the in-process proof of the commit path is the unit test
# a_text_goes_as_one_commit_to_the_text_input_of_the_focus in
# crates/chora/src/inject.rs.
#
# Usage: tests/gate-0.8-4.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need sway
need foot
for what in chora choractl chora-tiling; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
out=$(mktemp -d /tmp/gate4.XXXXXX)
runtime=$(mktemp -d /tmp/gate4-rt.XXXXXX)
trap 'kill "${pids[@]}" 2>/dev/null; rm -rf "$out" "$runtime"' EXIT
export XDG_RUNTIME_DIR=$runtime
printf 'output HEADLESS-1 resolution 1280x800\ndefault_border none\n' >"$out/sway.conf"
WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1 \
    sway -c "$out/sway.conf" >"$out/sway.log" 2>&1 &
pids+=($!)
for _ in $(seq 50); do
    test -S "$XDG_RUNTIME_DIR/wayland-1" && break
    sleep 0.2
done
test -S "$XDG_RUNTIME_DIR/wayland-1" || fail "no host sway: $(cat "$out/sway.log")"
export WAYLAND_DISPLAY=wayland-1
# chora-tiling gives foot the focus, which the text input needs.
"$bin/chora" -- "$bin/chora-tiling" >"$out/chora.log" 2>&1 &
pids+=($!)
for _ in $(seq 50); do
    compgen -G "$XDG_RUNTIME_DIR/chora/wayland-*[0-9].sock" >/dev/null && break
    sleep 0.2
done
CHORA_SOCKET=$(compgen -G "$XDG_RUNTIME_DIR/chora/wayland-*[0-9].sock") ||
    fail "no chora socket: $(cat "$out/chora.log")"
export CHORA_SOCKET WAYLAND_DISPLAY=$(basename "$CHORA_SOCKET" .sock)
foot --title=gate4 sh -c "read line; printf %s \"\$line\" >$out/typed.txt" &
pids+=($!)
sleep 3
grant input.inject
printf '[{"text": {"text": "Gr\\u00fc\\u00dfe \\u00e4\\u00f6\\u00fc\\u00df\\n"}}]' | ctl inject ||
    fail "the injection"
for _ in $(seq 50); do
    test -s "$out/typed.txt" && break
    sleep 0.2
done
test "$(cat "$out/typed.txt" 2>/dev/null)" = "Grüße äöüß" ||
    fail "foot shows $(cat "$out/typed.txt" 2>/dev/null)"
ok "foot shows the injected Grüße äöüß unchanged on a US layout"
