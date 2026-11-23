# mc-host

## r1

Branch claude/mc-host from 1ec04694, head 8dd16c0f (red fa828e4d, fix 8dd16c0f). Row: mc-58 (MAJOR, F5, rc 3 flake): tests/gate-mc-host.sh "the field gets no keyboard after the click".

**Cause: product, not harness.** `plugins/chora-assistant/src/chat/host.rs` `announce_keyboard` (was line 554) told `assistant.chat.keyboard` only when `chat.keyboard()` differed from what was told last, and the host reads the field only every 100 ms (`FIELD_POLL`, `chat_step` reads `field_events`). A click on Yes lands on a node of the host, so the core drops the keyboard from the field (`click_focus` / `set_layer_focus(None)`, crates/chora/src/input.rs:1120) and the field sends `on: false`; the host then answers the press (`press_card`, then `chat_refocus`, voice.rs / host.rs:443) and the field takes the keyboard back with the token of the press, `on: true`. When the host reads the field after both, `plugins/chora-assistant/src/chat.rs` `field_events` gets `[Keyboard(false), Keyboard(true)]` in one call and keeps only the last value (`self.keyboard = on`); the told state (true) equals the state seen (true), so nothing is emitted. The gate's `wait_keyboard` counts `on: true` events after the click and waits for a count that never comes. Which side wins (the press read before the field event or after) depends on scheduling: a race, hence the flake. What the run lost is the announcement of the event; I did not observe the keyboard itself failing. Any consumer of the event (chora-mc subscribes, plugins/chora-mc/src/main.rs:609) had the same hole.

**Fix (product).** `Session::set_keyboard` (chat.rs) notes every change in `edges`; `take_keyboard_edges` hands them over; `announce_keyboard` emits each one in order (a change equal to the told state is still skipped). docs/configuration.md says each change is told, also a loss and a return between two looks. No timeout raised, no known-red entry.

| step | result |
|---|---|
| red commit fa828e4d | unit `a_loss_and_a_return_of_the_keyboard_between_two_looks_are_both_told` fails by assertion (`[true]` vs `[false, true]`); the accessor still gave the state of the look, the host used it |
| fix commit 8dd16c0f | unit green, `cargo test -p chora-assistant` 245 passed |
| reproduce, before (1ec04694, `tests/gate-mc-host.sh`, one theme per run, 20 runs each) | light 2 red of 20 (runs 8, 11), dark 1 red of 20 (run 13), all "the field gets no keyboard after the click" |
| after, light and dark in one run, 20 runs | 0 red of 20 |
| after, 10 runs with `cargo build --workspace` (4 jobs, own target dir) running the whole time | 0 red of 10 |
| full gate `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` on 8dd16c0f, no --only | exit 0, 63 PASS, no red, no skip; mc-host and keys PASS |

Notes: the first loaded and unloaded series of this run passed `both` as the theme by mistake (one bogus theme, not light and dark); both were thrown away and redone with the two real themes. The before counts are per theme (20 runs light, 20 runs dark); 3 red of 40 runs.

Logs in the lane container only (scratchpad); the gate run directory was /tmp/merge-gate.vcK6k0.

## PM review r1 (2026-11-23 21:05 UTC)
Base 1ec04694..8dd16c0f, 3 files +56 -11 (PM review: under 200 lines; PM not the builder).
- Red first: fa828e4d test `a_loss_and_a_return_of_the_keyboard_between_two_looks_are_both_told` before fix 8dd16c0f: yes.
- Cause matches mc-58: `announce_keyboard` compared one state per 100 ms read; a false/true pair in one `field_events` call was lost. Fix queues each edge in `Session::set_keyboard`; every write of `keyboard` goes through it (`git grep "self.keyboard ="`: one hit, inside set_keyboard).
- Host loop emits each edge through `keyboard_change`, so a repeat is still dropped; an emit error returns early and drops the rest of that batch (no finding: the connection is gone then).
- Docs: configuration.md line updated.
- Gate: lane ran the full gate, exit 0, 63 PASS; cell mc-host 20/20 light+dark, 10/10 under load.
Verdict: 0 MAJOR, MERGE-OK 8dd16c0f; train120.
