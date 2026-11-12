#!/bin/bash
# Gate of the mc v2 look (design/looks/mc-v2.md, the build list): the
# margin column of chora-mc on a headless output of 1280x800. Each step of
# the list adds its checks here; a check reads the texts and the rects of
# the nodes of mc with `choractl query`.
#
# The mission scene holds data in every panel and scrolls with the wheel
# (rule 13 of docs/mc-style.md), so its panels are not all in view.
#
# Usage: tests/gate-mc-column.sh   BIN: directory with the binaries
# shellcheck disable=SC2016 # until_ok evaluates its first argument
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-mc
need jq
export PATH=$bin:$PATH
export CHORA_BIN=$bin
unset DBUS_SESSION_BUS_ADDRESS
temp_config
mkdir -p "$XDG_CONFIG_HOME/chora"
printf 'theme: light\ntest_accept_injected_keys: true\n' >"$XDG_CONFIG_HOME/chora/chora-mc.yaml"
session_start 1280x800
grant screen.capture input.inject
"$bin/chora-mc" >"$XDG_CONFIG_HOME/mc.log" 2>&1 &
pids+=($!)
ctl wait owner:mc --timeout 20 >/dev/null || fail "mc does not start"

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject $1"; }
shown='select(.visible and (.rects | length) > 0)'
texts() { ctl query "owner:mc kind:text" | jq -r ".[] | $shown | .text"; }
has_text() { texts | grep -qxF "$1"; }
text_rect() {
    ctl query "owner:mc kind:text" | jq -c --arg t "$1" \
        '[.[] | select(.visible and (.rects | length) > 0 and .text == $t)][0].rects | to_entries[0].value'
}
shot() { ctl capture "${SHOTS:-/tmp}/mc-column-$1.png" >/dev/null || fail "capture $1"; }

until_ok 'has_text "AGENDA"' || fail "no first screen: $(texts | tr '\n' '|')"

for t in "Super+F1" "All keys" "Super+ ← →" "Turn the page" "Super+Space" "Start a program" \
    "Super+n" "Hide this column" "Esc" "Back one step"; do
    has_text "$t" || fail "the key list lacks \"$t\" on the first screen"
done
rect=$(text_rect "Start a program")
[ "$(jq -n --argjson r "$rect" '$r[0] >= 700')" = true ] || fail "the key list is not in the right margin: $rect"
ok "the first screen shows the six rows of the key list in the right margin"

key Super+F1
until_ok 'has_text "Show overview"' || fail "Super+F1 does not open the full list"
ok "Super+F1 opens the full list of keys"
key Super+F1
until_ok '! has_text "Show overview"' || fail "Super+F1 does not close the full list"
has_text "Start a program" || fail "the six rows do not come back"
ok "Super+F1 closes the full list"

move_to() { echo "[{\"pointer_move\": {\"position\": [$1, $2]}}]" | ctl inject >/dev/null || fail "inject move"; }
click_at() {
    move_to "$1" "$2"
    echo '[{"button": {"state": "pressed"}}, {"button": {"state": "released"}}]' |
        ctl inject >/dev/null || fail "inject click"
}
center_of() {
    jq -r 'select(. != null) | "\(.[0] + 40) \(.[1] + .[3] / 2)"' <<<"$(text_rect "$1")"
}
hit_center() {
    ctl query "owner:mc kind:rect" | jq -r --arg s "$1-hit" \
        '[.[] | select((.id | endswith($s)) and (.rects | length) > 0)][0].rects | to_entries[0].value | "\(.[0] + 160) \(.[1] + .[3] / 2)"' 2>/dev/null
}
bars() { ctl query "owner:mc kind:rect" | jq -r '[.[] | select((.id | endswith("-mark")) and .opacity == 1 and (.rects | length) > 0)] | length'; }
current() { ctl call mc scene.list '{}' | jq -r '.current'; }

[ "$(bars)" = 0 ] || fail "a focus bar shows without the pointer"
read -r x y <<<"$(center_of "Start a program")"
move_to "$x" "$y"
until_ok '[ "$(bars)" = 1 ]' 8 || fail "no focus bar over the row Start a program"
ok "the pointer over a key row shows the focus bar"
move_to 400 400
until_ok '[ "$(bars)" = 0 ]' 8 || fail "the focus bar stays after the pointer left"
ok "the focus bar goes when the pointer leaves"

read -r x y <<<"$(center_of "All keys")"
click_at "$x" "$y"
until_ok 'has_text "Show overview"' || fail "a click on the row All keys does not show all keys"
sleep 1
read -r x y <<<"$(center_of "Show overview")"
click_at "$x" "$y"
until_ok '[ "$(current)" = overview ]' 20 || fail "a click on Show overview does not show the overview"
until_ok 'has_text "Overview" || [ -n "$(hit_center scenes-row-4)" ]' 20
sleep 1
read -r x y <<<"$(hit_center scenes-row-4)"
move_to "$x" "$y"
until_ok '[ "$(bars)" = 1 ]' 8 || fail "no focus bar over a line of the overview"
click_at "$x" "$y"
until_ok '[ "$(current)" = terminal ]' 20 || fail "a click on a line of the overview does not show that scene: $(current)"
ok "a mouse alone reaches All keys, the overview and the terminal scene"

sleep 3
key Super+Home
until_ok '[ "$(current)" = agenda ]' 40 || fail "no agenda before the morning scene: $(current)"
sleep 1
key Super+Right
until_ok '[ "$(current)" = morning ]' 20 || fail "Super+Right does not show the morning scene: $(current)"
until_ok 'has_text "← 1  Agenda"' 20 || fail "the footer has no line to the previous page"
sleep 1
read -r x y <<<"$(center_of "← 1  Agenda")"
click_at "$x" "$y"
until_ok '[ "$(current)" = agenda ]' 20 || fail "a click on the previous page does not turn back: $(current)"
sleep 1
read -r x y <<<"$(hit_center chapter-head)"
[ -n "$x" ] || fail "the head of the agenda takes no pointer"
move_to "$x" "$y"
until_ok '[ "$(bars)" = 1 ]' 8 || fail "no focus bar over the head of a page"
click_at "$x" "$y"
until_ok '[ "$(current)" = overview ]' 20 || fail "a click on the head does not show the overview: $(current)"
ok "the footer turns back with the mouse and the head of a page opens the page list"

small=$(ctl query "owner:mc kind:text" | jq -r '[.[] | select(.visible and (.rects | length) > 0 and .font_size < 14) | "\(.text):\(.font_size)"] | join(" ")')
[ -z "$small" ] || fail "text under 14 px on the page: $small"
ok "no text on the page is under 14 px"

sans=$(ctl query "owner:mc kind:text" | jq -r '[.[] | select(.visible and (.rects | length) > 0 and (.family | IN("Inter", "TeX Gyre Pagella", "DejaVu Serif", "DejaVu Sans Mono") | not)) | "\(.text):\(.family)"] | join(" ")')
[ -z "$sans" ] || fail "text in a family of the frame that is not Inter or a serif: $sans"
ok "the page is set in Inter and a serif, never in a second sans"

rect_of() {
    ctl query "owner:mc kind:rect" | jq -c --arg s "$1" \
        '[.[] | select((.id | endswith($s)) and (.rects | length) > 0)][0].rects | to_entries[0].value'
}
expect_rect() {
    local got
    got=$(rect_of "$1")
    [ "$got" = "$2" ] || fail "$1 is at $got, not at $2"
}
close_list() { has_text "Show overview" && key Super+F1; sleep 1; }
key Super+Home
until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda to measure"
sleep 1
close_list
expect_rect "right-keys-title-rule" "[888.0,562.0,336.0,1.0]"
expect_rect "right-edge" "[860.0,28.0,1.0,744.0]"
ok "at 1280x800 the column stands at 888 to 1224 with its hairline at 860"

overlaps() {
    ctl query "owner:mc kind:text" | jq -r '
        [.[] | select(.visible and (.rects | length) > 0) | {t: .text, r: (.rects | to_entries[0].value)}] as $a |
        [range(0; $a | length) as $i | range($i + 1; $a | length) as $j |
         ($a[$i].r) as $p | ($a[$j].r) as $q |
         select($p[0] + 1 < $q[0] + $q[2] and $q[0] + 1 < $p[0] + $p[2] and
                $p[1] + 1 < $q[1] + $q[3] and $q[1] + 1 < $p[1] + $p[3]) |
         "\($a[$i].t) / \($a[$j].t)"] | join("; ")'
}
for scene in morning agenda; do
    ctl call mc scene.show "{\"name\": \"$scene\"}" >/dev/null || fail "scene.show $scene"
    until_ok '[ "$(current)" = '"$scene"' ]' 20 || fail "no $scene scene"
    sleep 2
    clash=$(overlaps)
    [ -z "$clash" ] || fail "texts of the $scene scene overlap: $clash"
done
ok "no two texts of the morning scene or of the agenda overlap"

placed_panels() {
    local out
    out=$(ctl query "owner:mc kind:group") || fail "ctl query of the panels failed"
    jq -r --arg p "mc-s-$1-panels-" '
        [.[] | select((.id | startswith($p)) and (.rects | length) > 0 and
            (.rects | to_entries[0].value) as $r | $r[2] > 0 and $r[3] > 0 and
            $r[0] >= 0 and $r[1] >= 0 and $r[0] + $r[2] <= 1280 and $r[1] + $r[3] <= 800)] | length' <<<"$out"
}
for scene in agenda agents morning overview; do
    ctl call mc scene.show "{\"name\": \"$scene\"}" >/dev/null || fail "scene.show $scene"
    until_ok '[ "$(current)" = '"$scene"' ]' 20 || fail "no $scene scene"
    want=$(grep -cE '^- (\{id:|id:)' "$repo/plugins/chora-mc/scenes/$scene.yaml")
    until_ok '[ "$(placed_panels '"$scene"')" = '"$want"' ]' 20 ||
        fail "the $scene scene places $(placed_panels "$scene") of its $want panels at 1280x800"
done
ok "every panel of the agenda, agents, morning and overview scenes is placed at 1280x800"

fits() {
    local w=$1 h=$2 over
    over=$(ctl query "owner:mc kind:text" | jq -r --argjson w "$w" --argjson h "$h" \
        '[.[] | select(.visible and (.rects | length) > 0) | (.rects | to_entries[0].value) as $r |
          select($r[0] < 0 or $r[1] < 0 or $r[0] + $r[2] > $w or $r[1] + $r[3] > $h) | .text] | join("|")')
    [ -z "$over" ] || fail "text beyond the output ${w}x${h}: $over"
}
column_at() {
    local size=$1 left=$2 width=$3 w=${1%x*} h=${1#*x}
    kill "${pids[@]}" 2>/dev/null
    wait 2>/dev/null
    pids=()
    env_file=$(mktemp /tmp/gate-env.XXXXXX) session_start "$size"
    grant screen.capture input.inject
    "$bin/chora-mc" >"$XDG_CONFIG_HOME/mc-$size.log" 2>&1 &
    pids+=($!)
    ctl wait owner:mc --timeout 20 >/dev/null || fail "mc does not start at $size"
    until_ok 'has_text "AGENDA"' || fail "no first screen at $size"
    sleep 1
    expect_rect "right-keys-title-rule" "[$left.0,$((h - 28 - 196 - 14)).0,$width.0,1.0]"
    expect_rect "right-edge" "[$((left - 28)).0,28.0,1.0,$((h - 56)).0]"
    fits "$w" "$h"
    shot "$size"
    ok "at $size the column stands at $left to $((left + width)), nothing is clipped"
}
has_text "Chat with a helper" || fail "the key list does not name the chat"
has_text "Super+a" || fail "the key list does not name Super+a"
key Super+Return
until_ok '[ "$(current)" = terminal ]' 20 || fail "Super+Return does not show the terminal scene: $(current)"
key Super+Home
until_ok '[ "$(current)" = agenda ]' 20 || fail "Super+Home does not show the agenda"
key Super+a
until_ok 'grep -q "chat.show" "$XDG_CONFIG_HOME/mc.log"' 20 || fail "Super+a does not ask the host for the chat"
[ "$(current)" = agenda ] || fail "Super+a left the agenda for $(current)"
drawer=$(ctl call mc region.list '{}' | jq -r '.regions.drawer.contents | length')
[ "$drawer" = 0 ] || fail "the default setup still has contents in the drawer"
ok "Super+a asks for the chat, Super+Home shows the agenda, Super+Return the terminal, and the drawer is empty"

notices() { texts | grep -c '^Note [0-9]$'; }
has_text "NOTIFICATIONS" || fail "no NOTIFICATIONS head on an empty column"
has_text "Nothing new." || fail "the empty notifications do not say so"
ok "an empty column says Nothing new. under its head"
for n in 1 2 3 4; do
    ctl call mc notify.push "{\"title\": \"Note $n\"}" >/dev/null || fail "notify.push"
done
until_ok '[ "$(notices)" = 3 ]' 20 || fail "the column shows $(notices) notifications, not 3: $(texts | tr '\n' '|')"
has_text "Nothing new." && fail "Nothing new. stays beside notifications"
shot notifications
ok "three notifications show, the newest first, and no Nothing new."
ctl call mc notify.clear '{}' >/dev/null
until_ok 'has_text "Nothing new."' 20 || fail "Nothing new. does not come back"

column_at 1280x720 888 336
column_at 1920x1080 1416 448

own_session() {
    kill "${pids[@]}" 2>/dev/null
    wait 2>/dev/null
    pids=()
    printf 'theme: %s\ntest_accept_injected_keys: true\nvoice_hud: own\n' "$1" >"$XDG_CONFIG_HOME/chora/chora-mc.yaml"
    env_file=$(mktemp /tmp/gate-env.XXXXXX) session_start 1280x800
    grant screen.capture input.inject
    "$bin/chora-mc" >"$XDG_CONFIG_HOME/mc-own.log" 2>&1 &
    pids+=($!)
    ctl wait owner:mc --timeout 20 >/dev/null || fail "mc does not start"
    until_ok 'has_text "AGENDA"' || fail "no first screen"
}
text_y() { text_rect "$1" | jq -r '.[1]'; }
text_x() { text_rect "$1" | jq -r '.[0]'; }
own_session light
ctl call mc hud.set '{"state": "Listening", "text": ""}' >/dev/null
for a in 'Atlas working "Tidy notes"' 'Vega thinking ""' 'Orion idle ""' 'Mira done ""' 'Juno idle ""' 'Kai idle ""'; do
    eval set -- "$a"
    ctl call mc agent.set "{\"name\": \"$1\", \"state\": \"$2\", \"task\": \"$3\"}" >/dev/null || fail "agent.set $1"
done
until_ok 'has_text "Listening"' 20 || fail "no state word"
until_ok 'has_text "Atlas"' 20 || fail "no helper row"
until_ok '[ "$(texts | grep -cxF Listening)" = 1 ]' 20 || fail "not one state word"
[ "$(text_x Listening)" = 888.0 ] || fail "the state word is not at the left edge of the column: $(text_rect Listening)"
for name in Atlas Vega Orion Mira; do
    has_text "$name" || fail "no row for $name"
    [ "$(text_x "$name" | jq '. >= 888')" = true ] || fail "$name is not in the column: $(text_rect "$name")"
done
has_text "Juno" && fail "a fifth helper shows"
has_text "+2 more" || fail "no +2 more under four helpers"
[ "$(text_y Atlas | jq '. > 40')" = true ] || fail "Atlas stands in the header: $(text_rect Atlas)"
header=$(ctl query "owner:mc kind:text" | jq -r '[.[] | select(.visible and (.rects | length) > 0 and (.rects | to_entries[0].value[1]) < 56 and (.rects | to_entries[0].value[0]) < 860) | .text] | length')
[ "$header" = 2 ] || fail "the header holds $header texts, not the running head and the clock"
shot helpers-light
ok "state and helpers stand in the column, four rows and +2 more, the header holds the head and the clock"

host_session() {
    kill "${pids[@]}" 2>/dev/null
    wait 2>/dev/null
    pids=()
    printf 'theme: %s\ntest_accept_injected_keys: true\nvoice_hud: host\n' "$1" >"$XDG_CONFIG_HOME/chora/chora-mc.yaml"
    printf 'enabled: true\nposition: null\nhud: mc\n' >"$XDG_CONFIG_HOME/chora/chora-assistant.yaml"
    export XDG_DATA_HOME=$XDG_CONFIG_HOME/data
    mkdir -p "$XDG_DATA_HOME/chora/huds"
    rm -rf "$XDG_DATA_HOME/chora/huds/mc"
    cp -rL "$repo/plugins/chora-mc/hud" "$XDG_DATA_HOME/chora/huds/mc"
    env_file=$(mktemp /tmp/gate-env.XXXXXX) session_start 1280x800
    grant screen.capture input.inject methods.call
    "$bin/chora-mc" >"$XDG_CONFIG_HOME/mc-host.log" 2>&1 &
    pids+=($!)
    ctl wait owner:mc --timeout 20 >/dev/null || fail "mc does not start"
    "$bin/chora-assistant" >"$XDG_CONFIG_HOME/assistant.log" 2>&1 &
    pids+=($!)
    ctl wait owner:assistant --timeout 20 >/dev/null || fail "the host does not start: $(tail -n 3 "$XDG_CONFIG_HOME/assistant.log")"
    until_ok 'has_text "AGENDA"' || fail "no first screen"
    sleep 2
}
page_shot() { ctl capture "$1" --node mc-s-agenda >/dev/null || fail "capture of the page"; }
chat_session() {
    local theme=$1 before=$XDG_CONFIG_HOME/page-before-$1.png after=$XDG_CONFIG_HOME/page-after-$1.png
    host_session "$theme"
    has_text "Chat with a helper ..." || fail "no door of the chat in the column"
    has_text "Back one step" || fail "no key list before the chat"
    ctl call mc notify.push '{"title": "Note 1"}' >/dev/null
    ctl call mc notify.push '{"title": "Note 2"}' >/dev/null
    sleep 1
    page_shot "$before"
    ctl call assistant chat.show '{"name": "atlas"}' >/dev/null || fail "chat.show"
    until_ok 'ctl query id:hud-chat | jq -e "length > 0" >/dev/null' 40 || fail "the host draws no chat"
    sleep 1
    page_shot "$after"
    [ "$(python3 -I "$repo/tests/png-same.py" "$before" "$after")" = "0 0" ] || fail "the page changed when the chat opened"
    chat=$(ctl query id:hud-chat | jq -c '.[0].rects | to_entries[0].value')
    jq -ne --argjson c "$chat" '$c[0] >= 888 and $c[0] + $c[2] <= 1224 and $c[1] > 56' >/dev/null ||
        fail "the chat is not in the column: $chat"
    until_ok '! has_text "Back one step"' 20 || fail "the key list stays beside the open chat"
    has_text "NOTIFICATIONS  2" || fail "the notifications do not fold into their count: $(texts | tr '\n' '|')"
    texts | grep -q "Note 2" && fail "a notification stays open beside the chat"
    shot "chat-$theme"
    ok "the open chat fills the column ($chat), the page is the same, the keys go and the notifications fold ($theme)"
    ctl call assistant chat.hide '{}' >/dev/null || fail "chat.hide"
    until_ok 'has_text "Back one step"' 40 || fail "the column does not come back"
    until_ok 'texts | grep -q "Note 2"' 20 || fail "the notifications do not unfold"
    ok "closing the chat brings the key list and the notifications back"
}
chat_session light
chat_session dark
