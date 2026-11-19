# flake-k: hud flake of train 73b (F5)

## r1

Lane flake-k r1, branch claude/flake-k from 219d5afe, head 9dde8523. One file: tests/gate-hud.sh. No product fault.

Cells: construct-host-hud and chat-long-task of tests/gate-hud.sh (73b, `--jobs 3`).

### Reproduction (cells construct-host-hud, construct-chat-panel, chat-long-task in one run, serial runs)

Load: 14 busy loops plus a looping `cargo build --workspace` of a second target dir on 4 cores (load average 20).

| | construct-host-hud | chat-long-task |
|---|---|---|
| before, 30 runs | 11 red ("no card for the detail of 6 KiB", 5 s) | 11 red ("A question is waiting."), the same 11 runs |
| before, load 6 busy loops plus cargo, 30 runs | 0 red | 0 red |
| before, light load, 41 runs (3 probe loops) | 3 red ("a done of another id closed the question", cell over in 8 s) | 3 red (follow-up) |
| after, 30 runs, same load of 20 | 0 red, no timeout line at all | 0 red |

hud step through merge-gate: green in the gate (`--jobs 3 --only pre-commit,ci-check,build,test,gate-lib,hud`: pre-commit 1 s, gate-lib 25 s, build 23 s, test 193 s, hud 81 s, ci-check 86 s), then twice more under the load above: PASS 109 s, PASS 111 s.

### Cause

construct-host-hud: the cell closes question 51 with `done`, then emits question 41 (a detail of 6 KiB) and waited with `node_exists hud-card`. The host lays out the card of 6 KiB for seconds in a debug build under load (the probe on the node saw 3 s at load 20): old card 51 is gone at +0.4 s, the card of 41 ("Too long to show. Refused", no buttons) comes at +3 s. Two outcomes:

1. the old card 51 still stands when the cell asks, `detail_whole_or_refused` accepts it, the host then takes the done of 51, the card goes, `done 40` + `sleep 0.5` + `node_exists` finds nothing: "a done of another id closed the question" (the 73b line, cell over in 4 s). Probe trace (22:39): card 51 at 15.2, `host_detail` at 16.24 passes at 16.40 on the old card, card gone at 16.50, no card of 41 yet.
2. the old card is already gone and the card of 41 does not come within 5 s: "no card for the detail of 6 KiB".

Product check: `end_card` (plugins/chora-assistant/src/voice.rs:430) closes only when `card.id == id`; a done of 40 does nothing to the card of 41. Not a product fault.

chat-long-task: a follow-up of the same cell, not a timeout too short and not a lost event. The failed cell returns in the middle of the question: question 41 stays open (`run_cell` ended only 51), so the chat of chat-long-task opens with "A question is waiting." and its header never starts with `atlas · working`. Same line in the 73b log and in the 11 runs above.

### Fix (tests/gate-hud.sh, commit 9dde8523)

- host_detail waits (20 s, on the state) for a card whose text is not the card of 51 (`card_of_detail_question`).
- the done of another id is followed by a wait on the state (hud-state shows Thinking, then Question, only after the host took the done) in place of `sleep 0.5`; then a bounded wait for the card of 41, so one failed `choractl query` is no answer.
- run_cell ends 41 and 42 as well as 51 and waits for no card, so a failed cell leaves no question to the next cell.
- the wait for the long question of 42 is 20 s too.

The brief names `until_ok` of the gate lib: tests/gate-lib.sh has none (the only until_ok is local to tests/gate-chat-keys.sh); gate-hud.sh has its own bounded `waiting` on the state, used here.

### Open

- A voice.say during a question is not shown by the figure, so it is no barrier; the state change is.
- The 6 KiB card layout takes 3 s under load in a debug build: not looked at in a release build; no finding raised.
- Other `sleep 0.5` in host_long_words (before the rect checks) stay; 0 of 30 red after.
