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
