#!/bin/bash
# Gate of the main paths of chora-mc for a person with arrows, Escape and a
# mouse: chora-mc with `--manage-windows` (it holds the claim of its output,
# as in a session of the mc look, so the core reserves new windows for mc)
# and `voice_hud: host`, the mc folder of the HUD host and
# a fake voice agent on a headless output of 1280x800, where an injected
# press is an act of the person (--test-injected-press-is-real). One cell
# for each path; a cell that fails prints `FAIL:` and the next one runs.
#
#   first-page     the first page says how to start and no text is under
#                  14 px
#   first-turn     in a fresh session (no terminal yet) the first plain
#                  Right from page 4 shows page 5, the terminal page that
#                  the footer names, and its window lands in that page,
#                  not in a page of its own; Right, Left and Escape then
#                  act on the page (mc-42, persona round 4: mc claims its
#                  output, so the core reserves the window for mc)
#   arrows         plain Right and Left turn the page, Escape goes back one
#                  step, and the key list names the same keys as the footer
#   terminal-keys  on the terminal page the arrows belong to the shell, the
#                  key list names Super with the arrows, and Super+Left
#                  turns the page; Ctrl+g after the plain Escape ends the
#                  Meta prefix of the shell, which takes the next
#                  character otherwise
#   mouse-back     a click on the footer of page 2 turns back
#   mute           the state line has the word Mute; a press on it mutes
#                  the microphone, the badge of the host then turns it on,
#                  the word comes back, and the arrows still turn the page
#   terminal-chat  the terminal page keeps the column; the chat opens in it
#                  and a page turn with the chat open leaves no part of it
#                  outside the column; after Super+a and Super+Right the
#                  typed text still reaches the chat field, after Escape it reaches
#                  the terminal, and not the hidden terminal after a page turn
#   chat-click-turn   with the chat open a click on a window of mc, a page
#                  turn, and the chat closes: the hidden window gets no
#                  key, the page gets the keyboard
#   chat-click-close  (a second output with chora-tiling) with the chat
#                  open a click on a window of tiling, that window closes,
#                  the chat closes: the page of mc gets the keyboard
#   terminal-back  a program that mc adopted has a page with the column
#                  beside it; the page is reached by Right, then Left and
#                  Escape of the remote go back to the page before and
#                  the doors of the column are on the screen; Return
#                  gives the keyboard to the window, and the key list
#                  shows on the page of a window (mc-42, mc-43)
#   terminal-turn  the page of the terminal that a plain Right starts keeps
#                  the keyboard as a page reached by a plain key does:
#                  Left, Right, Escape and XF86Back of a remote act on the
#                  page, XF86Back one step back as Escape (mc-42, mc-46);
#                  also Right, Left, Right before the window maps (rv-1)
#   window-chat    the chat opens over a window that holds the keyboard:
#                  state.get of mc names the surface that the core still
#                  focuses under the field (I3); after Escape the window
#                  holds it again
#   chat-focus     after a request to the helper and Escape in the chat
#                  the page holds the keyboard again, state.get says mc
#                  (mc-45)
#   stray-super    Super+x, Super+k, Super+m and Super+q, which mc binds
#                  to nothing, type nothing into the window of a page
#                  that holds the keyboard (mc-60); Super+F1 still acts
#   all-keys       the full key list stays in the column and on the
#                  screen, and its rule crosses no text
#
# The gate ends with the window audit: the terminal of the terminal page
# is still there.
#
# Usage: tests/gate-mc-paths.sh   BIN: directory with the binaries
#   CELLS  a regular expression of the cells that run (default all)
#   SHOTS  a directory that keeps one capture of each cell as
#          mc-paths-CELL.png
# shellcheck disable=SC2016 # until_ok evaluates its first argument
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-mc chora-assistant chora-chat chora-tiling
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
emit() { echo "$1 $2" >&3; }
shown='select(.visible and (.rects | length) > 0)'
texts() { ctl query "owner:mc kind:text" | jq -r ".[] | $shown | .text"; }
has_text() { texts | grep -qxF "$1"; }
says() { texts | grep -qF "$1"; }
text_rect() {
    ctl query "owner:mc kind:text" | jq -c --arg t "$1" \
        '[.[] | select(.visible and (.rects | length) > 0 and .text == $t)][0].rects | to_entries[0].value'
}
text_size() {
    ctl query "owner:mc kind:text" | jq -r --arg t "$1" \
        '[.[] | select(.visible and (.rects | length) > 0 and .text == $t)][0].font_size'
}
hit_rect() {
    ctl query "owner:mc kind:rect" | jq -c --arg s "$1-hit" \
        '[.[] | select((.id | endswith($s)) and (.rects | length) > 0)][0].rects | to_entries[0].value'
}
rect_of() {
    ctl query "owner:mc kind:rect" | jq -c --arg s "$1" \
        '[.[] | select((.id | endswith($s)) and (.rects | length) > 0)][0].rects | to_entries[0].value'
}
node_exists() { ctl query "id:$1" | jq -e 'length > 0' >/dev/null; }
node_rect() { ctl query "id:$1" | jq -c '.[0].rects | to_entries[0].value'; }
move_to() { echo "[{\"pointer_move\": {\"position\": [$1, $2]}}]" | ctl inject >/dev/null || fail "inject move"; }
click_rect() {
    local x y
    x=$(jq -n --argjson r "$1" '$r[0] + ([$r[2] / 2, 40] | min) | floor')
    y=$(jq -n --argjson r "$1" '$r[1] + $r[3] / 2 | floor')
    move_to "$x" "$y"
    sleep 0.3
    echo '[{"button": {"state": "pressed"}}, {"button": {"state": "released"}}]' |
        ctl inject >/dev/null || fail "inject click"
}
bars() { ctl query "owner:mc kind:rect" | jq -r '[.[] | select((.id | endswith("-mark")) and .opacity == 1 and (.rects | length) > 0)] | length'; }
current() { ctl call mc scene.list '{}' | jq -r '.current'; }
shot() {
    [ -n "${SHOTS:-}" ] || return 0
    mkdir -p "$SHOTS"
    sleep 1.2
    ctl capture "$SHOTS/mc-paths-$1.png" >/dev/null || fail "capture $1"
}
home() {
    ctl call assistant chat.hide '{}' >/dev/null 2>&1
    has_text "Show overview" && key Super+F1
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    move_to 400 400
    sleep 0.5
}
small_texts() {
    ctl query "owner:mc kind:text" | jq -r '[.[] | select(.visible and (.rects | length) > 0 and .font_size < 14) | "\(.text):\(.font_size)"] | join(" ")'
}

printf 'theme: dark\ntest_accept_injected_keys: true\nvoice_hud: host\n' >"$work/chora/chora-mc.yaml"
printf 'enabled: true\nposition: null\nhud: mc\n' >"$work/chora/chora-assistant.yaml"
cat >"$work/entries.json" <<'JSON'
[{"seq": 1, "role": "user", "text": "Tidy my notes."},
 {"seq": 2, "role": "agent", "text": "I read 14 notes and made 5 days."}]
JSON
export XDG_DATA_HOME=$work/data
mkdir -p "$XDG_DATA_HOME/chora/huds"
cp -rL "$repo/plugins/chora-mc/hud" "$XDG_DATA_HOME/chora/huds/mc"
CHORA_TEST_FLAGS=--test-injected-press-is-real session_start 1280x800
grant screen.capture input.inject methods.call
mkfifo "$work/voice.in"
FAKE_VOICE_ENTRIES=$work/entries.json python3 -I "$repo/tests/fake-voice.py" "$work/claims" \
    <"$work/voice.in" >"$work/voice.log" 2>&1 &
pids+=($!)
exec 3>"$work/voice.in"
"$bin/chora-mc" --manage-windows >"$work/mc.log" 2>&1 &
pids+=($!)
ctl wait owner:mc --timeout 20 >/dev/null || fail "mc does not start"
"$bin/chora-assistant" >"$work/assistant.log" 2>&1 &
pids+=($!)
ctl wait owner:assistant --timeout 20 >/dev/null || fail "the host does not start: $(tail -n 3 "$work/assistant.log")"
until_ok 'has_text "AGENDA"' || fail "no first page"
sleep 2

cell_first_page() {
    local small
    says 'Say “Atlas” or press Super+a.' || fail "the first page does not say how to start: $(texts | tr '\n' '|')"
    has_text "Chat with a helper ..." || fail "the first page has no door to a helper"
    small=$(small_texts)
    [ -z "$small" ] || fail "text under 14 px on the first page: $small"
    shot first-page
}

cell_arrows() {
    has_text "← →" || fail "the key list does not name the plain arrows: $(texts | grep -F '←' | tr '\n' '|')"
    has_text "→ 2  The Morning" || fail "the footer does not point to page 2"
    key Right
    until_ok '[ "$(current)" = morning ]' 20 || fail "Right does not turn to page 2: $(current)"
    key Right
    until_ok '[ "$(current)" = agents ]' 20 || fail "a second Right does not turn to page 3: $(current)"
    key Left
    until_ok '[ "$(current)" = morning ]' 20 || fail "Left does not turn back to page 2: $(current)"
    key Escape
    until_ok '[ "$(current)" = agents ]' 20 || fail "Escape does not go back one step, to page 3: $(current)"
    key Escape
    until_ok '[ "$(current)" = morning ]' 20 || fail "a second Escape does not go back to page 2: $(current)"
    key Super+F1
    until_ok 'has_text "Show overview"' 20 || fail "Super+F1 does not show all keys"
    key Escape
    until_ok '! has_text "Show overview"' 20 || fail "Escape does not close the list of all keys"
    [ "$(current)" = morning ] || fail "Escape left the page with the list: $(current)"
    shot arrows
}

cell_terminal_keys() {
    key Super+Return
    until_ok '[ "$(current)" = terminal ]' 20 || fail "Super+Return does not show the terminal: $(current)"
    until_ok '[ "$(ctl query kind:surface | jq length)" -ge 1 ]' 40 || fail "no terminal window"
    sleep 2
    until_ok 'has_text "Super+← →"' 20 || fail "the key list of the terminal page does not name Super with the arrows: $(texts | grep -F '←' | tr '\n' '|')"
    key Left
    key Escape
    key Ctrl+g
    sleep 1
    [ "$(current)" = terminal ] || fail "a plain key of the shell turned the page: $(current)"
    key Super+Left
    until_ok '[ "$(current)" = mission ]' 20 || fail "Super+Left does not turn the page from the terminal: $(current)"
    key Left
    until_ok '[ "$(current)" = agents ]' 20 || fail "Left does not turn the page after the terminal: $(current)"
}

focus_holder() { ctl call mc state.get '{}' | jq -r '.focus.holder'; }
cell_chat_focus() {
    key Super+a
    until_ok 'node_exists hud-chat' 40 || fail "Super+a does not open the chat"
    sleep 1
    type_line "tidy"
    until_ok 'chat_has_sent tidy' 20 || fail "the request does not reach the chat"
    key Escape
    until_ok '! node_exists hud-chat' 40 || fail "Escape does not close the chat"
    sleep 1
    until_ok '[ "$(focus_holder)" = mc ]' 20 || fail "after the chat closed the focus is with $(focus_holder), not mc: $(ctl call mc state.get '{}' | jq -c .focus)"
    key Right
    until_ok '[ "$(current)" = morning ]' 20 || fail "after the chat closed Right does not turn the page: $(current)"
}

core_window() { ctl query kind:surface | jq -r '[.[] | select(.focused) | .id][0] // "none"'; }
mc_focus() { ctl call mc state.get '{}' | jq -c .focus; }
cell_window_chat() {
    local core focus
    key Super+Return
    until_ok '[ "$(current)" = terminal ] && [ "$(core_window)" != none ]' 40 || fail "Super+Return gives no terminal the keyboard: $(current)"
    sleep 1
    core=$(core_window)
    [ "$(mc_focus)" = "{\"holder\":\"mc\",\"surface\":\"$core\"}" ] || fail "the terminal of Super+Return holds no keyboard: $(mc_focus)"
    key Super+a
    until_ok 'node_exists hud-chat' 40 || fail "Super+a does not open the chat over the terminal"
    sleep 1
    core=$(core_window)
    focus=$(mc_focus)
    if [ "$core" = none ]; then
        [ "$(jq -r .holder <<<"$focus")" = null ] || fail "with the chat open the core focuses nothing, mc says $focus"
    else
        [ "$focus" = "{\"holder\":\"mc\",\"surface\":\"$core\"}" ] || fail "with the chat open over the terminal the core focuses $core, mc says $focus (I3)"
    fi
    key Escape
    until_ok '! node_exists hud-chat' 40 || fail "Escape does not close the chat over the terminal"
    until_ok '[ "$(mc_focus)" = "{\"holder\":\"mc\",\"surface\":\"$(core_window)\"}" ]' 20 || fail "after the chat the terminal does not hold the keyboard: $(mc_focus), core $(core_window)"
}

back_alive() { session_pids -f -- "--app-id=mcback" >/dev/null; }
back_end() {
    local pid
    for pid in $(session_pids -f -- "--app-id=mcback"); do kill "$pid"; done
    until_ok '! back_alive' 20
}
cell_terminal_back() {
    local mark=mcbackkey i
    rmdir $mark 2>/dev/null
    foot --app-id=mcback &
    until_ok 'back_alive && [ "$(current)" = mcback ]' 40 || { back_end; fail "the window of a program has no page: $(current)"; }
    sleep 2
    has_text "Chat with a helper ..." || { back_end; fail "the page of the window has no column: $(texts | tr '\n' '|')"; }
    has_text "Programs" || { back_end; fail "the page of the window has no door Programs"; }
    shot terminal-back
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || { back_end; fail "no agenda: $(current)"; }
    for i in 1 2 3 4 5 6; do key Right; sleep 0.4; done
    until_ok '[ "$(current)" = mcback ]' 20 || { back_end; fail "Right does not reach the page of the window: $(current)"; }
    key Left
    until_ok '[ "$(current)" = overview ]' 20 || { back_end; fail "Left on the page of a window does not go to the page before: $(current)"; }
    key Right
    until_ok '[ "$(current)" = mcback ]' 20 || { back_end; fail "Right does not come back to the page of the window: $(current)"; }
    key Escape
    until_ok '[ "$(current)" = overview ]' 20 || { back_end; fail "Escape on the page of a window does not go back: $(current)"; }
    key Right
    until_ok '[ "$(current)" = mcback ]' 20 || { back_end; fail "no page of the window again: $(current)"; }
    sleep 1
    has_text "Chat with a helper ..." || { back_end; fail "after the turns the page of the window has no column"; }
    key Super+F1
    until_ok 'has_text "Show overview"' 20 || { back_end; fail "Super+F1 on the page of a window draws no key list: $(texts | tr '\n' '|')"; }
    shot terminal-back-keys
    key Super+F1
    until_ok '! has_text "Show overview"' 20 || { back_end; fail "Super+F1 does not close the key list on the page of a window"; }
    key Return
    sleep 1
    type_line "mkdir $mark"
    until_ok "[ -e $mark ]" 20 || { back_end; fail "after Return typed text does not reach the window"; }
    rmdir $mark 2>/dev/null
    back_end
    return 0
}

foot_alive() { session_pids -x foot >/dev/null; }
foot_end() {
    local pid
    for pid in $(session_pids -x foot); do kill "$pid"; done
    until_ok '! foot_alive' 20
}
turn_arrive() {
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || return 1
    for _ in 1 2 3 4; do key Right; sleep 0.4; done
    until_ok '[ "$(current)" = terminal ] && foot_alive' 40 || return 1
    sleep 2
}
no_page_of_its_own() {
    local scenes
    scenes=$(ctl call mc scene.list '{}' | jq -r '[.scenes[] | .name? // .] | join(" ")')
    [[ " $scenes " != *" foot "* ]] || { echo "the window has a page of its own: $scenes"; return 1; }
}
cell_first_turn() {
    local k page why
    foot_alive && fail "a terminal runs before the first turn"
    for _ in 1 2 3; do key Right; sleep 0.4; done
    until_ok '[ "$(current)" = mission ]' 20 || fail "no page 4 by Right x3: $(current)"
    until_ok 'has_text "→ 5  Terminal"' 20 || fail "the footer of page 4 does not name page 5 Terminal: $(texts | grep -F '→' | tr '\n' '|')"
    key Right
    until_ok '[ "$(ctl query kind:surface | jq length)" -ge 1 ]' 40 || { foot_end; fail "the first Right from page 4 starts no terminal"; }
    sleep 2
    page=$(current)
    [ "$page" = terminal ] || { foot_end; fail "the first Right from page 4 shows $page, not the terminal page 5"; }
    why=$(no_page_of_its_own) || { foot_end; fail "the first Right from page 4: $why"; }
    texts | grep -qE '^←.*→$' || { foot_end; fail "the page of the first start names no plain arrows: $(texts | grep -F '←' | tr '\n' '|')"; }
    key Right
    until_ok '[ "$(current)" = overview ]' 20 || { foot_end; fail "Right on the page of the first start does not go on: $(current)"; }
    foot_end
    for k in Left:mission Escape:mission; do
        key Super+Home
        until_ok '[ "$(current)" = agenda ]' 20 || fail "no first page before $k: $(current)"
        for _ in 1 2 3 4; do key Right; sleep 0.4; done
        until_ok '[ "$(ctl query kind:surface | jq length)" -ge 1 ]' 40 || { foot_end; fail "Right x4 starts no terminal before $k"; }
        sleep 2
        page=$(current)
        why=$(no_page_of_its_own) && [ "$page" = terminal ] || { foot_end; fail "Right x4 before $k shows $page: $why"; }
        key "${k%%:*}"
        until_ok "[ \"\$(current)\" = ${k##*:} ]" 20 || { foot_end; fail "${k%%:*} on the page that Right started does not go to ${k##*:}: $(current)"; }
        foot_end
    done
}
cell_terminal_turn() {
    local k want
    for k in Left:mission Right:overview Escape:mission XF86Back:mission; do
        turn_arrive || { foot_end; fail "no terminal page by Right x4 before $k: $(current)"; }
        texts | grep -qE '^←.*→$' || { foot_end; fail "the page that Right started names no plain arrows: $(texts | grep -F '←' | tr '\n' '|')"; }
        key "${k%%:*}"
        want=${k##*:}
        until_ok "[ \"\$(current)\" = $want ]" 20 || { foot_end; fail "${k%%:*} on the terminal page reached by Right does not go to $want: $(current)"; }
        foot_end
    done
    turn_arrive || fail "no terminal page by Right x4 at the end: $(current)"
    foot_end
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no first page before the quick turn: $(current)"
    for _ in 1 2 3; do key Right; sleep 0.4; done
    echo '[{"shortcut": {"keys": "Right"}}, {"shortcut": {"keys": "Left"}}, {"shortcut": {"keys": "Right"}}]' |
        ctl inject >/dev/null || fail "inject Right, Left, Right"
    until_ok '[ "$(current)" = terminal ] && foot_alive' 40 || { foot_end; fail "no terminal page after Right, Left, Right: $(current)"; }
    sleep 2
    for k in Left:mission Right:terminal Escape:mission; do
        key "${k%%:*}"
        until_ok "[ \"\$(current)\" = ${k##*:} ]" 20 || { foot_end; fail "${k%%:*} after Right, Left, Right before the window mapped does not go to ${k##*:}: $(current)"; }
    done
    foot_end
}

cell_mouse_back() {
    key Super+Right
    until_ok '[ "$(current)" = morning ]' 20 || fail "no page 2"
    until_ok 'has_text "← 1  Agenda"' 20 || fail "the footer of page 2 has no line back"
    has_text "→ 3  Helpers" || fail "the footer of page 2 has no line on"
    shot mouse-back
    sleep 1
    click_rect "$(text_rect "← 1  Agenda")"
    until_ok '[ "$(current)" = agenda ]' 20 || fail "a click on the line back does not turn back: $(current)"
    click_rect "$(text_rect "→ 2  The Morning")"
    until_ok '[ "$(current)" = morning ]' 20 || fail "a click on the line on does not turn the page: $(current)"
    key Super+Left
    until_ok '[ "$(current)" = agenda ]' 20 || fail "Super+Left does not turn back after the clicks: $(current)"
}

cell_mute() {
    local rect
    emit voice.state '{"state": "listening", "task": "", "said": "", "muted": {"mic": false, "speaker": false}}'
    until_ok 'has_text "Mute"' 20 || fail "the state line has no word Mute: $(texts | tr '\n' '|')"
    rect=$(text_rect "Mute")
    jq -ne --argjson r "$rect" '$r[0] >= 888 and $r[0] + $r[2] <= 1224 and $r[1] < 84' >/dev/null ||
        fail "Mute is not on the first line of the column: $rect"
    [ "$(jq -n --argjson s "$(text_size Mute)" '$s >= 14')" = true ] || fail "Mute is under 14 px"
    move_to "$(jq -n --argjson r "$rect" '$r[0] + 10')" "$(jq -n --argjson r "$rect" '$r[1] + $r[3] / 2 | floor')"
    until_ok '[ "$(bars)" = 1 ]' 8 || fail "no focus bar over Mute"
    shot mute
    click_rect "$rect"
    until_ok 'grep -q "\"mic\": true" "$work/claims.mute" 2>/dev/null' 20 || fail "a press on Mute does not mute the microphone"
    until_ok '! has_text "Mute"' 20 || fail "Mute stays beside a muted microphone"
    until_ok 'node_exists hud-badge' 20 || fail "the host shows no badge of the muted microphone"
    shot muted
    key Right
    until_ok '[ "$(current)" = morning ]' 20 || fail "Right does not turn the page after the press on Mute: $(current)"
    key Left
    until_ok '[ "$(current)" = agenda ]' 20 || fail "Left does not turn back"
    click_rect "$(node_rect hud-badge)"
    until_ok 'grep -q "\"mic\": false" "$work/claims.mute"' 20 || fail "a press on the badge does not turn the microphone on"
    until_ok 'has_text "Mute"' 20 || fail "Mute does not come back"
}

chat_has_sent() { ctl query "id:hud-chat" | jq -e --arg t "$1" '.[0].meta.text | endswith("You\n" + $t)' >/dev/null; }
type_line() {
    local i
    for ((i = 0; i < ${#1}; i++)); do
        case ${1:i:1} in
        " ") key space ;;
        *) key "${1:i:1}" ;;
        esac
    done
    key Return
}
terminal_got() { [ -e "$PWD/$1" ]; }
in_column() { jq -ne --argjson c "$1" '$c[0] >= 888 and $c[0] + $c[2] <= 1224 and $c[1] > 56 and $c[1] + $c[3] <= 786' >/dev/null; }
cell_terminal_chat() {
    local chat mark_fwd=mckeyfwd mark_rev=mckeyrev
    rmdir mckeyfwd mckeyrev 2>/dev/null
    key Super+Return
    until_ok '[ "$(current)" = terminal ]' 20 || fail "no terminal page: $(current)"
    sleep 2
    has_text "Chat with a helper ..." || fail "the terminal page has no door of the chat"
    has_text "Back one step" || fail "the terminal page has no key list"
    [ "$(rect_of right-edge)" = "[860.0,37.0,1.0,738.0]" ] || fail "the terminal page has no hairline of the column: $(rect_of right-edge)"
    ctl call assistant chat.show '{"name": "atlas"}' >/dev/null || fail "chat.show"
    until_ok 'node_exists hud-chat' 40 || fail "the host draws no chat on the terminal page"
    sleep 1
    chat=$(node_rect hud-chat)
    in_column "$chat" || fail "the chat of the terminal page is not in the column: $chat"
    has_text "Esc  Close" || fail "the open chat has no head with Esc  Close"
    shot terminal-chat
    key Super+Left
    until_ok '[ "$(current)" = mission ]' 20 || fail "Super+Left does not turn the page with the chat open: $(current)"
    sleep 2
    if node_exists hud-chat; then
        chat=$(node_rect hud-chat)
        in_column "$chat" || fail "after the page turn the chat is not in the column: $chat"
        has_text "Esc  Close" || fail "after the page turn the chat has no head"
    else
        has_text "Back one step" || fail "the chat closed with the page turn, but the key list is not back"
    fi
    [ "$(rect_of right-edge)" = "[860.0,37.0,1.0,738.0]" ] || fail "after the page turn the hairline of the column is gone"
    ctl call assistant chat.hide '{}' >/dev/null || fail "chat.hide"
    until_ok '! node_exists hud-chat' 40 || fail "the chat does not close"
    key Super+a
    until_ok 'node_exists hud-chat' 40 || fail "Super+a does not open the chat"
    sleep 1
    key Super+Right
    until_ok '[ "$(current)" = terminal ]' 20 || fail "Super+Right does not turn the page with the chat open: $(current)"
    sleep 1
    key h
    key i
    key Return
    sleep 2
    until_ok 'chat_has_sent hi' 20 || fail "after the page turn typed text does not reach the chat field"
    key Escape
    until_ok '! node_exists hud-chat' 40 || fail "Escape does not close the chat"
    sleep 3
    type_line "mkdir $mark_fwd"
    until_ok 'terminal_got $mark_fwd' 20 || fail "after the chat closed typed text does not reach the terminal"
    key Super+a
    until_ok 'node_exists hud-chat' 40 || fail "Super+a does not open the chat on the terminal page"
    sleep 1
    key Super+Left
    until_ok '[ "$(current)" = mission ]' 20 || fail "Super+Left does not turn the page: $(current)"
    key Escape
    until_ok '! node_exists hud-chat' 40 || fail "Escape does not close the chat on the page turned to"
    sleep 3
    type_line "mkdir $mark_rev"
    sleep 2
    terminal_got $mark_rev && fail "after the chat closed on another page typed text reached the hidden terminal"
    rmdir mckeyfwd mckeyrev 2>/dev/null
    return 0
}

side_window() {
    ctl call mc app.open "{\"command\": \"foot --app-id=mcside\"}" >/dev/null || fail "app.open"
    until_ok 'side_alive' 40 || fail "no window of app.open"
    sleep 2
    move_to 400 400
    sleep 0.3
    echo '[{"button": {"state": "pressed"}}, {"button": {"state": "released"}}]' |
        ctl inject >/dev/null || fail "inject click"
    sleep 1
}
side_alive() { session_pids -f -- "--app-id=mcside" >/dev/null; }
side_end() {
    local pid
    for pid in $(session_pids -f -- "--app-id=mcside"); do kill "$pid"; done
    until_ok '! side_alive' 20
}
cell_chat_click_turn() {
    local mark=mcclickhidden
    rmdir $mark 2>/dev/null
    key Super+a
    until_ok 'node_exists hud-chat' 40 || fail "Super+a does not open the chat"
    sleep 1
    side_window
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || { side_end; fail "Super+Home does not turn the page with the chat open: $(current)"; }
    ctl call assistant chat.hide '{}' >/dev/null || { side_end; fail "chat.hide"; }
    until_ok '! node_exists hud-chat' 40 || { side_end; fail "the chat does not close"; }
    sleep 2
    type_line "mkdir $mark"
    sleep 2
    side_end
    terminal_got $mark && { rmdir $mark; fail "after the chat closed on another page typed text reached the hidden window"; }
    key Right
    until_ok '[ "$(current)" = morning ]' 20 || fail "after the chat closed the page gets no keys: $(current)"
}

chat_output() { ctl query "id:hud-chat" | jq -r '.[0].rects | keys | join(",")'; }
tiling_alive() { session_pids -f -- "--app-id=mctiled" >/dev/null; }
tiling_window() {
    echo '[{"pointer_move": {"position": [600, 400], "output": "HEADLESS-2"}},
        {"button": {"state": "pressed"}}, {"button": {"state": "released"}}]' |
        ctl inject >/dev/null || fail "inject click on the second output"
    foot --app-id=mctiled &
    until_ok '[ "$(ctl query "kind:surface owner:tiling" | jq length)" -ge 1 ]' 40 ||
        fail "no window of tiling: $(ctl query kind:surface | jq -c '[.[] | [.id, .owner]]')"
    sleep 2
}
tiling_end() {
    local pid
    for pid in $(session_pids -f -- "--app-id=mctiled"); do kill "$pid"; done
    until_ok '! tiling_alive' 20
}
cell_chat_click_close() {
    key Super+a
    until_ok 'node_exists hud-chat' 40 || fail "Super+a does not open the chat"
    sleep 1
    [ "$(chat_output)" = HEADLESS-1 ] || fail "the chat does not open on the output of mc: $(chat_output)"
    tiling_window
    type_line "exit"
    until_ok '! tiling_alive' 20 || { tiling_end; fail "the clicked window of tiling does not get the keys"; }
    [ "$(chat_output)" = HEADLESS-1 ] || fail "the chat left the output of mc: $(chat_output)"
    move_to 400 400
    ctl call assistant chat.hide '{}' >/dev/null || fail "chat.hide"
    until_ok '! node_exists hud-chat' 40 || fail "the chat does not close"
    sleep 2
    key Right
    until_ok '[ "$(current)" = morning ]' 20 || fail "after the window of tiling and the chat closed the page gets no keys: $(current)"
}

cell_all_keys() {
    local rule over crossed
    key Super+F1
    until_ok 'has_text "Show overview"' 20 || fail "Super+F1 does not show all keys"
    sleep 1
    shot all-keys
    has_text "Back one step" || fail "the last row of the list does not show"
    has_text "Chat with a helper ..." && fail "the door of the chat stays under the full list"
    over=$(ctl query "owner:mc kind:text" | jq -r '[.[] | select(.visible and (.rects | length) > 0) | (.rects | to_entries[0].value) as $r | select($r[0] >= 860 and ($r[1] + $r[3] > 786 or $r[0] + $r[2] > 1226)) | .text] | join("|")')
    [ -z "$over" ] || fail "rows of the full list leave the column: $over"
    rule=$(rect_of right-keys-title-rule)
    crossed=$(ctl query "owner:mc kind:text" | jq -r --argjson l "$rule" '[.[] | select(.visible and (.rects | length) > 0) | (.rects | to_entries[0].value) as $r | select($r[0] >= 860 and $r[1] + 4 < $l[1] and $r[1] + $r[3] - 4 > $l[1]) | .text] | join("|")')
    [ -z "$crossed" ] || fail "the rule of the full list crosses: $crossed"
    key Super+F1
    until_ok '! has_text "Show overview"' 20 || fail "Super+F1 does not close the list"
}

stray_alive() { session_pids -f -- "--app-id=mcstray" >/dev/null; }
stray_end() {
    local pid
    for pid in $(session_pids -f -- "--app-id=mcstray"); do kill "$pid"; done
    until_ok '! stray_alive' 20
}
cell_stray_super() {
    local log=$work/mcstray typed chord
    rm -f "$log"
    foot --app-id=mcstray sh -c "stty raw -echo; cat >$log" &
    until_ok 'stray_alive && [ "$(current)" = mcstray ]' 40 || { stray_end; fail "the window of a program has no page: $(current)"; }
    sleep 2
    key Return
    sleep 1
    echo '[{"text": {"text": "start:"}}]' | ctl inject >/dev/null || { stray_end; fail "inject text"; }
    for chord in Super+x Super+k Super+m Super+q; do
        key "$chord"
    done
    echo '[{"text": {"text": ":end"}}]' | ctl inject >/dev/null || { stray_end; fail "inject text"; }
    sleep 1
    typed=$(tr -d '\r' <"$log" 2>/dev/null)
    stray_end
    [ "$typed" = "start::end" ] || fail "mc-60: the window page typed '$typed' and not start::end"
    key Super+F1
    until_ok 'has_text "Show overview"' 20 || fail "Super+F1 no longer shows the key list"
    key Super+F1
    until_ok '! has_text "Show overview"' 20 || fail "Super+F1 does not close the key list"
}

failed=()
run_cell() {
    local name=$1
    [[ $name =~ ${CELLS:-.} ]] || return 0
    home
    if ("cell_${name//-/_}"); then
        ok "$name"
    else
        failed+=("$name")
    fi
}
for name in first-page first-turn arrows terminal-keys terminal-back terminal-turn mouse-back mute terminal-chat window-chat chat-click-turn chat-focus all-keys stray-super; do
    run_cell "$name"
done
if [[ chat-click-close =~ ${CELLS:-.} ]]; then
    ctl add-output 1280x800+1280+0 >/dev/null || fail "add-output"
    "$bin/chora-tiling" --output HEADLESS-2 >"$work/tiling.log" 2>&1 &
    pids+=($!)
    until_ok '[ "$(ctl query kind:group | jq -r "[.[] | select(.claim) | .owner] | index(\"tiling\")")" != null ]' 80 ||
        fail "tiling holds no claim on the second output"
    run_cell chat-click-close
fi

if [[ terminal-keys =~ ${CELLS:-.} ]]; then
    [ "$(ctl query kind:surface | jq length)" -ge 1 ] || fail "window audit: the terminal of the terminal page is gone"
    ok "window audit: the terminal of the terminal page is still there"
fi
[ ${#failed[@]} = 0 ] || fail "cells failed: ${failed[*]}"
ok "every path of the mc gate"
