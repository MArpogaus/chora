#!/bin/bash
# Captures of chora-mc for the pixel review: one headless session with
# chora-mc, the HUD host in the look of mc and a fake voice agent, in one
# theme at one scale, and one PNG for each state:
#
#   scene-head     the first page, at rest
#   hover          the pointer over the key row "Start a program"
#   all-keys       Super+F1, the whole key list
#   agenda         the first page with notifications, an urgent line and a
#                  helper that asks
#   terminal       Super+Return, the terminal page
#   chat-question  the chat open with a question and its card
#
# Usage: tests/mc-capture.sh THEME SCALE OUTDIR
#   THEME   light or dark
#   SCALE   1 or 2: the scale of the output, which is 1280x800 logical
#           pixels, so 2560x1600 pixels at 2
#   OUTDIR  where STATE-THEME-sSCALE.png go (created)
#   BIN     directory with the binaries (default: target/debug)
# The binaries chora, choractl, chora-mc, chora-assistant and chora-chat
# must be built. The session runs with LANG=C.UTF-8, so that foot prints no
# locale warning into the terminal state.
# shellcheck disable=SC2016 # until_ok evaluates its first argument
. "$(dirname "$0")/gate-lib.sh"

[ $# -eq 3 ] || fail "usage: tests/mc-capture.sh THEME SCALE OUTDIR"
theme=$1
scale=$2
out=$3
case $theme in light | dark) ;; *) fail "THEME is light or dark, not $theme" ;; esac
case $scale in 1 | 2) ;; *) fail "SCALE is 1 or 2, not $scale" ;; esac
mkdir -p "$out" || fail "cannot make $out"
out=$(cd "$out" && pwd)

need_bins chora choractl chora-mc chora-assistant chora-chat
need jq
need python3
need foot
export PATH=$bin:$PATH
export CHORA_BIN=$bin
export LANG=C.UTF-8 LC_ALL=C.UTF-8
unset DBUS_SESSION_BUS_ADDRESS
temp_config
work=$XDG_CONFIG_HOME
mkdir -p "$work/chora"

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject $1"; }
q() { QOUT=$(ctl query "$@") || fail "ctl query $* failed"; }
node_exists() {
    q "id:$1"
    jq -e 'length > 0' <<<"$QOUT" >/dev/null
}
text_rect() {
    ctl query "owner:mc kind:text" | jq -c --arg t "$1" \
        '[.[] | select(.visible and (.rects | length) > 0 and .text == $t)][0].rects | to_entries[0].value'
}
move_to() { echo "[{\"pointer_move\": {\"position\": [$1, $2]}}]" | ctl inject >/dev/null || fail "inject move"; }
emit() { echo "$1 $2" >&3; }
shot() {
    sleep 1.2
    ctl capture "$out/$1-$theme-s$scale.png" >/dev/null || fail "capture $1"
    ok "$1-$theme-s$scale.png"
}
text_shows() {
    ctl query "owner:mc kind:text" | jq -e --arg t "$1" '[.[] | select(.visible and (.rects | length) > 0 and .text == $t)] | length > 0' >/dev/null
}

cat >"$work/entries.json" <<'JSON'
[{"seq": 1, "role": "user", "text": "Please write notes.txt"},
 {"seq": 2, "role": "agent", "text": "I will write the file now."},
 {"seq": 3, "role": "question", "text": "Write notes.txt?"}]
JSON
printf 'theme: %s\ntest_accept_injected_keys: true\nvoice_hud: host\n' "$theme" >"$work/chora/chora-mc.yaml"
printf 'enabled: true\nposition: null\nhud: mc\n' >"$work/chora/chora-assistant.yaml"
export XDG_DATA_HOME=$work/data
mkdir -p "$XDG_DATA_HOME/chora/huds"
cp -rL "$repo/plugins/chora-mc/hud" "$XDG_DATA_HOME/chora/huds/mc"
env_file=$(mktemp /tmp/gate-env.XXXXXX)
tmp_dirs+=("$env_file")
env_file=$env_file CHORA_TEST_FLAGS=--test-injected-press-is-real session_start "$((1280 * scale))x$((800 * scale))@$scale"
grant screen.capture input.inject methods.call
mkfifo "$work/voice.in"
FAKE_VOICE_KEYS=1 FAKE_VOICE_ENTRIES=$work/entries.json python3 -I "$repo/tests/fake-voice.py" "$work/claims" \
    <"$work/voice.in" >"$work/voice.log" 2>&1 &
pids+=($!)
exec 3>"$work/voice.in"
"$bin/chora-mc" >"$work/mc.log" 2>&1 &
pids+=($!)
ctl wait owner:mc --timeout 20 >/dev/null || fail "mc does not start"
"$bin/chora-assistant" >"$work/assistant.log" 2>&1 &
pids+=($!)
ctl wait owner:assistant --timeout 20 >/dev/null || fail "the host does not start: $(tail -n 3 "$work/assistant.log")"
until_ok 'text_shows "AGENDA"' || fail "no first page"

shot scene-head

until_ok '[ -n "$(text_rect "Start a program" 2>/dev/null)" ]' 120 ||
    fail "no key row to hover: $(ctl query "owner:mc kind:text" | jq -c '[.[] | select(.visible) | .text]')"
read -r x y <<<"$(jq -r '"\(.[0] + 40) \(.[1] + .[3] / 2)"' <<<"$(text_rect "Start a program")")"
move_to "$x" "$y"
until_ok 'ctl query "owner:mc kind:rect" | jq -e "[.[] | select((.id | endswith(\"-mark\")) and .opacity == 1)] | length > 0" >/dev/null' ||
    fail "no hover mark on the key row"
shot hover
move_to 400 400

key Super+F1
until_ok 'text_shows "Show overview"' || fail "Super+F1 does not show the whole list"
shot all-keys
key Super+F1
until_ok '! text_shows "Show overview"' || fail "Super+F1 does not close the list"

emit voice.state '{"state": "listening", "task": "", "said": ""}'
emit voice.state '{"name": "Atlas", "state": "question", "task": "Write file", "seq": 5}'
ctl call mc notify.push '{"title": "Update ready"}' >/dev/null || fail "notify.push"
ctl call mc notify.push '{"title": "Backup done and a longer line that wraps"}' >/dev/null || fail "notify.push"
ctl call mc notify.push '{"title": "Disk is full", "urgency": "critical"}' >/dev/null || fail "notify.push critical"
until_ok 'text_shows "Atlas"' || fail "no row for the helper"
shot agenda

key Super+Return
sleep 3
shot terminal
key Super+Home
until_ok 'text_shows "AGENDA"' || fail "Super+Home does not show the first page"

key Super+a
until_ok 'node_exists hud-chat' 40 || fail "Super+a does not show the chat"
if node_exists hud-chat-pick-1; then
    key 1
    key Return
    until_ok 'node_exists hud-chat-stop' 40 || fail "the chat of the helper does not open"
fi
emit voice.confirm '{"id": "71", "prompt": "Atlas wants to write the file notes.txt.", "detail": "~/projects/notes/notes.txt (412 bytes)", "physical": true, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": true}'
until_ok 'node_exists hud-card' 40 || fail "no card for the question"
shot chat-question
