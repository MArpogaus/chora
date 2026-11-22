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

## r1 at 6b01c976

The brief was re-seeded on the head 6b01c976 (61 first-parent merges); this section reviews the whole delta adf71956..6b01c976 again and supersedes the section above (taken at 1c599c74).
Lane: cloud, read-only on code, claude/review-delta at 6b01c976, base adf71956 (crates, plugins, services: 87 files, +6243 -1394; 61 first-parent merges). Reviewer is not a builder of the delta (rules.md 7).
Step 0: `git fetch --unshallow`, `cargo check --workspace` green (1m43s).

Commands (all green):
- `cargo test -p chora input:: ipc::` is still rejected by cargo (one filter only); ran as `cargo test -p chora input::` 60 passed, `cargo test -p chora ipc::` 152 passed.
- `cargo test -p chora-kit --features testing` (321 unit + integration, 0 failed), `cargo test -p chora-xp` 328 passed, `cargo test -p chora-construct` 267 passed, `cargo test -p chora-mc` 189 passed.
- `tools/lint-ratchet`: ok (allow 360, no-reason 0, eprintln 131), exit 0.

### Findings

Core (input.rs, ipc.rs, testing.rs)
- rd-1, MINOR, crates/chora/src/input.rs:207 (also :934): `take_grab` and `send_button` call `end_lone_taps_of_others` for injected keys and buttons too, so an `input.inject` client can cancel the Super tap that the person holds (no Start menu at its release); fix: end the taps only when `!injected || state.injected_press_is_real`.

Kit (launcher.rs, wm/focus.rs)
- rd-2, MINOR, crates/chora-kit/src/launcher.rs:172: the exit status is ignored, so a fuzzel that dies by a signal (crash, OOM kill) or exits with a code other than 0/1 without an "err:" line counts as a close and nothing is told; fix: return `Asked::Failed` for a death by a signal other than SIGINT/SIGTERM or an exit code outside 0/1.
- rd-3, MINOR, crates/chora-kit/src/launcher.rs:152: stderr is now piped and dropped unless it holds an "err:" line, so fuzzel's warnings (ignored or deprecated config keys) no longer reach the session log; fix: pass the other stderr lines on with `eprintln!`.

xp
- rd-4, MINOR, plugins/chora-xp/src/help.rs:341: a wheel step under half a row rounds to 0 and then scrolls a whole line by `signum`, so a touchpad scrolls the key table a line per tiny event (carried over from the section above; the dy 0.0 part is fixed); fix: keep the sub-row remainder and scroll only whole rows.

mc
- rd-5, MEDIUM, plugins/chora-mc/src/main.rs:603: mc does not subscribe to `focus.node`, so `Focus::away` is never set and `state.get` names mc and its window the holder while a node of another client (xp Start menu, the construct's walk) has the keys: two holders (I3); `refresh` (:657) checks only mc's own node, and the doc of `Plugin::refresh` (crates/chora-kit/src/runtime.rs:106) still says the core tells no look; fix: add "focus.node" to EVENTS (`focus.hear` already handles it) and correct that doc.
- rd-6, MINOR, plugins/chora-mc/src/main.rs:1113: 2696 lines holding four concerns: launch and args (431-512), the RPC param structs (152-283), the Plugin impl with id and command helpers (647-1112), and 1583 lines of inline tests (1113-2696); cut (named, not done): tests to src/tests.rs as construct did, the param structs to src/methods.rs next to `region_method`, the id helpers to src/scene.rs.

construct, logging: no findings.

### Checked, clean
- Core: the safety layer still comes first (`pressed_key`, unchanged), before grabs and node keys. Raw keys follow the keyboard focus only; the pointer picks the place of a grab (D55), and the grabs of the focus owner are never held back (place.rs:33). The button-grab owner walks the ancestors of the node under the pointer, else the smallest claim (co-30). No new IPC request or param struct in the delta (`focus.node` is an event; scene/command.rs unchanged), so no new validation surface. testing.rs is `#[cfg(test)]` (lib.rs:38).
- Kit: fuzzel is spawned with `Command::new` and args, no shell; a missing launcher is told once as a notice and on stderr; the launcher runs in its own process (`--launch`/`--run-box`), not on a plugin's event loop. Typed text that names an installed program runs via `sh -c` as before (the Run box, by design, the person's own input). `Focus::holder(name, ..)` makes only another client's node Foreign; `lose` keeps `core`, `forget` clears it.
- xp: Send To hands through `give_dragged`, which refuses a give to xp and rolls the book back on a refusal (`book.refused`) with a notice: no lost window. The Volume slider acts only on its track (the MEDIUM of the section above is fixed). `clients.list` runs at start, at client events and at a menu open with no look known, not per press.
- construct: an overlay shows the window of its own program without claiming it (D66), so tiling keeps it; `unseen_focus_to_walk` takes the keys off a pane that a look or a glide turned away from and regrabs, so no stray keys after a camera move; lone Super taps are swallowed while a pane has the keys; F1 without a modifier is grabbed only with `input.grab-any` (documented in construct.yaml).
- mc: `scene.show` and a click both go through `show`; the voice HUD question now fits in lines as well as bytes.
- Logging (rc 9a): the new eprintln! lines carry program names, RPC errors, look names and install paths; no typed text, no home paths; no new tracing line at info or above.

Known, not counted: FLK-DOORS-CHAT, co-41/co-42, mc-47.

Verdict (computed): MERGE-OK. BLOCK 0, MAJOR 0, MEDIUM 1, MINOR 5.
