## r1
Lane claude/co-i3 at 892d035c. Result: **not a regression of t101**; the breach is older and fires whenever the xp face draws. Red step pushed (afd63654).

**Repro** (persona harness, binaries snapshot, `ROUND_DIR` any): `round.sh snapshot; round.sh start NAME construct 1280x800` (PERSONA_CHECK=1), `act.py look` x2, `act.py click 330 292` (the start card row "A cube of desktops" = cube.open), wait 6 s, `act.py click 400 250` (capture pixels, the xp desktop, no window open). `breaches.jsonl`: `{"action": "click", "look": "construct", "invariant": "I3", "breach": "construct and xp each say they hold the focus"}` (same text on 892d035c and a84effda; older commits say `the core focuses xp-icons of xp, construct says it holds the focus`, same cause). The click is the first one in the face; I6 rows of the icons come with the cube opening. Script: bounded, build + run + exit 1 on I3, 125 if the face did not show.

**Bisect.** First-parent merges a84effda..892d035c: a84effda itself is bad (run twice), so the range holds no first bad merge (the 892d035c..a84effda `git bisect run` ended on test-only a2ee5219 because a84effda was assumed good; ignore it). Going older: bad at 1499cfdb and 8c26ce7a with the snapshot binaries. Between b843238f..d1a287d7 the snapshot run is "good" only because xp never drew its face: before e330fba1 (fix(persona): give the session the xp theme of the source tree, merged in d1a287d7) a snapshot of the binaries had no xp theme, so xp did not connect ("XP did not start", round 4 had 0 for that reason, all three construct sessions) and the face was one colour (I7). Run in place (`bin` -> target/debug) the I3 fires at b843238f and 88258c87 too. So the first bad merge in the range is none; first bad for the persona run = d1a287d7 (xp starts from a snapshot), the breach is in the construct focus code since the xp face exists.

**Cause** `plugins/chora-construct/src/state.rs:66`: `None if self.active && !self.overlay => json!({"holder": NAME})`. `core_focus` follows `focus.changed` (`plugin.rs:222`), which names surfaces only. A click on the xp desktop focuses the node `xp-icons` of xp (`focus.node`), so `core_focus` is None and the construct, active, says it holds the focus, while xp's `state.get` says holder xp (`crates/chora-kit/src/wm/focus.rs:241`, Holder::Node). The construct does not track `focus.node` of another client (`Focus::taken_by_other`, focus.rs:215).

**Red step** `tests/gate-construct-i3.sh` (cell `construct-extra` of tests/merge-gate.sh): headless session, cube.open, one injected click at (640, 300) in the front face, then `state.get` of construct and xp. On 892d035c: `FAIL: construct and xp each say they hold the focus after a click on the desktop of the xp face (construct: construct, xp: xp)`. Commit afd63654 `test(construct): I3 one focus holder on a click in the xp face (red)` (the hook caps a subject at 68 characters, so "single" went out of the subject).

**Fix proposal** (one line): in `focus_value` the construct says holder null while a node of another client has the focus (feed the chora_kit `Focus` with `focus.node` events and ask `taken_by_other`), so that `None if active && !overlay` needs no foreign node; the keys-route lane that owns choose_manager_of should take it together with its focus tracking.

## r2

Head `2b571478` on claude/co-i3 (merged with origin/claude/autonomous-dev-session-3w-i33pg7: already up to date).

- Red unit test `9903d353` `state::tests::a_focused_node_of_another_client_takes_the_focus_off_the_construct` (failed: holder construct, wanted null).
- Fix `66cccb18` (`state.rs` focus_value: `None if self.active && !self.overlay && !self.kit_focus.taken_by_other(NAME)`; `kit_focus` is a chora_kit `Focus` fed in `plugin.rs on_event` through `hear`) and `2b571478` (the subscription lacked `focus.node`: the first full gate still showed the i3 FAIL; `EVENTS` in main.rs now lists it). No reply changes, docs/api.md untouched.
- `cargo test -p chora-construct`: 266 passed. Full `tests/merge-gate.sh --jobs 1`: all steps passed, construct-extra PASS (includes gate-construct-i3.sh).

## Review r2 (PM, not the builder; afd63654..2b571478, 3 files +28 -3)

Checked: red first (9903d353 before 66cccb18); the kit is reused (`Focus::hear`, `taken_by_other`), no copy; `focus.node` is subscribed (main.rs EVENTS); the construct's own node does not count (owner == NAME); no core or xp code; the lane's gate green with construct-extra (gate-construct-i3.sh) PASS.

- rv-1 MAJOR, plugins/chora-construct/src/state.rs:65: the foreign-node check guards only the `None` arm. With a construct pane window focused (`core_focus` = an owned surface) a click on an xp node sends `focus.node` (owner xp) and no `focus.changed` (focus.rs: "the focused surface stays, but no window has the keys meanwhile"), so construct still answers `{"holder": "construct", "surface": ...}` while xp holds the node: I3 two holders again. Fix: answer holder null whenever `kit_focus.taken_by_other(NAME)`, before the match (as `Focus::holder` does, focus.rs:205), with a unit test: an owned pane surface focused, then a foreign `focus.node`, then holder null; and a gate step (a window in a pane, then a click in the xp face).
- rv-2 MINOR, crates/chora-kit/src/wm/focus.rs:120: `moved` does not clear `away`, so a foreign node that is followed by a window focus and then by an empty focus (that window exits) leaves the construct with no holder until a `focus.node` null comes. Pre-existing in the kit; check whether the core sends `focus.node` null when a window takes the keys and pin it in a kit test either way.

Verdict: FIX (rv-1 MAJOR). Next: r3 on 2b571478.
