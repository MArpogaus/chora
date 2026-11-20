# co-p2-fix (construct RC shared bar: no stray keys, key list by keys)

Lane: cloud, branch claude/co-p2-fix from 79c8d2b5; base branch already merged (up to date). Step 0: unshallow, cargo check green.

## r1
Head 7b43b063 (pushed). Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,keys,doors,construct-hud,cube,out,pull,matrix`: all 12 cells PASS (run dir /tmp/merge-gate.DxAlvN). A first run had ci-check red by my own doc line (cut-check: swallow.rs named chora-tiling; fixed 7b43b063, whole gate rerun green).

### co-25 (MAJOR): no unbound Super chord reaches the focused client in a face
- Red: cf27cfcb (gate-construct-keys, probe pane with `--report`; probe report `key 125 key 32 key 125 key 16 key 125 key 15 key 125 key 2 key 30`: d, q, Tab, 1 all arrived), c50beafa (gate-construct-cube, chora-probe on the front face of the cube with xp; q, Tab, 1 reached it).
- Fix: b061930f. New plugins/chora-construct/src/swallow.rs (the tiling key list; Super+KEY, Super+Shift+KEY, Super+Ctrl+KEY; nothing for modifier Alt). `regrab` grabs the bound modifier chords and then the unbound ones with `Grab::Swallow` (does nothing; input.rs arm placed before the `_ => fly_out` fallback) in a pane, and on a cube once the swallow is due. Bound chords keep their action in both places (Terminal, launcher, keys, chat, lock, power, screenshot, volume, close, and the cube key through `Grab::Own`).
- Design point: xp, the manager of the faces, grabs its own chords (Super+d, Super+e ...) as it connects, and a chord the construct held first would be lost to it (gate line "Super+d of the xp of the faces" went red when the construct swallowed at cube open). So on a cube the swallow starts 1 s after the manager connected (`Host::swallow_at`, deadline in `cube_deadline`, `check_cube` regrabs). Unit tests: swallow.rs (chord set, Alt) and tests_construct.rs (held after connect, not before).
- Tests green: gate-construct-keys "co-25: Super+d, q, Tab and 1 reach no window in a pane, a plain a does"; gate-construct-cube "co-25: on the front face Super+q, Tab, 1, Shift+x and Ctrl+z reach no window, a plain a does" and "stood back, Super+Tab reaches no window". Plain `a` still reaches the probe. Gate cells: keys, cube, doors, out, pull, matrix, construct-hud PASS.
- Docs: docs/configuration.md, construct section after the fly_out paragraph (beside the tiling paragraph, cross-referenced).

### co-26 (MAJOR, verify): Super+F1 in a face opens the key list
- Reproduced red on the cube face before the fix (build of c50beafa with the stray-key asserts relaxed): "FAIL: Super+F1 on the front face shows no key list of the construct (co-26)". Cause: not the harness. Outside a pane the construct held none of its modifier chords while a cube was open, so Super+F1 reached xp (which opens Help and Support, as the demo persona saw). In a pane the chord was grabbed already.
- Fix: same commit b061930f (bound chords are grabbed in a pane and on a cube). Green: gate-construct-keys "co-26: Super+F1 in a pane shows the key list" (the pane is left for the list, as before), gate-construct-cube "co-26: Super+F1 on the front face shows the key list of the construct, and a second one hides it".
- Behaviour to know: on a face the key list does not take the keyboard (the window keeps it), so Escape reaches the window; a second Super+F1 closes the list. In a pane the list steps out of the pane first, as before.

### For the ledger / PM
1. xp: a chord that nobody sees still opens the Start menu on the release of Super (xp's own rule). Swallowed Super+q on a face therefore opens the Start menu (pre-existing: unswallowed it did too). The gates close it with Escape. A lane for xp could treat a chord that another client grabbed as a cancel.
2. chora-tiling keeps its own copy of the swallow key list (not touched here, as ordered); swallow.rs repeats it. One shared list in chora-kit is a follow-up.
3. xp re-grabs its chords when a screen joins its outputs; with the construct holding unbound Super chords after the settle second, a chord xp lost then stays lost until the cube is rebuilt. Not seen in any cell; noted.
4. Tabs left open: co-26 text "only the Keys item of the top bar" is explained, not a harness issue.

## PM review r1 (2026-11-20 03:35 UTC)
Range 79c8d2b5..7b43b063 on claude/co-p2-fix (4 commits, +340 -14, 10 files). Red cf27cfcb, c50beafa before fix b061930f: order OK. PM is not the builder.
- co-25 in a pane: fixed. `swallow_unbound` grabs every unbound Super, Super+Shift, Super+Ctrl chord as `Grab::Swallow` after the construct's own grabs; gate-construct-keys red then green. OK.
- rv-1 MAJOR, cube face (co-25 face part, co-26): cube.rs `SWALLOW_AFTER` = 1 s after the face manager connected. The core's first grab wins, so (a) an xp that grabs later than 1 s under load (--jobs 3, slow start) loses its own Super+d to the swallow, silently; (b) any later xp regrab (plugins/chora-xp/src/wm.rs:410, settings reload or a new grab set) finds Super+d already held by the construct. A timer as ordering is a workaround (debt-2026-11-20 workarounds row) and fails POLS. Fix: no timer. The window manager that owns the window swallows for it: the construct in a pane, the face manager on a face (xp, its own lane later). The construct holds no swallow on a cube face.
- rv-2 MINOR (F2): swallow.rs KEYS is a second copy of the tiling list (the builder says so). Move the list and `chords()` into chora-kit (one `swallow` module, with a `Runtime` helper that grabs the unbound ones after a grab set); construct and tiling use it; a copies.toml row with max 0.
- rv-3 NIT: `chords()` returns nothing when the modifier is not Super; a user who sets modifier Alt gets no swallow and stray Alt chords. Doc line or swallow the configured modifier.
- Open, pre-existing: xp opens Start on a swallowed chord (to xp backlog).
Verdict (computed): rv-1 MAJOR open -> FIX. Round r2: briefs/co-p2-fix-r2.md.

## r2
Head ab4cbb4f on claude/co-p2-fix (pushed; 4 commits on 7b43b063; base already merged, up to date). Step 0: unshallow, cargo check green.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,keys,doors,construct-hud,cube,out,pull,matrix,tiling-look,gate-lib`: all 14 cells PASS (run dir /tmp/merge-gate.YYEbPn). A first run had ci-check red by my own code (tests/grab-gap-check.py: `grab_swallowed` had its first grab behind an `if`); fixed in ab4cbb4f, whole gate rerun green. gate-tiling-swallow.sh alone also green.

### rv-1 MAJOR: no timer, the construct swallows in a pane only
- Red: 837d9913. gate-construct-cube binds Super+k in xp 3 s after it connected (`ctl call xp settings.set` bindings) and presses it on a face; at 7b43b063 it failed: "Super+k, bound by the xp of the faces 3 s after it connected, did not reach it: visible True (rv-1)". (A late start alone does not show it, the old timer ran from the connect event; a late grab does.)
- Fix: 00469c80. SWALLOW_AFTER, `swallow_at`, `swallowing`, `swallows`, `swallow_deadline`, `swallow_due` of cube::Host and the swallow arm of `cube_deadline` and `check_cube` are gone; `regrab` swallows only `in_pane()`. Unit test of the timer deleted, `a_connected_manager_brings_the_faces_up` now asserts no deadline after connect.
- Green: gate-construct-cube "rv-1: a chord that the xp of the faces binds late is its own, the construct holds none on a face"; cell `cube` PASS.
- Face cases kept, marked `known: face manager swallow (xp lane)` (a `known()` helper in the gate; each case still asserts the good outcome when it holds and prints the known line when not):
  1. "co-25: on the front face a Super chord that nothing binds reaches the probe window" (Super+q, Tab, 1, Shift+x, Ctrl+z on the front face),
  2. "co-25: stood back, Super+Tab reaches the probe window".
  Both print `known:` in cube.log now. co-26 (Super+F1 on a face shows the key list of the construct) still passes as ok: the bound chords stay grabbed on a cube; no change.
- For the xp lane: xp must swallow the unbound Super chords of its faces itself (`Runtime::grab_swallowed` of the kit is the helper); then the two known cases turn back to ok.

### rv-2 MINOR (F2): one list in chora-kit
- Red: 9131008d (copies row with max 0 plus the kit module and its tests; `tools/cut-check` copies: "own list of swallowed keys (chora_kit::swallow): 1 left, the row allows 0" with the tiling list in place).
- Fix: 00469c80 (+ ab4cbb4f). New `crates/chora-kit/src/swallow.rs`: `KEYS`, `chords(modifier)`, `Runtime::grab_swallowed(modifier, &action)` (grabs the chords the plugin holds no action for; after the grab set, before `end_grabs`; the first grab wins). Construct (pane) and tiling use it; plugins/chora-construct/src/swallow.rs, tiling `SWALLOWED`, `SWALLOW_WITH`, `Config::swallowed` and their tests are deleted. tools/copies.toml row, max 0, allow crates/chora-kit/.
- Tests (kit, through FakeCore): super_chords_with_shift_and_ctrl_are_swallowed, another_modifier_keeps_the_keys_of_the_programs, a_bound_chord_keeps_its_action_and_the_rest_is_swallowed, a_chord_that_another_client_holds_stays_with_it, the_swallowed_chords_stay_in_the_set_of_the_old_grabs. Cells: ci-check, test, clippy, tiling-look PASS; gate-tiling-swallow PASS; keys PASS (pane).

### rv-3 NIT: modifier other than Super
Decision: doc line, not code. A user of `modifier: Alt` keeps `Alt+b`, `Alt+Left` in programs (same rule in tiling and the kit module doc); docs/configuration.md (construct section) now says so with the reason. Fix commit 00469c80. Test: kit another_modifier_keeps_the_keys_of_the_programs (nothing grabbed for Alt).

### For the PM
- One more doc change in the same paragraph: the construct no longer swallows on a face; the 1 s sentence is gone.
- `Host::connected` is still called on every client event of the manager (cube_client); harmless now it resets nothing.

## Review r2
Range 79c8d2b5..ab4cbb4f on claude/co-p2-fix (8 commits, 13 files, +396 -138). Reviewer: seeded cloud lane, read-only on code. Step 0: `cargo check -p chora-kit -p chora-construct -p chora-tiling` green. No merge gate run (the lane's 14 cells stand).
Checks:
1. rv-1 timer: `git grep -n SWALLOW_AFTER` empty; `regrab` swallows only `in_pane()` (camera.rs:476), the cube arm grabs bound chords only, so a chord the face manager binds late is not held by construct. `cargo test -p chora-construct` 241 ok. OK (see rw-2 for one edge).
2. Red first: 837d9913 and 9131008d precede 00469c80 (`git log --oneline 7b43b063..ab4cbb4f`). OK.
3. rv-2 one list: kit `swallow.rs` has KEYS, `chords()`, `Runtime::grab_swallowed`; tiling `SWALLOWED`, `SWALLOW_WITH`, `Config::swallowed` and tests gone; tiling and construct call the kit; copies.toml row max 0; `tools/cut-check` ok; kit tests (288, `--features testing`) ok. OK.
4. Tiling unchanged: old list (79c8d2b5 config.rs) and kit KEYS are identical, 75 items in the same order; chord order (none, Shift, Ctrl) the same; tiling `grab_all` grabs every bound binding, the drag and the give keys first, then `grab_swallowed` (first), and `grab_swallowed` skips any chord with an action. OK.
5. rv-3 doc: docs/configuration.md construct paragraph (Alt keeps readline and browser keys) matches `chords()` (nothing unless "Super") and the kit test `another_modifier_keeps_the_keys_of_the_programs`. OK.
6. Grab loop: `grab_swallowed` is a plain loop; `tests/grab-gap-check.py` rc 0; `Runtime::grab` keeps a chord of the old set (no free moment), a chord that another client holds stays with it (kit tests). OK.

Findings:
- rw-1 NIT plugins/chora-construct/src/cube.rs:156: the doc comment "When the construct starts to hold the chords of `Super` ... would be lost to it" (4 lines) is left over from the removed `swallow_at` and now sits on top of the doc of `told`, which it misdescribes. Fix: delete those four lines.
- rw-2 MINOR (plausible, not reproduced) plugins/chora-construct/src/camera.rs:476: `in_pane()` can hold with a cube open (modes.rs:322 steps back from a flown pane with `cube` present; input.rs:217 guards both), and then `grab_swallowed` holds every unbound Super chord, so a chord the face manager binds later (the rv-1 case) is lost until the camera leaves the pane. Fix: swallow only when `self.cube.is_none()`, or show by a cell that a pane and a cube cannot coexist and say so in a comment.
- rw-3 NIT plugins/chora-tiling/src/panel.rs (test `panel::the_layout_word_is_a_word_of_a_person`): red once in the first full `cargo test -p chora-tiling` here (175 of 176), green alone, green on a full rerun, and green at the base 79c8d2b5; the diff does not touch panel or its words. Named, not caused. Fix: find the order or timing dependency of that test.
Counts: BLOCK 0, MAJOR 0, MINOR 1, NIT 2.
Verdict (computed): no open BLOCK or MAJOR -> MERGE-OK.
