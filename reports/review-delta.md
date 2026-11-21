# review-delta

## r1

Lane: cloud, read-only on code, claude/review-delta at 1c599c74, base adf71956 (crates, plugins, services: 65 files, +3672 -535; 34 first-parent merges). Reviewer is not a builder of the delta (rules.md 7).
Step 0: `git fetch --unshallow` (already full), `cargo check --workspace` green (1m53s).

Commands (all green):
- `cargo test -p chora input:: ipc::` is rejected by cargo ("unexpected argument 'ipc::'": one filter only); ran as `cargo test -p chora input::` 57 passed, `cargo test -p chora ipc::` 150 passed.
- `cargo test -p chora-kit --features testing`, `cargo test -p chora-xp`, `cargo test -p chora-construct`, `cargo test -p chora-mc`: 1174 passed, 0 failed.
- `tools/lint-ratchet`: ok (allow 360, no-reason 0, eprintln 131), exit 0.

### Findings

Core (input.rs, ipc.rs, testing.rs)
- rd-1, MINOR, crates/chora/src/input.rs:177 (also :922): `end_lone_taps_of_others` runs for an injected grabbed key or button too, so an `input.inject` client can cancel the person's held Super tap (no Start menu at the release); fix: call it only when the press is not injected.

xp
- rd-2, MEDIUM, plugins/chora-xp/src/context.rs:943: a release anywhere on the Volume slider row sets the level from the clamped place, so a click on the word "Volume" sets 0 % and one in the right padding 100 %; fix: act only when the pointer is over the track (left - TRACK_END .. left + TRACK + TRACK_END), else no act.
- rd-3, MINOR, plugins/chora-xp/src/help.rs:341: `dy.signum()` is 1.0 for dy 0.0, so a horizontal-only wheel event scrolls the key table a line down, and each small touchpad step (under half a row) scrolls a full line; fix: return false for dy 0.0 and accumulate sub-row deltas (or use `discrete`).

mc
- rd-4, MINOR, plugins/chora-mc/src/main.rs:1431: 3018 lines holding four concerns: launch and args (103-510), the RPC param structs and region methods (149-270, 871-1193), the Plugin impl, and 1544 lines of inline tests (1431-2975); cut (named, not done): tests to src/tests.rs as construct did, param structs and `region_method` to src/methods.rs.

Kit, construct, logging: no findings.

### Checked, clean
- Core: the safety layer still comes first in `pressed_key` (press_action before grabs and node keys); keys never follow hover; the new button-grab owner walks the ancestors of the node under the pointer (`Pointer`), falling back to the smallest claim (D55, co-30). No new IPC request in the delta; `focus.node` is an event only, and `focus_node {}` ends only the caller's node (command.rs:95). testing.rs is `#[cfg(test)]` (lib.rs:38); the global test subscriber only formats for threads with a sink.
- Kit: launcher.rs spawns fuzzel with `Command::new` and args, no shell; a missing or failing fuzzel is reported once ("Run could not start: ...") as a notice and on stderr, a close stays silent (6 tests). Typed text that names an installed program runs as before (the Run box, unchanged). `Focus::holder(name, ..)` counts only another client's node as Foreign; `lose` keeps `core`, `forget` clears it.
- xp: Send To refreshes the look list and a refused give rolls the book back (`book.refused`) and shows a notice: no lost window. The new `xp-desktop` root is removed on restart by `remove_scene` (top-level owned node). `clients.list` and `scene.query` are short core calls, as elsewhere; no new blocking call on another client. No new risky act without a dialog.
- construct: windows of an overlay's own program are shown but not claimed (D66, tests); help and the walk take and give back the keyboard only to a window in sight; `unseen_focus_to_walk` takes the keys off a pane that a look turned away from, so no stray keys after a camera move; lone Super taps are swallowed while a pane has the keys.
- mc: window and voice paths share `focus.holder(NAME, ..)` and `sync_keys`; no divergence found in the delta.
- Logging (rc 9a): new eprintln! lines carry config program names, RPC errors and look names, no typed text and no user paths; no new tracing line at info or above.

Known, not counted: FLK-DOORS-CHAT, co-41/co-42, mc-47.

Verdict (computed): MERGE-OK. BLOCK 0, MAJOR 0, MEDIUM 1, MINOR 3.
