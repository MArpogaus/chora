# flakes-r (F5: no flaky tests)

## r1

Lane: cloud, branch `claude/flakes-r` from `a5b77488`; head `789aab53` (pushed). Written 11-21.
Step 0: `git fetch --unshallow`; the merge of `fb85b156` (claude/flakes-q) conflicted in `lifecycle.rs` because head
already carries the same list-fade fix (`06ea05e4`, co-hud-load): head's `lifecycle.rs` and `overlay.rs` were kept
(merge `e1ed73d1`), the gate script diagnostics of flakes-q came in. `cargo check` and `cargo build --workspace` green.
Load: 2 busy loops, 3 to 4 streams of the cell at once (`CHORA_GATE_LOCK_HELD=1`, else the gate scripts take the
lock and run one after the other: a first series of 30 doors runs was serial and so no real load), and for the
matrix and xp-faces a `cargo build -p chora-kit` loop beside; load average 6 to 9 on 4 cores. Logs are under
`/tmp/claude-0/` of the container only.

| Row | Result | Commits | Runs |
|---|---|---|---|
| FLK-DOORS-CO | cause shown (product), fixed | red `a9c48e02`, fix `4a1466f5` | 30 serial green; loaded: red 3 of about 86; after the fix 24 of 24 green |
| FLK-MATRIX-S1 | cause shown (test race), fixed | `789aab53` | before 19 of 24 red; after 24 of 24 green |
| FLK-XP-FACES | not reproduced | none | 30 of 30 green loaded |

### FLK-DOORS-CO
Red: 3 of about 86 loaded runs of `tests/doors/construct.sh`, always one `*-key` cell after a flight out of a pane
(`launcher-key` twice, with the same line as in the t96 log, `lock-key` once). A diagnostic in the cell (the focus,
the marks, the core log) showed that the key did reach the construct, as an `InputKey` of `Super+space` in the
walk (no pane, no cube), and that `rt.holds("Super+space")` and `rt.action(..)` were false. Cause: `regrab`
(`camera.rs:512`) holds the chords of the construct only while a pane window has the keyboard, so the step out of
the pane (`unsnap`, `camera.rs:603`) lets them go; the core can still have the press on its way, and the
`InputKey` arm of `input` (`input.rs`, the `rt.action(&binding)` match) fell to `_ => self.fly_out()`, which does
nothing in the walk. The key was lost; `marked '^fuzzel'` waited its 10 s and the cell was `missing`. Not caused
by the t96 diff.
Red `a9c48e02`: `a_chord_that_a_regrab_let_go_acts_as_a_key_of_the_walk` (lifecycle.rs) sends the `InputKey` of
`Super+space` to a construct in the walk and asks that the launcher starts: red 3 of 3.
Fix `4a1466f5`: `let_go_grab` finds the grab of the chord in `grab_keys` when the plugin holds none and it is in
the walk (no pane, no cube), and the arm runs it as the held grab would. No sleep, no retry. `cargo test -p
chora-construct` 262 of 262. Loaded after: 24 of 24 (4 streams of 6).

### FLK-MATRIX-S1
Full run: 1 of 3 red in flakes-p. `ONLY='construct [a-z]+ drag|mc construct (drag|leak|panes)'` at three streams
reproduces it in 12 of 30 (first batch) and 19 of 24 (second); a diagnostic that added two `ctl` calls before the
inject made it vanish (the race is below 0.3 s), one on the failure path kept it. A debug line in `node_point`
(not kept) gave the cause: the node `s1` is in the draw list, `shows_point` is true, `lands` fails in
`takes_pointer_at` (`inject.rs:300`): the node under the pointer is `construct-start-glass`. The start card of the
construct fades out when the first pane exists (`overlay.rs:778` `draw_start`, `start::hide`, a fade over
FADE_OUT) and its glass takes the pointer until the fade ends; `place` ends in `rects_stable`
(`gate-handover-matrix.sh:278`), which only compares the rects of the window, so the first `super_drag` (`:330`)
can come inside the fade; a moment later the same inject works. Test race: the injector is right to refuse.
`789aab53`: `super_drag` from the construct waits (`card_gone`, 50 x 0.2 s at most, on the state) until
`construct-start` is not visible. Loaded: 24 of 24 green. No red test of its own: a gate script race cannot be
forced in a unit test; the red series is the 19 of 24. Left to the PM: a fading card that still takes the pointer
for FADE_OUT is a small product question; not changed.

### FLK-XP-FACES
30 of 30 green loaded (3 streams of 10, 2 busy loops, the build loop beside), plus 2 of 2 in the gate. With the 45
of flakes-q that is 75 loaded runs not red. The row stays open on evidence; `fb85b156` logs the state at the next
sighting. No code.

## Gate
`tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,xp-faces,matrix,doors,acts` at
`789aab53`, twice, both all PASS (run 1: pre-commit 4 s, ci-check 40 s, build 26 s, test 207 s, clippy 14 s,
xp-faces 15 s, matrix 293 s, doors 370 s, acts 279 s; run 2: 4, 21, 0, 159, 1, 15, 302, 374, 276 s). gate-lib left
out as briefed.
