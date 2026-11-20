# xp-p2r3 (RC 2 persona, bar "no stray keys into clients")

## r1
Head f89f785c on claude/xp-p2r3 (pushed; from ab4cbb4f, xp-fix a767790d merged as ad8b955c; origin/claude/autonomous-dev-session-3w-i33pg7 already merged, up to date). Step 0: unshallow, cargo check and build green.
Gate: `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,cube,keys,xp,xp-faces,acts,doors,tiling-look`: all 12 cells PASS (run dir /tmp/merge-gate.Pvpju0). No red cell.

| item | red | fix | cell |
|---|---|---|---|
| 1 face swallow (co-25 face part) | 06c900b6 (gate-construct-cube: `known()` removed, the three face cases are plain asserts, red on the head with "co-25: on the front face ..."; plus xp unit test) | b9f4f5f3 | cube, xp PASS; cube.log has no `known:` line |
| 2 XPF-R1 rv-1 Starting message | f41199a4 (kit unit test, two starts) | 5449aa27 | test PASS |
| 3 CO-P2-FIX rw-2 | 8357b193 (construct unit test, pane and cube) | 851e8960 | test PASS |
| 4 CO-P2-FIX rw-1 NIT | docs commit f2430ab4 | | |
| 5 XPF-R1 rv-2 NIT | no red (nit) f89f785c | | persona-selftest ok |

### 1 face swallow
`grab_keys` of xp (wm.rs) calls `Runtime::grab_swallowed("Super", Look::Swallow)` after its own grabs when the cube keys are Flat or Hosted (virtual outputs, no cube of its own), before `end_grabs` of its callers. New `Look::Swallow` (serde skip, no setting; does nothing). A chord xp binds (Start, key list, Super+d, Super+e) keeps its action; a screen of xp (Own) swallows nothing; the swallow leaves with the face on `change_keys` to Own. Tests: xp `on_a_face_xp_swallows_the_super_chords_it_binds_nothing_for`; `a_change_of_the_cube_keys_keeps_the_other_keys` now counts only the ungrabs and checks no bound key is free between. Gate: co-25 front face, co-26 (Super+F1 key list of the construct) and co-25 stood back are plain asserts; all green.
Note: the brief names two known cases; the gate had three `known` uses (co-26 as well); all three are plain asserts now.

### 2 Starting message
`Launches` counts the windows that carry no token (`bare`); an entry keeps `bare_before`; the message of a shown start ends on a bare window, or on the window of its own token (the entry leaves). A window with another start's token (known or not) leaves it. The exit-without-window and give-up logic still counts every window (a handover launcher is unchanged). Test is in launch.rs on `Launches` directly (the module's own pattern; no FakeCore in that module): starts t1 and t2 shown, t2's window answers its note, t1's message stays, t1's window answers its own.

### 3 construct
`regrab` swallows only `in_pane() && self.cube.is_none()`. Test: a flown pane swallows; with a cube open too, `Super+q` is not held.

### 4 and 5
cube.rs: the four stale doc lines on `told` deleted. persona session.sh: the mousepad stand-in appends `mousepad stand-in foot` or `probe` to calls.log (PERSONA_CALLS) per start.

### For the PM
- xp still opens Start on the release of Super after a swallowed chord (pre-existing, backlog line CO-P2-FIX r1); the gate closes it with Escape.
- Commit hooks here: subject at most 68 characters, body lines at most 72, no inline comments in code (one of mine was refused and removed).

## Review r1
Range ab4cbb4f..f89f785c on claude/xp-p2r3 (18 files, +570 -60; it carries the xp-fix merge). Reviewer: seeded cloud lane, read-only on code. Step 0: `cargo check --workspace` green. No full merge gate (the lane's 12 cells stand); I ran the `cube` cell alone on the head: rc 0, `ok:` for co-25 (front face), co-26, co-25 (stood back), rv-1; no `known:` line.
Checks:
1. Red first: OK. 06c900b6 `cargo test -p chora-xp on_a_face_xp_swallows` fails ("Flat: Super+q is not held"), green at b9f4f5f3. f41199a4 `the_window_of_another_start_leaves_the_message_of_a_start` fails ("t1 still loads", left Done, right none), green at 5449aa27. 8357b193 `a_pane_and_a_cube_hold_no_unbound_super_chord` fails ("with a cube open the face manager swallows, not the construct"), green at 851e8960. The cube cell: the three `known()` uses are `fail` now (gate-construct-cube.sh:194, 202, 228); `known()` is deleted.
2. co-26 on this head: it is not fixed by this branch. The assert passed on the base already (co-p2-fix r2 says "still passes as ok"; b061930f grabs the bound chords on a cube). Path: xp binds Super+F1 itself (System::Keys), so xp's `grab_swallowed` skips it; the construct holds it first (`regrab` -> bound chords, camera.rs:473-476), and the key list shows (`construct-help` opacity 1.0, gate help_shown). The co-25 face part is the one this branch fixes (red message names co-25 only). Only the `known()` escape of co-26 is gone, so a later regression would now fail.
3. grab_keys (wm.rs:806-825): OK. Swallow only on Flat or Hosted, after the bindings loop; `Runtime::grab_swallowed` skips any chord that has an action, so Super+d, Super+e, the key list and the Tap chord stay xp's; the bare Super is no KEYS chord, so it still reaches the Tap action. Both callers (`grab` at startup.rs:266, `change_keys` at wm.rs:830 via sync_cube_keys) pass the current `cube`, so a settings reload keeps the swallow and a change to Own drops it inside one start_grabs/end_grabs (test `a_change_of_the_cube_keys_keeps_the_other_keys`: Super+q gone, no bound key free in between). The xp-skipped cube chords (Super+Ctrl+Left stand-ins) become swallowed on a face, as wanted; the construct's cube keys are Ctrl+Alt+Left/Right and fly_out Alt+Escape, none in the Super set; the give keys are Super+Alt+N, also outside it.
4. `Look::Swallow`: no hidden setting. `#[serde(skip)]`, never put in `bindings` (only `rt` holds it), so it is never serialized or listed. Dead only as two exhaustive arms (help.rs:99, 125). A plain fn cannot do it: `Runtime<KeyAction>` needs an action value, and the construct has the same shape (`Grab::Swallow`). Accept (rv-4).
5. Kit `bare` counter: OK in the three cases. Two starts at once: each entry keeps `bare_before`; a window with t2's token does not move `bare`, t1's message stays (test). A start that exits without a window: `gone && silent && crashed` still counts every window; with none, it fails at GIVE_UP with its note. A window with a stranger token: counts in `windows`, not in `bare`, pushes into `opened`; the entry leaves at GIVE_UP at the latest (`retain_mut` returns false), no leak. One edge: a start that is not silent and not watched leaves at GIVE_UP with its note not ended; the notice expires itself (`expire_ms` = GIVE_UP, panels.rs:57), so it is bounded. A bare window before SHOW_AFTER ends the message at the first check after it shows (by design, a bare window may be the start's own).
6. construct `regrab` (camera.rs:476): OK. A flat pane swallows; with a cube open and the camera still in the pane (`in_pane()` true, test asserts it) `Super+q` is not held. See rv-3 for the one edge.
7. `cargo test -p chora-xp -p chora-kit -p chora-construct --features chora-kit/testing`: xp 307, kit 291, construct 242, all ok. `cargo clippy --workspace --all-targets -- -D warnings`: clean. Files over 1,500 lines: construct cube.rs (1817 total, 1821 before; it shrank by 4) is under the size-ratchet list, nothing grew; xp wm.rs 1489, kit launch.rs 1351.
8. Start after a swallowed chord: not worse, better on a face. xp now receives the chord (action Swallow), and `Wm::key` (keys.rs:165-168) marks the pending Tap as used, so the release of Super opens no Start. I checked by running a copy of gate-construct-cube.sh with a marker in both `if start_menu_open` branches: no marker, so the menu was closed after the chords on the front face and stood back. It is still open in a pane (the construct swallows, xp never sees the chord): that stays CO-P2-FIX r1.

Findings:
- rv-1 MINOR tests/gate-construct-cube.sh:185 and 222: `if start_menu_open; then plain Escape ...` tolerates a Start menu that a swallowed chord opens, so a return of the stray Start (a stray key into the user's path) passes. On this head it never opens on a face (check 8). Fix: `start_menu_open && fail "a swallowed chord opened the Start menu"` in both places.
- rv-2 NIT reports/xp-p2r3.md "For the PM" first bullet: "xp still opens Start on the release of Super after a swallowed chord" no longer holds on a face (check 8), only in a pane. Fix: say so, and narrow the backlog line CO-P2-FIX r1 to the pane.
- rv-3 MINOR (plausible, not reproduced) plugins/chora-construct/src/camera.rs:476: with a cube open and the camera in a pane, nobody swallows when the face manager is not connected (starting, failed or restarting face, `faces::a_failed_face...`); then unbound Super chords reach the pane window. Fix: swallow in the pane also while the cube host has no connected manager, or name the gap in the comment.
- rv-4 NIT plugins/chora-xp/src/config.rs:116 `Look::Swallow`: right shape (see check 4); two arms only for exhaustiveness (help.rs:99, 125). No change needed; leave as is.
- rv-5 NIT crates/chora-kit/src/launch.rs:972-1000: the two-start test uses a known token (t2); the brief's stranger token (a token no start has) has no assert. Fix: add `window(Some("nobody"))` to `the_window_of_another_start_leaves_the_message_of_a_start` and assert t1 still loads.
Counts: BLOCK 0, MAJOR 0, MINOR 2, NIT 3.
Verdict (computed): no open BLOCK or MAJOR -> MERGE-OK.
