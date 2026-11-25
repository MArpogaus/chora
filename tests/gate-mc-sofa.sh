#!/bin/bash
# Gate of the sofa and kitchen scenarios of chora-mc (journey 1 of the rc 11
# review, mc-32 to mc-37): chora-mc --manage-windows with the HUD host on
# a headless output of 1280x720, as the tour starts it, in one theme.
#
#   remote      after a helper changed the content of the pages and with no
#               click, Right and Left of the remote turn the page and Escape
#               goes back one step (mc-32)
#
#   wide        Super+n hides the column and the page takes its room; Super+n
#               gives it back (mc-37)
#
#   door        a press of the mouse on the chat door in the column opens the
#               chat, also while a helper works and its row has moved the
#               door down (mc-33)
#
#   status      the word Mute of mc stands beside its state word and never
#               under the state word of the host (mc-36)
#
#   empty-head  the column that came back after Super+n twice draws no head
#               over a text without lines (mc-35)
#
#   answer      the answer that the host draws beside the state word is
#               at least 14 px high in its text, not the 11 px of the
#               host, and its ink is no grey: 7 to 1 on the page (mc-34)
#
#   answer-long an answer of 150 characters takes at most two lines and
#               covers neither the date, the page title row nor the
#               column (mc-39)
#
#   pill        the word of the host beside the state word never says
#               No helper while the voice agent speaks or a helper works
#               (mc-36)
#
#   alarm       with chora-notify as the server of the notices, two
#               critical notices leave the column whole: no NOTIFICATIONS
#               or empty NOTES head, the Chat door stays above the rule of
#               KEYS, and the alarm is a card in full that stands over
#               the page, not in the footer line (mc-35)
#
#   launcher-none  Super+Space and a program name that does not exist, then
#               Return: the list shows again under the row "Nothing found
#               for ..." and does not close without a word, as in xp and
#               tiling (mc-44)
#
#   launcher-click-row  the real fuzzel lists Chat with a helper, Foot and
#               Screenshot, 19 px apart from y 237 on 1280x720; a click on the
#               row Foot starts that program as Return does, and its window
#               has a page (mc-47)
#
#   card-size   the text of the card of chora-notify is at least as high
#               as the body of the page, and its ink is no grey: 7 to 1
#               (mc-40)
#
#   remote-back a remote without Super leaves a window that has the keys:
#               a program that the launcher started, then one press of
#               XF86Back after the Escape that the shell got shows the
#               plain arrows in the key list and gives the keyboard to the page of the window,
#               where Left turns the page, and XF86HomePage goes to the
#               first page, which holds the keyboard (mc-46)
#
# Usage: tests/gate-mc-sofa.sh   BIN: directory with the binaries
#   THEME  light or dark (default dark)
#   CELLS  a regular expression of the cells that run (default all)
#   SHOTS  a directory that keeps one capture of each cell as
#          mc-sofa-CELL-THEME.png
# shellcheck disable=SC2016 # until_ok evaluates its first argument
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-mc chora-assistant chora-chat chora-notify
need jq
need foot
fuzzel_old=
if ! command -v fuzzel >/dev/null 2>&1; then
    fuzzel_old="fuzzel is not installed here"
elif fuzzel_older 1.11; then
    fuzzel_old="fuzzel $fuzzel_version < 1.11"
fi
export PATH=$bin:$PATH
export CHORA_BIN=$bin
export LANG=C.UTF-8 LC_ALL=C.UTF-8
unset DBUS_SESSION_BUS_ADDRESS
theme=${THEME:-dark}
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
move_to() { echo "[{\"pointer_move\": {\"position\": [$1, $2]}}]" | ctl inject >/dev/null || fail "inject move"; }
emit() { echo "$1 $2" >&3; }
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject $1"; }
shown='select(.visible and (.rects | length) > 0)'
texts() { ctl query "owner:mc kind:text" | jq -r ".[] | $shown | .text"; }
has_text() { texts | grep -qxF "$1"; }
current() { ctl call mc scene.list '{}' | jq -r '.current'; }
shot() {
    [ -n "${SHOTS:-}" ] || return 0
    mkdir -p "$SHOTS"
    sleep 1.2
    ctl capture "$SHOTS/mc-sofa-$1-$theme.png" >/dev/null || fail "capture $1"
}

printf 'theme: %s\ntest_accept_injected_keys: true\n' "$theme" >"$work/chora/chora-mc.yaml"
cat >"$work/entries.json" <<'JSON'
[{"seq": 1, "role": "user", "text": "Please tidy the notes for Friday."},
 {"seq": 2, "role": "agent", "text": "I will go through your notes and group them by day."}]
JSON
mkdir -p "$work/fake"
real_fuzzel=$(command -v fuzzel)
cat >"$work/fake/fuzzel" <<FAKE
#!/bin/sh
[ ! -e "$work/fake/real" ] || exec "$real_fuzzel" "\$@"
n=\$((\$(cat "$work/fake/calls" 2>/dev/null || echo 0) + 1))
echo "\$n" >"$work/fake/calls"
case " \$* " in *" --dmenu "*) cat >"$work/fake/rows.\$n" ;; esac
case " \$* " in *" --dmenu "*)
    if [ -s "$work/fake/pick" ]; then cat "$work/fake/pick"; rm "$work/fake/pick"
    elif [ "\$n" = 1 ]; then echo nosuchprogram; fi ;;
esac
FAKE
chmod +x "$work/fake/fuzzel"
export PATH=$work/fake:$PATH
mkdir -p "$work/data/applications"
printf '[Desktop Entry]\nType=Application\nName=Foot\nExec=foot --app-id=mcclick\n' >"$work/data/applications/mcclick.desktop"
export XDG_DATA_HOME=$work/data XDG_DATA_DIRS=$work/data
CHORA_TEST_FLAGS=--test-injected-press-is-real session_start 1280x720 -- "$bin/chora-mc" --manage-windows
grant screen.capture input.inject methods.call
mkfifo "$work/voice.in"
FAKE_VOICE_KEYS=1 FAKE_VOICE_ENTRIES=$work/entries.json python3 -I "$repo/tests/fake-voice.py" "$work/claims" \
    <"$work/voice.in" >"$work/voice.log" 2>&1 &
pids+=($!)
exec 3>"$work/voice.in"
ctl wait owner:mc --timeout 20 >/dev/null || fail "mc does not start"
notify_slot=$repo/plugins/chora-mc/notify
"$bin/chora-notify" --assets "$notify_slot" --config "$notify_slot/notify.yaml" >"$work/notify.log" 2>&1 &
pids+=($!)
"$bin/chora-assistant" >"$work/assistant.log" 2>&1 &
pids+=($!)
ctl wait owner:assistant --timeout 20 >/dev/null || fail "the host does not start: $(tail -n 3 "$work/assistant.log")"
until_ok 'has_text "AGENDA"' || fail "no first page"
sleep 5
panel() { ctl call mc panel.set "{\"scene\": \"$1\", \"panel\": \"$2\", \"lines\": $3}" >/dev/null || fail "panel.set $1 $2"; }
panel agenda today '["18:30\tDinner with the family", "19:30\tNews on the television", "20:00\tCall Anna"]'
panel morning lead '["Good evening", "Anna will call at eight."]'
panel morning weather '["Tomorrow", "Zurich, 11 degrees, dry."]'
panel morning mail '["Post", "One card from Anna."]'
sleep 2

cell_remote() {
    move_to 640 360
    sleep 1
    key Right
    until_ok '[ "$(current)" = morning ]' 20 || fail "Right does not turn to page 2: $(current)"
    key Right
    until_ok '[ "$(current)" = agents ]' 20 || fail "a second Right does not turn to page 3: $(current)"
    key Left
    until_ok '[ "$(current)" = morning ]' 20 || fail "Left does not turn back to page 2: $(current)"
    key Escape
    until_ok '[ "$(current)" = agents ]' 20 || fail "Escape does not go back one step: $(current)"
    shot remote
}

lead_width() {
    ctl query "owner:mc kind:text" | jq -r --arg t "$1" \
        '[.[] | select(.visible and (.rects | length) > 0 and (.text | startswith($t)))][0].rects | to_entries[0].value[2]'
}
cell_wide() {
    local before after
    key Super+Home
    key Super+Right
    until_ok '[ "$(current)" = morning ]' 20 || fail "no page 2: $(current)"
    panel morning lead '["Boil the water", "Fill the large pot with two litres of water, add a spoon of salt and bring it to a rolling boil, then add the spaghetti and cook it for nine minutes."]'
    sleep 1
    before=$(lead_width "Boil the water")
    shot wide-column
    key Super+n
    until_ok '! has_text "Back one step"' 20 || fail "Super+n does not hide the column"
    sleep 1.5
    after=$(lead_width "Boil the water")
    shot wide-page
    [ "$(jq -n "$after > $before + 100")" = true ] || fail "the page does not widen when the column hides: $before then $after"
    key Super+n
    until_ok 'has_text "Back one step"' 20 || fail "Super+n does not bring the column back"
    sleep 1.5
    [ "$(lead_width "Boil the water")" = "$before" ] || fail "the page does not narrow when the column is back: $(lead_width "Boil the water") not $before"
}

click_rect() {
    local x y
    x=$(jq -n --argjson r "$1" '$r[0] + ([$r[2] / 2, 40] | min) | floor')
    y=$(jq -n --argjson r "$1" '$r[1] + $r[3] / 2 | floor')
    move_to "$x" "$y"
    sleep 0.4
    echo '[{"button": {"state": "pressed"}}, {"button": {"state": "released"}}]' |
        ctl inject >/dev/null || fail "inject click"
}
text_rect() {
    ctl query "owner:mc kind:text" | jq -c --arg t "$1" \
        '[.[] | select(.visible and (.rects | length) > 0 and .text == $t)][0].rects | to_entries[0].value'
}
doors_under_chat() {
    ctl query "owner:mc kind:text" | jq -r --argjson c "$(node_rect hud-chat)" \
        ".[] | $shown | select(.id | test(\"-doors-\")) | .rects | to_entries[0].value |
         select(.[0] < \$c[0] + \$c[2] and \$c[0] < .[0] + .[2] and .[1] < \$c[1] + \$c[3] and \$c[1] < .[1] + .[3]) | tostring"
}
hits_under_chat() {
    ctl query "owner:mc kind:rect" | jq -r --argjson c "$(node_rect hud-chat)" \
        ".[] | $shown | select(.id | endswith(\"-hit\")) | .rects | to_entries[0].value |
         select(.[2] > 1 and .[3] > 1 and .[0] < \$c[0] + \$c[2] and \$c[0] < .[0] + .[2] and .[1] < \$c[1] + \$c[3] and \$c[1] < .[1] + .[3]) | tostring"
}
cell_door() {
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    emit voice.state '{"name": "Atlas", "state": "working", "task": "Reminder", "step": 1, "steps": 1, "seq": 4}'
    until_ok 'has_text "HELPERS"' 20 || fail "no helper row: $(texts | tr '\n' '|')"
    sleep 1
    click_rect "$(text_rect "Chat with a helper ...")"
    until_ok 'ctl query id:hud-chat | jq -e "length > 0" >/dev/null' 20 || fail "a press on the chat door opens no chat: $(texts | tr '\n' '|')"
    until_ok '[ -z "$(doors_under_chat)" ]' 40 || fail "the chat stays over a door of the column (mc-56): $(doors_under_chat | tr '\n' ' ')"
    [ -z "$(hits_under_chat)" ] || fail "a hit area of mc lies under the open chat (mc-56): $(hits_under_chat | tr '\n' ' ')"
    shot door
    key Escape
    until_ok '! ctl query id:hud-chat | jq -e "length > 0" >/dev/null' 20 || fail "Escape does not close the chat"
}

node_rect() { ctl query "id:$1" | jq -c '.[0].rects | to_entries[0].value'; }
rect_of() {
    ctl query "owner:mc kind:rect" | jq -c --arg s "$1" \
        '[.[] | select((.id | endswith($s)) and (.rects | length) > 0)][0].rects | to_entries[0].value'
}
cell_status() {
    local word mute
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    emit voice.state '{"state": "listening", "task": "", "said": "", "muted": {"mic": false, "speaker": false}}'
    ctl call mc hud.set '{"state": "speaking", "text": ""}' >/dev/null || fail "hud.set"
    until_ok 'has_text "Mute" && has_text "speaking"' 20 || fail "no state word or no Mute: $(texts | tr '\n' '|')"
    until_ok 'ctl query id:hud-word | jq -e "length > 0" >/dev/null' 20 || fail "the host draws no state word"
    sleep 1
    word=$(node_rect hud-word)
    mute=$(text_rect Mute)
    shot status
    jq -ne --argjson w "$word" --argjson m "$mute" '$m[0] + $m[2] <= $w[0]' >/dev/null ||
        fail "Mute $mute stands under the state word of the host $word"
}

cell_empty_head() {
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    key Super+n
    until_ok '! has_text "Back one step"' 20 || fail "Super+n does not hide the column"
    key Super+n
    until_ok 'has_text "Back one step"' 20 || fail "Super+n does not bring the column back"
    sleep 1
    has_text "NOTES" && fail "an empty head NOTES stands in the column: $(texts | tr '\n' '|')"
    shot empty-head
}

cell_answer() {
    local rect contrast
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    emit voice.state '{"state": "listening", "task": "", "said": "", "muted": {"mic": false, "speaker": false}}'
    emit voice.say '{"text": "It is eight in the evening.", "duration_ms": 60000}'
    until_ok 'ctl query id:hud-line | jq -e "length > 0" >/dev/null' 20 || fail "the host draws no answer"
    sleep 1
    rect=$(node_rect hud-line)
    shot answer
    jq -ne --argjson r "$rect" '$r[3] >= 30' >/dev/null ||
        fail "the answer of the host is $rect, a line of the small 11 px text"
    ctl capture "$work/answer.png" >/dev/null || fail "capture answer"
    # shellcheck disable=SC2046 # four words, one for each number of the rect
    contrast=$(python3 -I "$repo/tests/png-contrast.py" "$work/answer.png" \
        $(jq -r '"\(.[0] | floor) \(.[1] | floor) \(.[2] | floor) \(.[3] | floor)"' <<<"$rect"))
    [ "$(jq -n "${contrast%% *} >= 7")" = true ] || fail "the ink of the answer is grey: $contrast"
}

cell_answer_long() {
    local text="The water has been boiling for nine minutes, so please take the pot off the stove and drain the spaghetti into the colander. Keep a cup of that water."
    local rect date title
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    emit voice.state '{"state": "listening", "task": "", "said": "", "muted": {"mic": false, "speaker": false}}'
    emit voice.say "{\"text\": \"$text\", \"duration_ms\": 60000}"
    until_ok 'ctl query id:hud-line | jq -e "length > 0" >/dev/null' 20 || fail "the host draws no answer"
    sleep 1
    rect=$(node_rect hud-line)
    shot answer-long
    date=$(ctl query "owner:mc kind:text" | jq -c '[.[] | select(.visible and (.rects | length) > 0 and (.text | test("^[A-Z]+ [0-9]+ [A-Z]+ +[0-9]+:[0-9]+$")))][0].rects | to_entries[0].value')
    title=$(text_rect AGENDA)
    [ "$date" != null ] || fail "no date in the header: $(texts | tr '\n' '|')"
    jq -ne --argjson r "$rect" '$r[3] <= 2 * 28 + 2 * 4' >/dev/null ||
        fail "the answer of 150 characters is $rect, more than two lines"
    jq -ne --argjson r "$rect" '$r[0] + $r[2] <= 860' >/dev/null ||
        fail "the answer $rect reaches into the column (x from 860)"
    jq -ne --argjson r "$rect" --argjson d "$date" \
        '($r[0] + $r[2] <= $d[0]) or ($d[0] + $d[2] <= $r[0]) or ($r[1] + $r[3] <= $d[1]) or ($d[1] + $d[3] <= $r[1])' >/dev/null ||
        fail "the answer $rect covers the date $date"
    jq -ne --argjson r "$rect" --argjson d "$title" \
        '($r[0] + $r[2] <= $d[0]) or ($d[0] + $d[2] <= $r[0]) or ($r[1] + $r[3] <= $d[1]) or ($d[1] + $d[3] <= $r[1])' >/dev/null ||
        fail "the answer $rect covers the page title $title"
}

hud_word() { ctl query id:hud-word | jq -r '.[0] | (.text // .meta.text // "")'; }
cell_pill() {
    local word
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    emit voice.state '{"state": "speaking", "task": "", "said": "It is eight.", "muted": {"mic": false, "speaker": false}}'
    until_ok '[ "$(hud_word)" = Speaking ]' 20 || fail "the voice agent speaks, the word beside the state word is $(hud_word)"
    shot pill
    emit voice.state '{"name": "Atlas", "state": "working", "task": "Reminder", "step": 1, "steps": 1, "seq": 6}'
    until_ok 'has_text "HELPERS"' 20 || fail "no helper row: $(texts | tr '\n' '|')"
    word=$(hud_word)
    [ "$word" != "No helper" ] || fail "a helper works, the word is $word"
    emit voice.state '{"name": "Atlas", "state": "idle", "task": "", "seq": 7}'
}

cell_alarm() {
    local door rule card foot body="The disk of the notes is almost full, so the copy of last night stopped half way."
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    emit voice.state '{"name": "Atlas", "state": "working", "task": "Reminder", "step": 1, "steps": 1, "seq": 8}'
    until_ok 'has_text "HELPERS"' 20 || fail "no helper row: $(texts | tr '\n' '|')"
    ctl call mc notify.clear '{}' >/dev/null
    ctl call mc notify.push "{\"title\": \"Backup failed\", \"body\": \"$body\", \"urgency\": \"critical\"}" >/dev/null || fail "notify.push"
    ctl call mc notify.push '{"title": "Update broke", "body": "The update of the voice agent did not finish and the old one runs again, please look at it.", "urgency": "critical"}' >/dev/null || fail "notify.push"
    until_ok '[ "$(ctl call notify state.get "{}" | jq "[.shown[].summary] | length")" -ge 1 ]' 20 || fail "chora-notify shows no alarm"
    sleep 1.5
    shot alarm
    has_text "NOTES" && fail "an empty head NOTES stands in the column: $(texts | tr '\n' '|')"
    has_text "NOTIFICATIONS" && fail "mc draws a column of notifications: $(texts | tr '\n' '|')"
    door=$(text_rect "Chat with a helper ...")
    rule=$(rect_of right-keys-title-rule)
    jq -ne --argjson d "$door" --argjson r "$rule" '$d[1] + $d[3] <= $r[1]' >/dev/null ||
        fail "the Chat door $door stands on the rule of KEYS $rule"
    jq -ne --argjson d "$door" '$d[0] >= 860 and $d[0] + $d[2] <= 1230' >/dev/null || fail "the Chat door $door leaves the column"
    [ "$(ctl call notify state.get '{}' | jq -r '.shown[0].body')" = "$body" ] || fail "the card of the alarm is cut"
    card=$(ctl call notify state.get '{}' | jq -c '.shown[0].rect')
    foot=$(text_rect "→ 2  The Morning")
    jq -ne --argjson c "$card" --argjson f "$foot" '$c[0] + $c[2] <= 860 and $c[1] + $c[3] <= $f[1]' >/dev/null ||
        fail "the card of the alarm $card is not over the page, above the footer $foot"
    ctl call mc notify.clear '{}' >/dev/null
}

cell_card_size() {
    local body="The disk of the notes is almost full, so the copy of last night stopped half way." card page heights ink
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    ctl call mc notify.clear '{}' >/dev/null
    ctl call mc notify.push "{\"title\": \"Backup failed\", \"body\": \"$body\", \"urgency\": \"critical\"}" >/dev/null || fail "notify.push"
    until_ok '[ "$(ctl call notify state.get "{}" | jq "[.shown[].summary] | length")" -ge 1 ]' 20 || fail "chora-notify shows no card"
    sleep 1.5
    shot card-size
    card=$(ctl call notify state.get '{}' | jq -c '.shown[0].rect')
    page=$(text_rect "Dinner with the family")
    ctl capture "$work/card.png" >/dev/null || fail "capture card"
    # shellcheck disable=SC2046 # four words, one for each number of the rect
    heights=$(python3 -I "$repo/tests/png-ink.py" "$work/card.png" \
        $(jq -r '"\(.[0] | floor + 6) \(.[1] | floor + 6) \(.[2] | floor - 36) \(.[3] | floor - 12)"' <<<"$card"))
    # shellcheck disable=SC2046 # four words, one for each number of the rect
    ink=$(python3 -I "$repo/tests/png-ink.py" "$work/card.png" \
        $(jq -r '"\(.[0] | floor) \(.[1] | floor) \(.[2] | floor) \(.[3] | floor)"' <<<"$page"))
    jq -ne --arg c "$heights" --arg p "$ink" \
        '($c | split(" ") | .[1:] | map(tonumber) | max) >= ($p | split(" ") | map(tonumber) | max) - 2' >/dev/null ||
        fail "the text of the card has lines of ink $heights, the page body $ink"
    # shellcheck disable=SC2046 # four words, one for each number of the rect
    contrast=$(python3 -I "$repo/tests/png-contrast.py" "$work/card.png" \
        $(jq -r '"\(.[0] | floor + 6) \(.[1] | floor + 6) \(.[2] | floor - 36) \(.[3] | floor - 12)"' <<<"$card"))
    [ "$(jq -n "${contrast%% *} >= 7")" = true ] || fail "the ink of the card is grey: $contrast"
    ctl call mc notify.clear '{}' >/dev/null
}

cell_launcher_none() {
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    rm -f "$work/fake/calls" "$work"/fake/rows.*
    key Super+space
    until_ok '[ -s "$work/fake/rows.2" ]' 40 || fail "a program that does not exist closes the launcher without a word: calls $(cat "$work/fake/calls" 2>/dev/null), rows: $(echo "$work"/fake/rows.*)"
    [ "$(head -n 1 "$work/fake/rows.2")" = 'Nothing found for "nosuchprogram"' ] ||
        fail "the list after the unknown name does not say so: $(head -n 1 "$work/fake/rows.2")"
}

remote_alive() { session_pids -f -- "--app-id=mcremote" >/dev/null; }
remote_end() {
    local pid
    for pid in $(session_pids -f -- "--app-id=mcremote"); do kill "$pid"; done
    until_ok '! remote_alive' 20
}
cell_remote_back() {
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    echo "foot --app-id=mcremote" >"$work/fake/pick"
    key Super+space
    until_ok 'remote_alive && [ "$(current)" = mcremote ]' 40 || { remote_end; fail "the window of the launcher has no page: $(current)"; }
    sleep 2
    key Escape
    sleep 1
    texts | grep -qE '^Super\+.*←' || { remote_end; fail "the window has the keys, the key list names no Super: $(texts | grep -F '←' | tr '\n' '|')"; }
    key XF86Back
    sleep 2
    texts | grep -qE '^←.*→$' || { remote_end; fail "after one XF86Back the key list does not name the plain arrows: $(texts | grep -F '←' | tr '\n' '|')"; }
    [ "$(current)" = mcremote ] || { remote_end; fail "XF86Back left the page of the window: $(current)"; }
    key Left
    until_ok '[ "$(current)" != mcremote ]' 40 || { remote_end; fail "after XF86Back the page does not hold the keyboard, Left does not turn it"; }
    ctl call mc scene.show '{"name": "mcremote"}' >/dev/null || { remote_end; fail "scene.show of the window page"; }
    until_ok '[ "$(current)" = mcremote ]' 20 || { remote_end; fail "no page of the window again: $(current)"; }
    sleep 2
    key XF86HomePage
    until_ok '[ "$(current)" = agenda ]' 40 || { remote_end; fail "XF86HomePage in the window does not go to the first page: $(current)"; }
    sleep 1
    key Right
    until_ok '[ "$(current)" = morning ]' 40 || { remote_end; fail "after XF86HomePage the page does not hold the keyboard: $(current)"; }
    shot remote-back
    remote_end
}

click_at() {
    move_to "$1" "$2"
    sleep 0.4
    echo '[{"button": {"state": "pressed"}}, {"button": {"state": "released"}}]' |
        ctl inject >/dev/null || fail "inject click"
}
click_alive() { session_pids -f -- "--app-id=mcclick" >/dev/null; }
click_end() {
    local pid
    for pid in $(session_pids -f -- "--app-id=mcclick"); do kill "$pid"; done
    : >/dev/null
}
foot_row_x=540
foot_row_y=256
cell_launcher_click_row() {
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    : >"$work/fake/real"
    key Super+space
    sleep 3
    shot launcher-click-row
    click_at "$foot_row_x" "$foot_row_y"
    until_ok 'click_alive' 40 || { rm -f "$work/fake/real"; key Escape; fail "a click on the row Foot starts no program (Return does)"; }
    until_ok '[ "$(current)" = mcclick ]' 40 || { click_end; rm -f "$work/fake/real"; fail "the window of the program has no page: $(current)"; }
    click_end
    rm -f "$work/fake/real"
}

off_screen() {
    ctl query "owner:mc kind:text" | jq -r ".[] | $shown | .rects | to_entries[0].value |
        select(.[0] < 0 or .[1] < 0 or .[0] + .[2] > 1280 or .[1] + .[3] > 720) | tostring"
    ctl query "owner:mc kind:rect" | jq -r ".[] | $shown | select(.id | endswith(\"-hit\")) | .rects | to_entries[0].value |
        select(.[2] > 1 and .[3] > 1 and (.[0] < 0 or .[1] < 0 or .[0] + .[2] > 1280 or .[1] + .[3] > 720)) | tostring"
}
struck_texts() {
    ctl query "owner:mc kind:rect" | jq -r ".[] | $shown | select(.rects | to_entries[0].value | .[3] <= 2 and .[2] > 8) |
        {id: .id, r: (.rects | to_entries[0].value)}" | jq -rs --argjson t "$(ctl query "owner:mc kind:text" | jq -c "[.[] | $shown | {t: .text, r: (.rects | to_entries[0].value)}]")" '
        [.[] | . as $h | $t[] | select($h.r[0] < .r[0] + .r[2] and .r[0] < $h.r[0] + $h.r[2] and $h.r[1] > .r[1] + 3 and $h.r[1] < .r[1] + .r[3] - 3) | "\($h.id) strikes \(.t)"] | join("; ")'
}
cell_keylist() {
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    emit voice.state '{"state": "listening", "task": "", "said": "", "muted": {"mic": false, "speaker": false}}'
    ctl call mc hud.set '{"state": "listening", "text": ""}' >/dev/null || fail "hud.set"
    until_ok 'has_text "Mute"' 20 || fail "no Mute: $(texts | tr '\n' '|')"
    key Super+F1
    until_ok 'has_text "Show overview"' 20 || fail "Super+F1 does not open the full list"
    sleep 1.5
    shot keylist
    [ -z "$(struck_texts)" ] || fail "a rule of the key list strikes a text (rpx1-9): $(struck_texts)"
    for t in "Take a screenshot" "Back one step" "Power menu" "Lock the screen"; do
        has_text "$t" || fail "the full list lacks the row \"$t\" at 1280x720"
    done
    [ -z "$(off_screen)" ] || fail "text or a hit area of the full list is off the 1280x720 output (rpx1-9): $(off_screen | tr '\n' ' ')"
    key Super+F1
    until_ok '! has_text "Show overview"' 20 || fail "Super+F1 does not close the list"
}

cell_page_end() {
    local card page foot
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    key Super+Right
    until_ok '[ "$(current)" = morning ]' 20 || fail "no morning: $(current)"
    panel morning overnight '["Then", "Toss the drained spaghetti in the pan with a splash of the pasta water and the parsley, then serve it at once with a little grated cheese on top."]'
    ctl call mc notify.clear '{}' >/dev/null
    ctl call mc notify.push '{"title": "Timer: spaghetti, 9 minutes", "body": "Started by voice"}' >/dev/null || fail "notify.push"
    until_ok '[ "$(ctl call notify state.get "{}" | jq "[.shown[].summary] | length")" -ge 1 ]' 20 || fail "no card"
    sleep 2
    shot page-end
    card=$(ctl call notify state.get '{}' | jq -c '.shown[0].rect')
    page=$(node_rect mc-s-morning-page)
    foot=$(text_rect "→ 3  Helpers")
    jq -ne --argjson c "$card" --argjson p "$page" '$p[1] + $p[3] <= $c[1]' >/dev/null ||
        fail "the page $page reaches under the timer card $card (rpx1-11): the page does not reserve its height"
    jq -ne --argjson f "$foot" --argjson p "$page" '$p[1] + $p[3] <= $f[1]' >/dev/null ||
        fail "the page $page reaches under the footer line $foot (rpx1-12)"
    ctl call mc notify.clear '{}' >/dev/null
    sleep 1.5
    ctl capture "$work/page-end.png" >/dev/null || fail "capture page-end"
    # shellcheck disable=SC2046 # four words, one for each number of the rect
    cut=$(python3 -I "$repo/tests/png-ink.py" "$work/page-end.png" \
        $(jq -r '"160 \(.[1] + .[3] - 2 | floor) 664 2"' <<<"$page"))
    [ -z "$cut" ] || fail "a line of the page is cut at the edge of the page $page: ink $cut in its last two rows (rpx1-12)"
}

cell_pill_clock() {
    local clock word
    key Super+Home
    until_ok '[ "$(current)" = agenda ]' 20 || fail "no agenda: $(current)"
    emit voice.state '{"state": "listening", "task": "", "said": "", "muted": {"mic": false, "speaker": false}}'
    until_ok 'ctl query id:hud-word | jq -e "length > 0" >/dev/null' 20 || fail "the host draws no state word"
    key Super+n
    until_ok '! has_text "Back one step"' 20 || fail "Super+n does not hide the column"
    sleep 1.5
    clock=$(ctl query "owner:mc kind:text" | jq -c "[.[] | $shown | select(.text | test(\"^[A-Z]+ [0-9]+ [A-Z]+ +[0-9]+:[0-9]+\$\"))][0].rects | to_entries[0].value")
    word=$(node_rect hud-word)
    shot pill-clock
    key Super+n
    until_ok 'has_text "Back one step"' 20 || fail "Super+n does not bring the column back"
    jq -ne --argjson c "$clock" --argjson w "$word" '$c[0] + $c[2] <= $w[0] - 8 or $c[1] + $c[3] <= $w[1] or $w[1] + $w[3] <= $c[1]' >/dev/null ||
        fail "the state pill $word lies over the clock $clock when the column is hidden (rpx1-13)"
}

cell_helpers_empty() {
    local name words
    for name in $(ctl call mc agent.list '{}' | jq -r '.agents[].name'); do
        ctl call mc agent.remove "{\"name\": \"$name\"}" >/dev/null
    done
    ctl call mc scene.show '{"name": "agents"}' >/dev/null || fail "scene.show agents"
    until_ok '[ "$(current)" = agents ]' 20 || fail "no helpers page: $(current)"
    sleep 1.5
    shot helpers-empty
    words=$(ctl query "owner:mc kind:text" | jq -r "[.[] | $shown | select((.rects | to_entries[0].value | .[0] < 860 and .[1] > 150) and (.text | test(\"helper\"; \"i\"))) | \"\\(.family)|\\(.text)\"] | join(\"; \")")
    [ -n "$words" ] || fail "the helpers page without a helper says nothing in words (rpx1-14): $(texts | tr '\n' '|')"
    case "$words" in *Pagella* | *Serif*) ;; *) fail "the words of the empty helpers page are not in the page type: $words" ;; esac
}

failed=()
skipped=()
ran=0
run_cell() {
    local name=$1
    [[ $name =~ ${CELLS:-.} ]] || return 0
    if [ "$name" = launcher-click-row ] && [ -n "$fuzzel_old" ]; then
        cell_skip "$name" "$fuzzel_old"
        skipped+=("$name")
        return 0
    fi
    if ("cell_${name//-/_}"); then
        ran=$((ran + 1))
        ok "$name ($theme)"
    else
        failed+=("$name")
    fi
}
cells=(remote wide door status empty-head answer answer-long pill alarm card-size launcher-none keylist page-end pill-clock helpers-empty remote-back launcher-click-row)
for name in "${cells[@]}"; do
    run_cell "$name"
done
[ ${#failed[@]} = 0 ] || fail "cells failed: ${failed[*]}"
cells_end "$ran" "${skipped[*]}"
[ ${#skipped[@]} != 0 ] || ok "every cell of the mc sofa gate ($theme)"
