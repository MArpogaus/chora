# mc-host

## r1

Branch claude/mc-host from 1ec04694, head 8dd16c0f (red fa828e4d, fix 8dd16c0f). Row: mc-58 (MAJOR, F5, rc 3 flake): tests/gate-mc-host.sh "the field gets no keyboard after the click".

**Cause: product, not harness.** `plugins/chora-assistant/src/chat/host.rs` `announce_keyboard` (was line 554) told `assistant.chat.keyboard` only when `chat.keyboard()` differed from what was told last, and the host reads the field only every 100 ms (`FIELD_POLL`, `chat_step` reads `field_events`). A click on Yes lands on a node of the host, so the core drops the keyboard from the field (`click_focus` / `set_layer_focus(None)`, crates/chora/src/input.rs:1120) and the field sends `on: false`; the host then answers the press (`press_card`, then `chat_refocus`, voice.rs / host.rs:443) and the field takes the keyboard back with the token of the press, `on: true`. When the host reads the field after both, `plugins/chora-assistant/src/chat.rs` `field_events` gets `[Keyboard(false), Keyboard(true)]` in one call and keeps only the last value (`self.keyboard = on`); the told state (true) equals the state seen (true), so nothing is emitted. The gate's `wait_keyboard` counts `on: true` events after the click and waits for a count that never comes. Which side wins (the press read before the field event or after) depends on scheduling: a race, hence the flake, and the same race in mc-57's neighbourhood only under load. The keyboard of the field was right all the time (Esc closed the chat in the cases that passed); what was lost is the announcement. Any consumer of the event (chora-mc subscribes, plugins/chora-mc/src/main.rs:609) had the same hole.

**Fix (product).** `Session::set_keyboard` (chat.rs) notes every change in `edges`; `take_keyboard_edges` hands them over; `announce_keyboard` emits each one in order (a change equal to the told state is still skipped). docs/configuration.md says each change is told, also a loss and a return between two looks. No timeout raised, no known-red entry.

| step | result |
|---|---|
| red commit fa828e4d | unit `a_loss_and_a_return_of_the_keyboard_between_two_looks_are_both_told` fails by assertion (`[true]` vs `[false, true]`); the accessor still gave the state of the look, the host used it |
| fix commit 8dd16c0f | unit green, `cargo test -p chora-assistant` 245 passed |
| reproduce, before (1ec04694, `tests/gate-mc-host.sh`, one theme per run, 20 runs each) | light 2 red of 20 (runs 8, 11), dark 1 red of 20 (run 13), all "the field gets no keyboard after the click" |
| after, light and dark in one run, 20 runs | 0 red of 20 |
| after, 10 runs with `cargo build --workspace` (4 jobs, own target dir) running the whole time | 0 red of 10 |
| full gate `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` on 8dd16c0f, no --only | exit 0, 63 PASS, no red, no skip; mc-host and keys PASS |

Notes: the first loaded and unloaded series of this run passed `both` as the theme by mistake (one bogus theme, not light and dark); both were thrown away and redone with the two real themes. The before counts are per theme (20 + 20 runs, one baseline run of 40 in all); the red rate is about 7 % of the runs of one theme.

Logs in the lane container only (scratchpad); the gate run directory was /tmp/merge-gate.vcK6k0.
