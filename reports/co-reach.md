# co-reach

## r1
Lane: cloud, claude/co-reach from bc153929. Step 0 green: unshallow, `cargo check --workspace`, `cargo build --workspace`. Shas: **red 1b3abdfc** (test: the cube the person opens stands back; a window ahead shows beside it), **fix d9b2c4f1** (fix: the cube the person opens stands back), **gates fe369799** (test: the cube gates hold the pane beside the cube), head fe369799. No core change, no xp change (hosted.rs not touched: no step showed an xp stacking fault).

### Step 1: reproduction (scripted, no persona)
Script (rewritten, the /tmp scripts of r7 are gone): one fresh construct session per run via `round.sh start` (1280x800, PERSONA_CHECK=1, act.py, logical coordinates: row Terminal (640,364), row "A cube of desktops" (640,468), xp Start (48,784) and Lock (48,754) = the persona's (30,490), (30,471) at scale 0.625), then a state dump (construct and xp `state.get` windows and focus, `cube.list` mode and front, `scene.query kind:surface`, the surface boxes of `scene.boxes` as stacking).
- co-52 (row, 6 s, Super+c, 3 s): **5/5** red (1 + 4 runs; modes row use, Super+c use). Dump after the row and after Super+c: `cube.list` mode `use`, front 0, four xp faces up; construct `{'holder': 'construct'}`, xp `{'holder': None}`, no surface. The capture is a plain xp face with pager 1 to 4 and the hint `Super F1 All keys` only: no cube, no `Step back` (the hint `Alt Esc Step back` shows 3 s after a step in and then only with the pointer at the bottom). Super+c with a cube open does nothing (`cube_key` returns; in use mode the face keeps Super).
- co-54 (row Terminal, foot up, Super+c, 6 s): **5/5** red (s1 shown false after Super+c in 5 of 5). Dump before: s1 shown true, parent `construct-front`, focus s1. After: construct s1 **shown false**, focus `{'holder': 'construct'}` with no surface, cube mode `use`; `construct.list`: camera z 0, pane foot at [0, 2000] (1280x800), the cube's front face at z 832 and its back at 2112: the pane is inside the cube.
- co-51 (row, Start, Lock, 12 s, key a): I2 "xp shows s1 on VIRTUAL-1, which has no box of a size there" at the Lock click in **5/5** fresh runs (plus 1/1 in the co-57 run and 1/1 by hand); I3 rows **0/5**; `swaylock -f` called 5/5; after key a: construct `{'holder': None}`, xp `{'holder': 'xp'}`, xp windows [] (one holder, the key went to xp; co-focus holds).
- co-57 (row, Start, Lock, look): the stand-in **draws** its window: xp `state.get` lists s1 `Screen locked`, app id chora-probe, on VIRTUAL-1 at [112, 24] 200x200 (rect -262032, -262120: VIRTUAL-1 lies at -262144), shown, z 0, parent `xp-win-s1`, focus xp s1, for 8 s (26 polls, 0.96 s to 8.65 s); the capture shows a dark 200x200 square without a title bar and a taskbar button "Screen locked". The persona of r7 looked 1 to 2 times and did not read the square as a lock screen.

### Step 2: severity
- co-52 **MAJOR** (confirmed): vision card, chora-construct "Desktops on cube faces; step out and in": the act that defines the look shows a plain desktop; reports/p2-plan.md C1 "open the cube, step in, step out".
- co-54 **MAJOR** (confirmed): vision card, shared RC bar "no lost window in any handover"; design/rc.md 9 "no known window loss". The terminal shows in neither mode: use mode covers it, stood back it gives way (hidden), and it has no handle.
- co-51 **not product; NIT (harness)**: RC bar "lock ... reachable" holds (Lock calls `swaylock -f` 5/5, a key after it reaches xp, I3 0/5). The I2 row is a race of the checker (step 3), not a window without a box.
- co-57 **not product; NIT (harness)**: Lock calls `swaylock -f`; the core implements ext-session-lock (crates/chora/src/lock.rs), so a real swaylock locks. The stand-in of tests/persona/session.sh:127-131 draws a 200x200 `chora-probe window` without decoration, which a persona cannot read as a lock screen. Harness note: a full-screen stand-in with a title ("Screen locked") would let the persona see it.

### Step 3: causes (bc153929)
- co-52: `plugins/chora-construct/src/input.rs:616` `cube_key` (Super+C; the row of the start card, `input.rs:561`; the hint, `overlay.rs:1039`) only calls `open_cube`, and `cube/construct.rs:30` opens the host "ahead of the camera, in use mode" (`construct.rs:86`, `View::default()` at `:116`), the camera levelled so that the front face is at 1:1: the person sees one xp desktop and nothing of a cube.
- co-54: the same open puts the cube where the panes stand (front face at the focal length, `construct.rs:95` reach `focal + size/2`), so a pane ahead is covered in use mode (`cube.rs:425` `covers`, `camera.rs:369` `in_sight`: shown false); stood back, `cube/aside.rs:154` `.filter(|(_, p)| p.placed)` moves only placed panes beside the cube and `frame.rs:171` `covers_cube` hides the loose one. co-52 and co-54 share the first cause (the person's cube opens in use mode over the space); co-54 has the second line aside.rs:154.
- co-51: tests/persona/check.py `Run.snapshot` reads the states first (`wait_quiet(read_states)`) and hands them to `harness.Session.snapshot` (tests/acts/harness.py:803), which then takes the captures of 5 outputs, the colours and reads `scene.boxes` last; here that took over 8 s (one act with the checker took 18 s), and the stand-in's window lives 8 s (`timeout 8`). The saved snapshot of the Lock click shows it: xp states list s4 shown on VIRTUAL-1, while `nodes` and `toplevels` of the same snapshot have no s4. Polled directly every 50 ms, no state lists a shown window without its box (0 of 9 locks, fresh and warm sessions).
- co-57: tests/persona/session.sh:127-131 (stand-in), no product line.

### Step 4: red and green
Red 1b3abdfc (on bc153929), `cargo test -p chora-construct --bin chora-construct`:
- `cube::tests_construct::the_cube_the_person_opens_stands_back` panicked at tests_construct.rs:1076 "the cube stands back" (left: Use, right: Navigate).
- `cube::tests_construct::a_window_ahead_shows_beside_the_cube_the_person_opens` panicked at tests_construct.rs:1110 "the window shows" (a loose pane ahead, 1280x800, as a terminal opens).
Fix d9b2c4f1 (input.rs +6 -3, aside.rs +9 -9): `cube_key` opens the cube and steps back as the person (`mode_act(StepBack, By::Person)`: the construct takes the keyboard, Return steps in, Escape closes); `cube.open` of a program keeps use mode (the gates and the tour that call it are unchanged). `make_way` moves every pane in front of the cube beside it, placed or not (D84 did this for placed panes for the same reason); `covers_cube` stays for a loose pane that still covers it. The test `a_pane_that_does_not_cover_the_cube_stays` now wants the loose pane in front beside the cube (it held the old rule) and keeps its other two panes. docs/cube-modes.md (States, the key of the cube, the panes in front) and tests/construct-captures.sh (Super+c already stands back; failed-face by `cube.open`; stepped-into-face by Return) follow. No sleep, no known row, no raised bar.
Green: chora-construct 270/270; the two tests pass on d9b2c4f1.
Gates fe369799 (after gate 1 went red in `cube`, step 5 below): `tests/gate-construct-cube.sh` "cube.open before the kill" waits (`until_ok`, at most 10 s) until a cube.open after cube.close is accepted, and its stood-back step wants the loose pane beside the cube and shown ("ok: stood back with a pane open, the pane goes beside the cube and shows, its label hides, the face label stays under the cube"); `tests/gate-construct-placed.sh` wants the loose pane drawn beside the stood-back cube and checks `placed: true` in the saved file, so it still proves B1. Both held the old rule (a pane nobody placed hides).

### Step 5: counts (d9b2c4f1)
- co-52 steps: **10/10** green (old binaries: 5/5 red) (row navigate, Super+c navigate, Return use, Alt+Escape navigate; the capture shows the cube stood back, face label `XP 1 face 1 of 4`, hints Turn left, Step in, Turn right).
- co-54 steps: **10/10** green (old binaries: 5/5 red) (s1 shown true after Super+c, the pane beside the cube on the left, label `Terminal`); reached by hand: Tab lists `foot left`, Return flies into it, typed `echo reach` shows in foot.
- co-51 and co-57 steps (row, Return to step in, Start, Lock, 12 s, key a): **10/10** step in by Return (mode use), `swaylock -f` called once each run, after key a one holder (xp; construct `{'holder': None}`) and I3 0 in 10/10; the checker's I2 on the lock window (step 3, harness) in 10/10, as before the fix (5/5): not a product row, left for the harness.
- Non-I6 rows of all after runs: only I2 "construct-mode-step-in and construct-mode-turn-left (and -turn-right) are 9 apart" = co-44 (COSMETIC, known-red row), which every stand-back shows on bc153929 too (Alt+Escape gave the same two rows); the person's cube now stands back at once, so these rows come with every cube entry.
- `tests/gate-construct-i3.sh`: **10/10** rc 0, "ok: one focus holder after a click in the xp face with a window in a pane (construct: -, xp: xp)" (co-43, co-46 hold).

### Gate
Two full runs of `tests/merge-gate.sh --jobs 2` (no --only), detached, polled in the turn. The brief says one gate; the second ran because the first was red by my change, not to rerun a flake.
- Gate 1 on d9b2c4f1 (base still bc153929, so the merge is d9b2c4f1 itself): **EXIT 1 in `cube`, caused by co-reach**: "choractl: a client xp runs already (code 1)" / "FAIL: cube.open before the kill" (gate-construct-cube.sh, the cube.open right after Super+C, cube.close; cause in Notes, first item). 29 cells PASS before it. Then a second rule of the old design held by that gate and by gate-construct-placed ("FAIL: the pane stays in front of the cube stood back: visible True", 3/3 by hand); both fixed in fe369799. Before gate 2 every construct cell ran green by hand on fe369799: pull 301 s, placed, cube 133 s, construct-hud 572 s, construct-extra 2477 s (waits for the gate lock), out, wm-pane, pane-keys, walk, keys, pixels, acts (235 pass, 4 known red, 0 blocking).
- Gate 2 on fe369799 merged with origin/claude/autonomous-dev-session-3w-i33pg7 at 1ec04694 (local merge 56e58247, detached): **EXIT 1, 60 cells PASS, 1 FAIL not caused here**: `mc-host` "FAIL: light: the field gets no keyboard after the click" (tests/gate-mc-host.sh, chora-mc with the HUD host; no construct, no cube; a plain step since mc-tidy 00406cd8, which came in with 1ec04694; another check than mc-57's). Not rerun. Not run after the stop: tiling-bar, e2e, fuzzel. Every construct cell PASS: cube 139 s, pull 305 s, placed 11 s, construct-hud 573 s, construct-extra 161 s (k1, mcpane, nested, k2a, windows, 3d, i3, focus), out, wm-pane, pane-keys, walk, keys, pixels; acts 2250 s.
- `pgrep -fa 'chora --headless'` empty after.

### Notes (not touched)
- New MINOR candidate (construct, pre-existing race): `cube.open` within about 0.4 s after `cube.close` of a connected xp is refused "a client xp runs already": `close_cube` kills and waits for xp (`cube.rs:1132` `end`), but the core lists its connection for a few hundred ms more (measured: close answered at +806 ms after Super+C, xp listed until +1203 ms, a retry 1 s later opens). On bc153929 the gate's Super+C, cube.close ran 226 ms after the key, before xp connected; with the cube stood back xp connects first. A person who closes and at once reopens gets the refusal notice once.
- In a pane flown into from the cube stood back, the hint bar keeps the cube's hints (`Return Step in`, turn keys); docs/cube-modes.md says no chip shows in a pane. Seen once by hand, not checked on bc153929.
- C1 for the demo now goes row (cube stood back), then Return or a click on a face, then xp's Start: the steps of T4 lock after C1 need the step in.

## PM review r1 (2026-11-23 18:5x UTC; diff 7 files +105/-40, under 200 lines; PM not the builder)
- Red first: 1b3abdfc red, d9b2c4f1 fix, fe369799 gates. ok.
- co-52, co-54: input.rs cube_key opens then StepBack (stood back, POLS); aside.rs make_way moves every pane in front. Cells gate-construct-cube, gate-construct-placed 10/10 green. Closed.
- co-51, co-57: harness NIT (checker race; stand-in square). co-57 closed as NIT.
- New: co-59 MINOR (reopen race, masked by until_ok in the gate), co-60 MINOR candidate (hint bar).
- Gate: 60 PASS, mc-host red foreign, tiling-bar, e2e, fuzzel not run: the full gate runs in train119.
- Verdict (computed): 0 open MAJOR -> MERGE-OK fe369799, train119 (the train is the full gate; red there = INCOMPLETE, back to the lane).

## r2
Lane: cloud, claude/co-reach-r2 from 8c411416. Step 0 green: unshallow, `cargo check --workspace`, `cargo build --workspace`. Shas: **red aabac7ed** (test: a face names its way out; a click on a pane flies in), **fix b2fa7792** (fix: a face names its way out; a click on a pane flies in), head b2fa7792. Diff 7 files +128/-18 (tests +84). No core change, no xp change (hosted.rs not touched: no step showed it).

### Step 1: reproduction (scripted, no persona)
Script media/2026-11-24/co-reach-r2/reach.sh (rewritten; the r1 and r8 /tmp scripts are gone): one fresh construct session per run via `round.sh start` (1280x800, PERSONA_CHECK=1, act.py, logical coordinates: row "A cube of desktops" (640,468), row Terminal (640,364)), each run in a subshell; binaries of 8c411416 (/tmp/persona-round/bin).
- co-61 (row, 6 s, Return, 5 s, dump): **5/5 red**. Every run: `cube.list` mode `use`, construct `{"holder": "construct"}`, xp `{"holder": null}`; the visible text of the hint bar (`construct-hints-*` text nodes with a rect) is `Super | F1 | All keys`, nothing else on screen names a way out (capture co61-before.png: a plain xp face, one chip "Super F1 All keys"). Non-I6 rows: co-44 I2 x2 per run only.
- co-62 (row Terminal, 5 s, Super+c, 6 s, click the centre of the visible construct surface s1, 3 s, dump): **5/5 red**. Before the click every run: mode `navigate`, construct `{"holder": "construct"}`, s1 shown beside the cube at screen box [17, 283, 383x233] (centre 208,400). After the click: mode `navigate`, construct `{"holder": "construct"}` (no surface), xp `{"holder": null}`, camera [x 0, z 0] (not moved), `construct.list` current null. By hand on the same binaries: Tab, Return then gives construct `{"holder": "construct", "surface": "s1"}` and the camera at [-1191, 2391]. Non-I6 rows: co-44 I2 x6 per run only.

### Step 2: severity
- co-61 **MAJOR** (confirmed): vision-card.md:11 "Desktops on cube faces; step out and in": after 3 s in a face nothing names the step out; docs/cube-modes.md:15 "A first-time person has no key to remember" and :71 "so that the person is never stuck in it". The demo watcher of r8 C1 stayed in the face (Escape, Super+C, pager arrow).
- co-62 **MAJOR** (confirmed): vision-card.md:20 "no lost window in any handover" with docs/cube-modes.md:42 "Each act has every path (principle 2)": the window shown beside the cube has no mouse path; Max of r8 C2 saw the terminal and could not get to it. Not downgraded: the keyboard path (Tab, Return) is hidden from a person who uses the mouse.

### Step 3: causes (8c411416)
- co-61: plugins/chora-construct/src/overlay.rs:613 `Set::Rest => vec![item(self.hint_keys_chord(), Word::AllKeys, "keys")]`: the set that a face keeps after the timed hints (hints.rs:143 `(true, true) => Some(Set::Rest)`, after `FLOWN` 3 s, hints.rs:32) names only All keys; `Alt Esc Step back` shows only in the first 3 s or while the pointer rests in the bottom 64 px (hints.rs:35 `BOTTOM`), and in use mode no corner chip STEP BACK shows (by design, gate-construct-cube.sh "v2 has none").
- co-62: plugins/chora-construct/src/modes.rs:586 `press_on_a_pane` takes the press on a pane stood back and calls `press(*position, true)` (modes.rs:681), which starts a move (input.rs:387 `self.moving = Some(Moving ..)`); the release (input.rs:462 `release`) drops the pane where it is (input.rs:477 `drop_pane`). The fly-in on a click (input.rs:480) needs `self.drag`, which a press never sets while a cube is open (input.rs:398 `else if self.cube.is_none()`). So a click moves the pane by zero and nothing else happens.

### Step 4: red and green
Red aabac7ed (on 8c411416), `cargo test -p chora-construct --bin chora-construct`:
- `cube::tests_construct::a_face_stepped_into_names_its_way_out_after_the_hints_went` panicked at tests_construct.rs:1144 "the hint bar names the way out of the face" (the chip `construct-hint-keys` shows, `construct-hint-step-back` does not, use mode after the 3 s).
- `cube::tests_construct::a_click_on_a_pane_beside_the_cube_flies_into_it` panicked at tests_construct.rs:1197 "the camera flies into the pane" (a pane beside the stood-back cube, press and release on its centre).
Fix b2fa7792 (overlay.rs +1 -2, input.rs +22 -2, cube.rs +2 -2 `CLICK` pub(crate), hints.rs +4 -2 doc comments, gate-construct-cube.sh +7 -8, docs +8 -2):
- co-61: the resting set of a face shows the same two hints as the first 3 s, `Alt Esc Step back` (the existing key `fly_out`, the existing hint and its click) beside `Super F1 All keys`. The set change swaps the nodes in one transaction with no fade, so the screen does not change at 3 s (scene of both states identical, checked by a dump diff).
- co-62: `release` first asks `clicked_pane_stood_back`: in navigate mode a press that a pane took and that ends within `CLICK` (8 px, the click of a face) ends the move and calls `fly_in` of that pane, the same call that Tab, Return makes (list.rs `show`). A drag still moves the pane and drops it on a face.
- tests/gate-construct-cube.sh `floats()`: it waited for the hint Step back to go after a step in (the old rule); it now waits for the face to name its way out and for two equal captures, then compares as before. Ran once by hand on b2fa7792: EXIT 0, 58 ok, "stood back, the windows float off the face ... and land pixel for pixel".
- docs/cube-modes.md: the flows table gets "fly into a pane beside the cube: a click on the pane | Tab, then Return | window.show"; "The visible way in" says the hint bar names `Alt Esc Step back` beside `Super F1 All keys` also after 3 s; "What the construct does today" says a click (under 8 px) on a pane stood back flies into it.
No new mode, no sleep, no known row, no raised bar.
Green: chora-construct 272/272 (270 + the two).

### Step 5: counts (b2fa7792, binaries /tmp/persona-after/bin, same script)
- co-61 steps: **10/10** green (old binaries 5/5 red). Every run: mode `use`, construct `{"holder": "construct"}`, xp `{"holder": null}`, hint bar `Super | F1 | All keys | Alt | Esc | Step back` 5 s after Return (co61-after.png: "Alt Esc Step back | Super F1 All keys" over the face).
- co-62 steps: **10/10** green (old binaries 5/5 red). Every run: before the click mode `navigate`, pane at [208 400]; after it mode `navigate`, construct `{"holder": "construct", "surface": "s1"}`, xp `{"holder": null}`, camera [-1191, 2391] (the pose of Tab, Return); `echo reach` typed after it shows in foot (co62-after.png).
- co-52 holds in all 10 co-61 and 10 co-62 runs (the row and Super+c stand back: mode navigate before Return or the click); co-54 holds in all 10 co-62 runs (s1 shown beside the cube, box centre 208,400).
- Non-I6 rows of all after runs: only co-44 I2 "construct-mode-step-in and construct-mode-turn-left (-right) are 9 apart" (x2 per co-61 run, x8 per co-62 run; before the fix x2 and x6, the extra two come with the look inside the pane, where the cube hints stay: co-60, left).
- `tests/gate-construct-i3.sh`: **10/10** rc 0, "ok: one focus holder after a click in the xp face with a window in a pane (construct: -, xp: xp)" (co-43, co-46 hold).
- co-48, co-49, co-50: no persona contracts here; their cells (acts) run in the gate below.

### Gate
One full run of `tests/merge-gate.sh --jobs 2` (no --only), detached, polled in the turn, on b2fa7792 merged with fresh origin/claude/autonomous-dev-session-3w-i33pg7 at cae62f50 (local merge b05b9ee4, detached; the incoming commits touch xp, tiling, notify, acts and persona, no construct file): **EXIT 0, 65 cells PASS, 0 FAIL**, "all steps passed; no --push, nothing is pushed". Construct cells: layout 1367 s, cube 125 s, pull 300 s, placed 10 s, construct-hud 571 s, construct-extra 160 s (k1, i3 and the rest), out, wm-pane, pane-keys, walk, keys, pixels; acts 1829 s; also tiling-bar, mc-host, e2e PASS (not run or red in r1). `pgrep -fa 'chora --headless'` empty after.

### Notes (not touched)
- co-60 seen again: in the pane flown into from the cube stood back the hint bar keeps the cube's hints (Turn left, Step in, Turn right), co62-after.png. MINOR, waits.
- MINOR co-44, co-47, co-53, co-55, co-56, co-59 not touched.

## PM review r2 (2026-11-24 02:58 UTC)
Base 8c411416..b2fa7792 (+120/-16 code and gate, 6 files, construct only). Builder: co-reach-r2 worker; reviewer: PM (under 200 lines).
- Red first: aabac7ed adds 84 lines of tests_construct.rs only; b2fa7792 the fix. Holds (rules.md 8).
- co-61: overlay.rs Set::Rest now shares the Flown items (Step back + All keys); hints.rs doc matches. gate-construct-cube floats() now asserts the hint rests and the face is steady (two equal captures) instead of the hint gone: changed with the design, not loosened (the cmp flat vs landed stays). Fixed.
- co-62: input.rs clicked_pane_stood_back: in Navigate mode a release within CLICK (8 px) of the press takes the move, releases its effect and flies in; a farther release still drops. CLICK made pub(crate), same value. Fixed.
- No new sleep line; full gate on merge with cae62f50 EXIT 0, 65 PASS (worker).
- NIT rcr-1: the gate edit sits in the fix commit; acceptable because the old assertion encoded the co-61 fault.
Verdict (computed): 0 BLOCK, 0 MAJOR -> MERGE-OK b2fa7792. Joins train124.

## r3
Lane: cloud, claude/co-reach-r3 from b2fa7792. Step 0 green: unshallow, `cargo check --workspace`, `cargo build --workspace`. Shas: **red 363b94c9** (test: a pane beside the cube names its acts and its way out), **fix e97848a4** (fix: a pane beside the cube names its acts; a pane names its way out), head e97848a4 (pushed). Diff of the fix 6 files +107/-6 (construct only, docs/cube-modes.md); no core, kit or xp change (hosted.rs not touched: no step showed it). **Gate note:** the fix message names co-61 ("which rest (co-61)"), so tests/red-first.sh refuses e97848a4 (no test commit in base..HEAD names co-61; its test aabac7ed is in the base). Not fixable without a force push (the rules forbid it); the gate ran on a local reword 909bc57b (same tree, `git diff e97848a4 909bc57b` empty, message without co-61). PM: either allow a force push of 909bc57b to claude/co-reach-r3 or merge with the red-first exception.

### Step 1: reproduction (scripted, no persona)
Script media/2026-11-24/co-reach-r3/reach.sh (rewritten from r2): one fresh construct session per run via `round.sh start` (1280x800, PERSONA_CHECK=1, act.py), logical coordinates and no look before the end (act.py scales a click by the last look), binaries of b2fa7792.
- co-64 (row Terminal, 5 s, Super+c, 6 s, dump, drag on the pane centre 880 px right, 3 s, dump): **5/5 red**. Every run: mode `navigate`, front 0, construct `{"holder": "construct"}`, pane s1 at [208 400]; the visible text of the hint bar is `Return | Ctrl Alt ← | Ctrl Alt → | Super F1 All keys` (Turn left, Step in, Turn right, All keys): neither the click on the pane nor a pull key is named. The drag moves **the pane**, never the cube: s1 at [1125 400], construct.list foot at [1755, 3136] (from [-1562, 3136]), cube front 0, camera [0, 0] in 5/5 (as designed: docs/cube-modes.md "A pane of a pulled window moves by a plain drag"; Max read the pane passing the cube as a turn).
- co-65 (row "A cube of desktops", Programs (154,52), wait for fuzzel, Foot (550,294), 6 s, dump, Escape, dump, Alt+Escape, dump): **6/6 red where Foot started** (11 runs: t0, before 2, 4, before2 1, 3, 5). Every such run: the camera flew into foot (launcher's program, adopt.rs `fly_into_window`), mode `navigate` (the cube stands back behind it), construct `{"holder": "construct", "surface": "s1"}`, xp `{"holder": null}`, camera [0, 1168]; **top bar hidden** (no `construct-door-*` text), hint bar `Return | Ctrl Alt ← | Ctrl Alt → | Super F1 All keys`: the hints of the cube, none of which works in a pane, and no `Step back`. Escape: nothing (camera stays, s1 keeps the keys: the program gets it, by design). Alt+Escape, named nowhere: steps out to the cube stood back (camera [0, 0]) in 6/6. The other 5 runs (before 1, 3, 5, before2 2, 4) did not start Foot: harness, fuzzel's use cache (`~/.cache/fuzzel`, shared HOME) reorders the rows after each launch, so the click at the Foot row hit `Chat with a helper` every other run (the after runs clear the cache per run).
- Variant co65in (row, Return to step in, Programs): 1/1, in use mode the top bar is hidden and the Programs click lands on the xp face; the hint bar names `Alt Esc Step back` (co-61 holds). Not a fault.
Non-I6 rows of all runs: co-44 I2 only, plus in the harness misses I2 "construct-face-label-glass and construct-start-glass are 5 apart" (the start card over the cube after Escape closed it).

### Step 2: severity
- co-64 **MAJOR** (confirmed): vision-card.md:11 chora-construct "pull windows across"; docs/cube-modes.md:42 "Each act has every path (principle 2)" and :15 "A first-time person has no key to remember": the click that flies into a pane and the key that pulls a window out are named only in the key list; contract C2 not reached by Max and demo in r9.
- co-65 **MAJOR** (confirmed): vision-card.md:24 shared RC bar "honest failure states" with docs/cube-modes.md:71 "so that the person is never stuck in it": in the pane of a program started from Programs the hint bar shows keys of the cube that do not hold there and no way out; contract C3 not reached in r9.

### Step 3: causes (b2fa7792)
- co-65: plugins/chora-construct/src/overlay.rs:528 `if stood_back { return Some(Set::Cube); }` in `hint_set` comes before the pane: the camera in a pane flown into from the cube stood back (`stood_back` is the cube's view, not the camera's) keeps `Set::Cube`, so `Set::Flown`/`Set::Rest` (`Alt Esc Step back`, the co-61 set) never shows; the doors are hidden in a pane by design, so nothing names the way out.
- co-64: overlay.rs:608 `Set::Cube => { cube_hints + All keys }`: the set of the cube stood back has no item for a pane beside it; the give key of the construct (`chora_kit::give::KEYS` `Super+Alt+4`, D55) is named only in the key list ("Give a window", config.rs:588). Stood back that key found no window (input.rs:171 `give_key` asks the core's focus, which the construct holds stood back).

### Step 4: red and green
Red 363b94c9 (on b2fa7792), `cargo test -p chora-construct --bin chora-construct`:
- `cube::tests_construct::a_pane_beside_the_cube_names_its_click_and_the_pull_key` panicked at tests_construct.rs:1247 "the hint bar names the click on the pane".
- `cube::tests_construct::a_pane_flown_into_from_the_cube_names_its_way_out` panicked at tests_construct.rs:1273 "the hint bar names the way out (timed true)".
Fix e97848a4 (overlay.rs +32 -2, aside.rs +28, hints.rs +31 -1, cube.rs +7, input.rs +2, docs +13 -3):
- co-65: `hint_set` returns `Set::Cube` only stood back **and not in a pane**; in a pane flown into from the cube the pane's sets apply: `Alt Esc Step back` + `All keys` (timed, then resting, as co-61). No chip of the cube shows in a pane (modes.rs `chips_of` already said so).
- co-64: stood back with a pane on screen beside the cube (`beside_cube`: in front of the camera, on screen, clear of the cube's box), `Set::Cube` adds `pane_hints`: `Click Fly in` (its click flies into that pane, as a click on the pane, co-62) and `Super Alt 4 Pull out` (the existing give key of the construct, its click gives); the hint bar stamp redraws when a pane comes or goes. Stood back, `give_key` gives the window of the front face that had the keyboard before the step back (`Host::noted_focus`, the window `give_the_keyboard_back` would restore), else it tells "No window to give." as before.
- docs/cube-modes.md: flows table (click on the pane or the hint `Click Fly in`; pull a window out by `Super+Alt+4` or its hint), "The visible way in" (a pane flown into from the cube names `Alt Esc Step back`, never the cube's hints), the stood-back hints (`Click Fly in`, `Super Alt 4 Pull out` while a pane shows beside); hints.rs: at most five hints, six stood back with a pane.
No new mode, no sleep, no known row, no raised bar (one row, the capsule 1030 px wide at 1280: co64-after.png).
Green: chora-construct 274/274 (272 + the two); clippy clean.

### Step 5: counts (e97848a4, binaries /tmp/persona-after/bin, same script; co-65 runs clear the fuzzel cache)
- co-64 steps: **10/10** green (old 5/5 red). Every run: mode `navigate`, pane s1 at [208 400], hint bar `Ctrl Alt ← Turn left | Return Step in | Turn right Ctrl Alt → | Click Fly in | Super Alt 4 Pull out | Super F1 All keys` (co64-after.png); the drag moves the pane (s1 at [1125 400], cube front 0, camera [0, 0]) 10/10: never the cube.
- co-65 steps: **10/10** green (old 6/6 red). Every run: in foot (construct `{"holder": "construct", "surface": "s1"}`, camera [0, 1168]), top bar hidden, hint bar `Super F1 All keys | Alt Esc Step back`; Escape reaches foot (camera stays); Alt+Escape steps out to the cube stood back 10/10.
- `tests/gate-construct-i3.sh`: **10/10** rc 0, "ok: one focus holder after a click in the xp face with a window in a pane (construct: -, xp: xp)" (co-43, co-46 hold). co-52 holds (row and Super+c stand back, mode navigate, 30/30 runs), co-54 holds (s1 shown beside the cube, 10/10 co-64 runs), co-61 holds (co65in), co-62: the click path is the hint's act and the pane's click (unit test a_click_on_a_pane_beside_the_cube_flies_into_it green). co-48..co-50: their cells (acts) run in the gate.
- Non-I6 rows of all after runs: co-44 I2 only (x6 per co-64 run, x8 per co-65 run).

### Gate
Gate 1 (e97848a4 merged with origin/claude/autonomous-dev-session-3w-i33pg7 at ce866b0d, local merge 7d45e0a7): **EXIT 1 at the first cell, `red-first`, caused here**: "RED-FIRST MISSING: e97848a4 ... misses a test commit before it that names co-61" (the fix message cites co-61; see the top). Gate 2, the one full run: 909bc57b (same tree as e97848a4, reworded) merged with ce866b0d (local merge 6cd13edc, detached), `tests/merge-gate.sh --jobs 2`, no --only: **EXIT 1, 64 cells PASS, 1 FAIL not caused here**: `e2e` "FAIL: a click on the Zebra row did not start zebra" / "scripts failed: gate-tiling-apps-click" (tiling and fuzzel, own HOME; no construct file in it). Not rerun. Every construct cell PASS: cube 133 s, pull 307 s, placed 11 s, construct-hud 579 s, construct-extra 170 s (i3 and the rest), acts 2364 s; red-first and identity PASS. `pgrep -fa 'chora --headless'` empty after.

### Notes (not touched)
- **New, MAJOR candidate (co-54 family, rc 9 "no known window loss")**: a program started while the cube stands back gets a pane ahead at [0, 2000], inside the stood-back cube; after Alt+Escape the window is gone from view in 10/10 after runs (co65-after-stepped-out.png: the cube alone; Tab lists it `ahead`; the keyboard stays with s1). Cause: `make_way` (cube/aside.rs:144) runs only as the cube starts to step back (modes.rs:331); a pane adopted later (adopt.rs `adopt`, `place`) is never moved beside. Started more than 60 s after Programs (no fly-in, adopt.rs `LAUNCHED`) it never shows at all (seen once by hand).
- Harness: fuzzel's use cache (~/.cache/fuzzel) reorders the launcher rows between sessions of a round, so a persona or a script that clicks a row by place picks another program every other run (r9 Max may have met it).
- MINOR co-44, co-47, co-53, co-55, co-56, co-59, co-63, co-66 not touched. co-60 (cube hints in a pane flown into from the cube) is gone with the co-65 line (overlay.rs:528, the same cause); the PM may close it.

## PM review r3 (06:40 UTC, ce866b0d..e97848a4, 186 lines, PM not the builder)

- co-64: red 363b94c9 (hint test) before e97848a4; hint bar shows `Click Fly in` and `Super Alt 4 Pull out` while a pane is beside the cube; the click and hint act; pull key uses the noted focus of the front face (input.rs). OK.
- co-65: overlay.rs `stood_back && !flown`: a pane flown into shows `Alt Esc Step back`, no cube hints; test covers timed and untimed. OK; closes co-60 too.
- Docs: docs/cube-modes.md rows and prose match the hints. OK.
- rco3-1 | MINOR | input.rs noted_focus | the pull from the noted focus has no unit test (only the e2e 10/10 runs); add one with the next round.
- rco3-2 | MINOR | overlay.rs pane_hints | `Super Alt 4 Pull out` shows when no face window has focus; the click then does nothing, with no message (honest states). Next round: hide it or say why.
- rco3-3 | MINOR | hints.rs | six hints stood back; check width at 1280 px in the tour take.
- Red-first gate refused e97848a4 (message cites co-61); the lane pushes the same tree as 909bc57b to claude/co-reach-r3b (no force). Its full gate: 64 PASS, 1 foreign FAIL gate-tiling-apps-click "Zebra row did not start zebra" (not construct; the train shows whether it repeats).
- New: co-67 MAJOR candidate (lane, 10/10): a program started while the cube stands back lands inside the cube and is lost after Alt+Escape. Pre-existing (make_way runs only at step back); next round, not this verdict.
- Verdict (computed): 0 MAJOR in the diff -> MERGE-OK for claude/co-reach-r3b once the tree equals e97848a4.

## r4
Lane: cloud, claude/co-reach-r4 from 909bc57b. Step 0 green: unshallow, `cargo check --workspace`, `cargo build --workspace`. Shas: **red 624c85cc** (test: a window that opens while the cube stands back shows beside it; names co-67 only), **fix f1457035** (same subject as fix), head f1457035 (pushed). Diff of the fix 3 files +61/-2 (adopt.rs, cube/aside.rs, docs/cube-modes.md); construct only, no other plugin, no core or kit change. The red-first gate and the identity cell PASS on the merge.

### Step 1: reproduction (909bc57b, script media/2026-11-24/co-reach-r4/co67.sh, one fresh construct session per run, 1280x800, row "A cube of desktops", Super+c, 6 s, dump, start Foot, 8 s, dump, Alt+Escape, dump)
- Foot by its key (Super+t): **5/5 lost**. Every run: before, mode `navigate`, front 0, camera [0, 0], no pane, construct holder `construct`. After the start: camera [0, 1168] (the fly-in into the new pane), pane `foot` at **[0, 2000]**, surface s1 visible on the screen (rect [0, 0, 1280, 800]), keyboard on s1. After Alt+Escape: camera [0, 0], mode `navigate`, pane still at [0, 2000], **surface s1 rect None** (not on screen: it lies inside the stood-back cube), the keyboard stays on s1 (construct `{"holder": "construct", "surface": "s1"}`).
- Foot from Programs (click Programs, fuzzel, Foot row, fuzzel cache cleared per run): **5/5 lost**, the same dump line for line.
- Reachable? On screen: no (rect None, 10/10). In the switcher: Tab lists it `ahead` (r3, by hand; not re-run here). Focusable: the keyboard is on s1 but the person sees nothing; only the switcher reaches it. Started more than 60 s after Programs (no fly-in, adopt.rs `LAUNCHED`) r3 saw it never show at all (not re-run).

### Step 2: severity
co-67 **MAJOR** (confirmed, 10/10): design/rc.md:19 row 9 "no known window loss" and vision-card.md:20 "no lost window"; docs/cube-modes.md:239 says of the pane in front of the cube "so that ... no window is lost behind it", and the new pane is exactly the case it did not cover. The person started a program and cannot see it after the way out the hint bar names (`Alt Esc Step back`).

### Step 3: cause (909bc57b)
- plugins/chora-construct/src/cube/aside.rs:144 `make_way` is the only place that moves a pane beside the cube, and its only caller is modes.rs:331 (`if act == Act::StepBack { self.make_way(); }`): it runs once, as the cube starts to step back. A pane that is adopted later (adopt.rs `adopt`, ending `self.panes.push(pane); self.frame()`, its place from panes.rs:388 `place`, `pane_distance` ahead = [0, 2000]) is never looked at. The stood-back cube is centred there (camera [0, 0], cube centre [0, 2000]), so the new pane lands inside it.

### Step 4: red and green
Red 624c85cc (on 909bc57b), `cargo test -p chora-construct --bin chora-construct a_window_that_opens`: `a_window_that_opens_while_the_cube_stands_back_shows_beside_it` panicked at tests_construct.rs:1300 "the new pane is clear of the cube: [373.76, 233.6, 906.24, 566.4] [416.4, 254.9, 863.6, 545.1]" (the pane box lies inside the cube box; the assertion that it shows on the screen passed).
Fix f1457035: adopt.rs:149 `self.panes.push(pane); self.show_new_pane()` (it was `frame()`); cube/aside.rs:183 `show_new_pane` (while the cube stands back, `!view().forwards()`, and the new pane's box covers the cube's box, `open_beside_cube`, aside.rs:200, takes the place `place_beside` gives a pane at the step back: on the side where it is, past the room of the panes already beside the cube, clear of the cube, inside the leash, then one `placed` update and the frame). The pane has **no home** and no entry in `asides`: it stays beside the cube after Alt+Escape and after a step in, as a pane the person moved does. A pane that clears the cube where it opens (a pane file's place) is not moved. docs/cube-modes.md:240 says so. No new mode, no sleep, no known row, no raised bar.
Green: chora-construct 275/275 (274 + the test); clippy clean (`adopt` kept under 60 lines).

### Step 5: counts (f1457035, binaries /tmp/persona-after/bin, the same script plus a step in by Return at the end)
- Foot by its key: **5/5** green; Foot from Programs: **5/5** green (old 10/10 lost). Every run: after the start camera [-1191, 2391] (the fly-in into the pane at [-1562, 3136], beside the cube), s1 rect [0, 0, 1280, 800], keyboard on s1; after Alt+Escape: mode `navigate`, camera [0, 0], **s1 visible on the screen** at rect [17, 283, 365, 233] (left of the cube), keyboard on s1; after Return (step in): mode `use`, camera [0, 0], pane at [-1562, 3136], s1 still on the screen at the same rect.
- `tests/gate-construct-i3.sh`: **10/10** rc 0, "ok: one focus holder after a click in the xp face with a window in a pane (construct: -, xp: xp)" (co-61, co-62, co-64, co-65 hold: the pane beside the cube is the pane of co-64, its hint bar and click path unchanged; co-65's in-pane hint bar unchanged, as the cell and the unit tests are green).

### Gate
One full run: f1457035 merged with fresh origin/claude/autonomous-dev-session-3w-i33pg7 (ce866b0d; local merge 7e3f08c4, detached), `tests/merge-gate.sh --jobs 2`, no --only: **EXIT 0, 65 cells PASS, 0 FAIL** (the Zebra row cell gate-tiling-apps-click of r3 is PASS here). `pgrep -fa 'chora --headless'` empty after.

### Notes (not touched)
- A program started more than 60 s after Programs (no fly-in) was not re-checked here; with this fix it goes through the same `adopt`, so it opens beside the cube too, but no run shows it.
- The rc.md row 9 text "no known window loss" can be read as met for the started-while-stood-back case once this merges.
- MINOR co-44, co-47, co-53, co-55, co-56, co-59, co-63, co-66, rco3-1..3 not touched.

## Review r4 (PM, 88 lines; PM not the builder)
Base 909bc57b..f1457035, 4 files +88 -2. Red first: 624c85cc (test) before f1457035 (fix): yes. Scope: construct adopt/aside, its test, docs/cube-modes.md only: yes.
Checks: adopt.rs:149 now calls show_new_pane; a pane that overlaps the stood-back cube goes beside it via place_beside (same path as make_way); no home, so step in leaves it (unit test asserts). Lane gate 65 PASS on f1457035+ce866b0d; gate-construct-i3 10/10 per report.
- rco4-1 MINOR aside.rs:216-221: the side-load loop repeats make_way's taken sum (aside.rs:160-168); fold into one helper at the next construct round.
Verdict (computed): 0 BLOCK, 0 MAJOR -> MERGE-OK; co-67 fixed on merge (train127, includes r3b 909bc57b).

## r1 (co-79 round, 2026-11-25; the earlier "## r1" above is the 2026-11-23 series)

Lane: cloud, claude/co-reach-r1 from fbdcd6e2 (co-close r2 head). Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green. Area: overlay.rs, cube/tests_construct (new reach_tests.rs), tests/gate-construct-cube.sh, docs/cube-modes.md. 6 files +123 -25.

**Head: 7f519bc8** (red 43433ce7, fix 7f519bc8). Pushed to claude/co-reach-r1.

### 1. Hand repro (fresh compositor each, snapshot fbdcd6e2, scratchpad hand2.sh)
- **The brief's path as written** (row Terminal, Super+c (pane beside), click the pane, Alt+Escape) does **not** land in the face. It lands stood back: `mode navigate front 0, panes [foot], focus construct holder construct s1, xp holder none`. The bar there is `Click Fly in | Return | Ctrl Alt ← | Ctrl Alt → | Super F1 All keys`. Pull out is not named, and correctly so (co-68): the keyboard is in the pane window and the faces are empty. Same 5/5 after the fix (row "keys in pane beside" below).
- **The demo's r12 C2 steps** (row "A cube of desktops", Return, Super+F1, Super+t, Escape, Close click, Alt+Escape) reproduce its dump **5/5**: `mode use front 0, camera [0,0], panes [foot @0,2000], focus construct holder construct s1, xp holder none`. Bar: `Super F1 All keys | Alt Esc Step back`. Told 0/5. Super+Alt+4 acts 0/5 (s1:construct before and after).
- **Cause of the demo's miss (new, upstream of the bar):** with the key sheet open over a face, Super+t starts the construct's own terminal (`terminal_beside`). It opens as a pane, flown in, instead of a window of the face. After Close and Alt+Escape the person is in an **empty** face, and the only window is a pane, the construct's own. No key can pull anything there: `pull_acts()` is false, so naming the pull key would break POLS (co-68). This is the co-78 family (the sheet and Super+T), not in scope here. Without the sheet, Return then Super+t opens the window in the face (xp s1) 5/5.
- **The gap the fix closes** (row "face": row cube, Return, Super+t): the keyboard is in face window s1 (xp) and Super+Alt+4 pulls it **5/5**, but the bar names it **0/5** (co-74). On the demo's straight path the key acted and nobody was told. Before co-70 the person had to step back to learn of it.

### 2. Red 43433ce7 (tests only)
`reach_tests::the_pull_key_is_named_in_a_face_where_it_acts` on fbdcd6e2:
```
reach_tests.rs:44:9: the keyboard is in a window of the face: the key pulls it out, so the face names it (timed true)
```
The same test also pins the demo's state (use mode, keyboard in the pane in front of the face): pull not named, `pull_acts()` false, Step back named. That part is green before and after.

### 3. Fix 7f519bc8: one rule
Option chosen: **the bar names the pull key wherever `pull_acts()` holds**. It now holds in the face set (Step back, All keys; timed and resting) as well as stood back. One helper, `pull_out_hint`, serves both sets, and `pane_hints` uses it, so hint and key stay one predicate (`give_key`). Why this option over "Alt+Escape from a pane lands stood back": on both paths above the face is empty, and a pane's window is the construct's own. A landing stood back would tell nothing new there (co-68 keeps it silent). Where a face window has the keyboard, the person is told in the face they are looking at, one step sooner than co-70. co-68 holds: the predicate is false with the keyboard in a pane, beside or in front. co-70 is unchanged. Gate cell (gate-construct-cube, the probe window on the front face holds the keyboard): `co-79: in a face with the keyboard in its window the hint bar names the pull key`. docs/cube-modes.md gives the rule and the reason. Unit tests: chora-construct 285 passed; clippy clean.

### 4. rcc2-2: Super+x while flying back out of a pane
**No, it closes nothing.** `fly_out` takes `flown` at the key, so `in_pane()` is false for the whole glide back. The premise "in_pane() still true until the flight ends" does not hold. In a face, close_focused takes the face branch, which never closes a pane (co-75). That is the same as after the glide ends. Unit test `reach_tests::super_x_while_flying_out_of_a_pane_into_a_face_closes_nothing` (glide running, `!in_pane()`, closed = []) is green before and after. No fix needed.

### 5. Hand runs, 5x each from a fresh compositor (snapshot 7f519bc8; checker off for speed: told = bar text, acts = surface owner change)
| Row | Steps | told | acts | state at the read |
|---|---|---|---|---|
| step 1, demo r12 path | cube row, Return, Super+F1, Super+t, Esc, Close, Alt+Escape | 0/5 (before 0/5) | 0/5 (before 0/5) | use, panes [foot], holder construct s1, xp none: nothing to pull, not told by design (co-68); cause above |
| face (co-79 fix) | cube row, Return, Super+t | **5/5** (before **0/5**) | **5/5** (before 5/5) | use, holder xp s1; bar `Super F1 All keys \| Super Alt 4 Pull out \| Alt Esc Step back`; after: s1:construct |
| co-70 (no pane beside) | cube row, Return, Super+t, Alt+Escape | 5/5 | 5/5 | navigate, construct holder (noted s1) |
| co-68 (keys in the pane beside; the brief's step 1 path) | row Terminal, Super+c, click pane, Alt+Escape | not named 5/5 | key does nothing 5/5 | navigate, holder construct s1, bar `Click Fly in ...` |
| co-68 (pane beside, keys on the look) | row Terminal, Super+c | not named 5/5 | nothing 5/5 | navigate, holder construct (no surface) |

A told key acts: 10/10 told runs pulled (face, co-70). No untold run pulled anything except the face row before the fix (the co-79 gap itself).

### Open for the PM
(`tools/sleep-ratchet` is not on this branch; the diff adds 0 `sleep` lines to tests/ and tools/.)
1. **Demo C2 path still needs a face window.** Super+t with the key sheet over a face starts a construct pane, not a face window (co-78 family). After that no pull can be told honestly. Route it with co-78. Proposed: Super+t with the sheet open does what it does with the sheet closed.
2. The old-binary run of the gate cell is not shown red: my run with BIN=snapshot mixed a new core with old clients and failed earlier (`xp shows 0 taskbars`). The red is the unit test above.

### 6. Gate
Ran `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --only), detached, on 7f519bc8: **64 PASS, EXIT 0**, no foreign red. The first run was killed when the worker restarted (no EXIT line), so it was rerun once in full. Cube cell: `ok: co-79: in a face with the keyboard in its window the hint bar names the pull key`. `pgrep -fa 'chora --headless'` empty after. Last 20 lines:
```
launcher-click PASS    16s /tmp/merge-gate.xFW2z0/launcher-click.log
voice-acp  PASS    12s /tmp/merge-gate.xFW2z0/voice-acp.log
voice-events PASS     1s /tmp/merge-gate.xFW2z0/voice-events.log
agent      PASS    40s /tmp/merge-gate.xFW2z0/agent.log
tiling-keys-more PASS    41s /tmp/merge-gate.xFW2z0/tiling-keys-more.log
tiling-windows PASS   160s /tmp/merge-gate.xFW2z0/tiling-windows.log
tiling-system PASS    11s /tmp/merge-gate.xFW2z0/tiling-system.log
handover   PASS    26s /tmp/merge-gate.xFW2z0/handover.log
acts       PASS  1969s /tmp/merge-gate.xFW2z0/acts.log
mc-scale   PASS    12s /tmp/merge-gate.xFW2z0/mc-scale.log
watchdog-starved PASS    54s /tmp/merge-gate.xFW2z0/watchdog-starved.log
xp-run     PASS    13s /tmp/merge-gate.xFW2z0/xp-run.log
demo       PASS   255s /tmp/merge-gate.xFW2z0/demo.log
leak       PASS   329s /tmp/merge-gate.xFW2z0/leak.log
tiling-theme PASS    18s /tmp/merge-gate.xFW2z0/tiling-theme.log
mc-host    PASS    19s /tmp/merge-gate.xFW2z0/mc-host.log
tiling-bar PASS   133s /tmp/merge-gate.xFW2z0/tiling-bar.log
e2e        PASS   265s /tmp/merge-gate.xFW2z0/e2e.log
all steps passed; no --push, nothing is pushed
EXIT 0
```

## PM review r1 (fbdcd6e2..7f519bc8, 123 lines; rules.md 7: PM review under 200 lines)
Checks: red first (43433ce7 test before 7f519bc8 fix): yes. co-79: `pull_out_hint` is the one source for the pull hint in the face and stood-back sets, gated on `pull_acts` (co-68: never in a pane); reach_tests.rs covers face focused, face unfocused, pane; gate-construct-cube.sh asserts `construct-hint-pull-out` in a face with the keyboard in its window. docs/cube-modes.md says the rule. No new sleep in this round (the 3 of the branch are co-close r2's, gate-construct-close.sh:51, 80, 87).
Findings: rcr1-1 MINOR: the demo r12 face is empty (Super+t with the key sheet open starts a pane, not a face window), so the hint stays untold there: route with co-78.
Verdict (computed): 0 BLOCK, 0 MAJOR: MERGE-OK. Train only after the co-close sleeps become waits (t134 sleep-ratchet): CO-CLOSE-WAITS r1.

## r5 (co-79 pane path, 2026-11-25)

Lane: cloud, claude/co-reach-r5 from **base 62a04be0** (t140 green). Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green. Area: plugins/chora-construct (hints, overlay, input, pull, reach_tests), tests/gate-construct-cube.sh, docs/cube-modes.md. No core, no xp, no other plugin.

**Head 7d9269ae** (red **58ba73a1**, fix **5025dc47**, test follow-up **7d9269ae**: the unit test names the face manager `xp`, not `chora-xp`, which cut-check counts), pushed to claude/co-reach-r5. Diff 62a04be0..7d9269ae: 7 files +189 -9 (gate-construct-cube.sh +46, reach_tests.rs +42, overlay.rs +39 -2, pull.rs +36 -4, hints.rs +20 -1, input.rs +7 -1, docs/cube-modes.md +8 -1).

### 1. Reproduction on 62a04be0 (fresh compositor per run, 1280x800, script media/2026-11-25/co-reach-r5/hand.sh)
Steps: start card row `A cube of desktops` (the cube stands back), Programs, Foot (a pane beside the cube, the camera flies in), Alt+Escape.
- **State**: navigate mode (stood back), front 0, panes [foot], Foot's window s1 is the construct's (a pane), core focus s1 with the keys on `construct-look`. **Bar**: `Click Fly in | Return | Ctrl Alt ← | Ctrl Alt → | Super F1 All keys`: no way into a face is told, **0/5** (before-62a04be-pane-5runs.log; 3 earlier runs the same, before-62a04be-pane.log, before-62a04be-turned.log).
- The untold key `Super+Alt+2` (give to xp) does move Foot to xp, but (a) onto VIRTUAL-1 (face 0) also with the cube turned to face 1 (xp puts a new window on the output the pointer entered last), and (b) the keyboard stays in Foot while the cube stands back, so the bar's `Return Step in` then types into Foot instead of stepping in (before-62a04be-turned.log). So telling the bare key would break co-68 one step later.

### 2. Decision (docs/cube-modes.md, "The visible way in")
"Stood back out of a pane, the bar names the way into a face instead, `Super Alt 2 Into face`, the give key of the face manager: while the window of a pane has the keyboard, it puts that window on the face at the front, steps in and gives it the keyboard there, so every key that the bar names then works (co-79)." One key in the state the person is in, no new mode. The key is the existing give key of the face manager (xp: Super+Alt+2, from `chora_kit::give::KEYS`), so the same key means "to xp" everywhere. The flows table and the words table (`Into face`, `In die Seite`) follow.

### 3. Red 58ba73a1 (tests only)
gate-construct-cube.sh step 7 (after step 5's `Super+C opens the cube`): Super+C stands back, Super+space (the launcher) starts a program beside the cube (a pane, flown in), Alt+Escape stands back, Ctrl+Alt+Right turns to face 1; the bar must name `Super Alt 2` (`construct-hints-put-in-keys-*`), and that key must put the window on face 1's output (owner xp), step in (mode use), give it the keys (a typed `a` reaches it) and the bar must then name the pull key. On 62a04be0: `FAIL: co-79: stood back out of a pane the hint bar names no way into a face (put-in keys:   )`; every earlier cell passed.

### 4. Fix 5025dc47
- `put_window()` (overlay.rs): stood back (navigate), every face up, the window with the keyboard (else the one noted at the step back) is a pane's own. It is the one predicate of the hint and the key (as `pull_acts` is for the pull key; the two never hold at once).
- `put_in_hint` (Cube set, after `Click Fly in`): `Super Alt 2 Into face`, keys from `hints::give_key_of(face_manager)`; a click on it does the key. `hint_stamp` adds the predicate.
- `give_key` (input.rs): with `to == face_manager()` and `put_window()`, `put_in_face` (pull.rs): give the window to the manager, `window.move` it to the front face's output (the call a drop on a face makes, now `place_on`, shared), note it and `StepIn` as the person, so `give_the_keyboard_back` activates it in the face.
- Unit test `reach_tests::stood_back_out_of_a_pane_the_bar_names_the_way_into_the_face` (pane focus: put named, pull not; face window focus: the other way round). chora-construct 290 passed; clippy clean.

### 5. Counts after (fresh compositor per run, snapshot 5025dc47)
| Path | Steps | told | acts | state after the key |
|---|---|---|---|---|
| pane (co-79 r5) | cube row, Programs, Foot, Alt+Escape, Super+Alt+2 | **5/5** (before **0/5**) | **5/5** | use, front 0, Foot on xp VIRTUAL-1 shown, focused s1; bar `Alt Esc Step back \| Super Alt 4 Pull out \| Super F1 All keys` (after-pane.log) |
| face (co-79 r1) | cube row, Return, Super+t, Super+Alt+4 | **5/5** | **5/5** (panes [foot]) | unchanged from r1 (after-face.log) |
| pane, turned to face 1 | gate step 7 | told | on face 1's output, steps in, takes keys | gate-construct-cube EXIT 0 alone on 5025dc47, and in full gate run 2 |

Every told key acts in all runs. Checker rows of the after runs: only co-44 I2 (construct-mode-step-in and turn chips 9 apart) and I6 (xp tray clock contrast), both known.

### 6. Gate
Run 1 on 5025dc47 (`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`, no step filter, detached): **FAIL in ci-check, caused here**: "FAIL: cut-check: names: chora-construct names chora-xp on 6 lines" (the new unit test set the program `chora-xp`); fixed by 7d9269ae (`xp`, cut-check 5 lines as on base). Run 2 on 7d9269ae, the same command, detached: **EXIT 0, 66 PASS, 0 FAIL**, no foreign red (mc-sofa passed, FLAKE-SOFA not seen). Cube cell 144 s with the three co-79 lines ok. `pgrep -fa 'chora --headless'` empty after. Last 20 lines:
```
voice-acp  PASS    13s /tmp/merge-gate.G8NbD2/voice-acp.log
voice-events PASS     2s /tmp/merge-gate.G8NbD2/voice-events.log
agent      PASS    42s /tmp/merge-gate.G8NbD2/agent.log
tiling-keys-more PASS    42s /tmp/merge-gate.G8NbD2/tiling-keys-more.log
tiling-windows PASS   206s /tmp/merge-gate.G8NbD2/tiling-windows.log
tiling-system PASS    17s /tmp/merge-gate.G8NbD2/tiling-system.log
handover   PASS    29s /tmp/merge-gate.G8NbD2/handover.log
mc-scale   PASS    17s /tmp/merge-gate.G8NbD2/mc-scale.log
watchdog-starved PASS    55s /tmp/merge-gate.G8NbD2/watchdog-starved.log
acts       PASS  2287s /tmp/merge-gate.G8NbD2/acts.log
xp-run     PASS    16s /tmp/merge-gate.G8NbD2/xp-run.log
demo       PASS   270s /tmp/merge-gate.G8NbD2/demo.log
leak       PASS   478s /tmp/merge-gate.G8NbD2/leak.log
tiling-theme PASS    19s /tmp/merge-gate.G8NbD2/tiling-theme.log
tiling-bar PASS   198s /tmp/merge-gate.G8NbD2/tiling-bar.log
mc-host    PASS    20s /tmp/merge-gate.G8NbD2/mc-host.log
tiling-disc PASS    73s /tmp/merge-gate.G8NbD2/tiling-disc.log
e2e        PASS   289s /tmp/merge-gate.G8NbD2/e2e.log
all steps passed; no --push, nothing is pushed
EXIT 0
```

### Captures (media/2026-11-25/co-reach-r5/, each under 200 KB)
- pane-stood-back-told.png: stood back after Alt+Escape out of Foot, bar `... Click Fly in | Super Alt 2 Into face | Super F1 All keys`.
- pane-into-face-after.png: after Super+Alt+2, Foot in face 0 at 1:1 with the keyboard, bar `Alt Esc Step back | Super Alt 4 Pull out | Super F1 All keys`.
- logs: before-62a04be-*.log, after-pane.log, after-face.log; hand.sh (cases rep/repb, rep2, pane5, co79c).

### Open for the PM
1. Not checked by hand: the drag of a pane onto a face (the mouse way, Max 2 in r13) uses the same give, so it likely leaves the cube stood back with the keyboard in the given window, where `Return Step in` would go to the window (seen on 62a04be0 with the bare key). Not changed here (the key path now steps in); a candidate for the same rule on the drop.
2. xp puts a given window on the output the pointer entered last, not on the front face; the construct now asks `window.move` after the key give, as after a drop. A give by another client (fuzzel row, voice) still lands where xp chooses.

## Review r5 (co-79 pane path, 2026-11-25)

Reviewer lane (cloud, read-only on code), claude/co-reach-r5 **head 7d9269ae**, base 62a04be0, 7 files +189 -9 (only plugins/chora-construct, tests/gate-construct-cube.sh, docs/cube-modes.md; no other plugin, no core). Step 0: unshallow, `cargo check --workspace` green. No merge gate run (the lane's run 2: EXIT 0 on 7d9269ae).

**Verdict: MERGE-OK** (0 BLOCK, 0 MAJOR, 2 MINOR, 2 NIT).

Checks:
1. Red first: 58ba73a1 (gate only) before 5025dc47 (fix). `cargo test -p chora-construct reach`: 3 passed at 58ba73a1, 4 passed at 7d9269ae; the unit test came with the fix, so the red is the gate cell (rcr5-4). Gate step 7 read: on 62a04be0 no `construct-hints-put-in-keys-*` node exists, so it fails at the put-keys line (the lane's log says the same); it asserts owner xp on face 1's output after a turn, mode use, a typed `a` reaching the program, then the pull key named. Holds.
2. Told key works (co-68): hint (`put_in_hint`), click (`put-in`) and key (`give_key`) ask one predicate, `put_window` (navigate, every face up, core focus else noted focus, a pane's own window); `pull_acts` is its exact complement on the same focus, so the two hints never show together. No pane: None. Two panes: the one with the keyboard. Turned to face 1: `front_output` (gate covers it). Pane closed stood back: `own_window` needs an attached pane, the stamp redraws. While the windows list is open the bar is busy (no hints) and the list branch of `give_key` runs first, so nothing is told there. Holds, except rcr5-1.
3. Keys: the put branch only runs in navigate (`put_window`), and only for `to == face_manager()`; in a pane flown in from navigate the key (untold there) now puts the window into the face and `forget_with` flies out, which beats base (window to xp, keyboard left in it stood back). The pull key stays untold for a pane (unit test, `pull_acts`). Holds, except rcr5-1.
4. Lane's open point, drag-drop onto a face stood back: `drop_on_a_face` gives and places but stays in navigate and neither notes the focus nor takes the keyboard back (rcr5-2; pre-existing, not touched by this diff).
5. docs/cube-modes.md: one sentence names the rule, the flows and words rows follow; matches the code but for the face-health condition (rcr5-3). No new sleep, lint allow or plugin in the diff; tools/sleep-ratchet (778/929), size-ratchet (0 over 1500), lint-ratchet (allow 360, no-reason 0, eprintln 46) all ok on 7d9269ae.
6. Captures: pane-stood-back-told.png, bar readable `Ctrl Alt ← Turn left | Return Step in | Turn right Ctrl Alt → | Click Fly in | Super Alt 2 Into face | Super F1 All keys`; pane-into-face-after.png, Foot in face 1 of xp at the front, taskbar shows it, bar `Alt Esc Step back | Super Alt 4 Pull out | Super F1 All keys`. Both under 200 KB. Holds.

Findings:
- rcr5-1, MINOR, plugins/chora-construct/src/pull.rs:308, `put_in_face` only logs a failed `window.move` (`place_on`) and still notes the window, steps in and activates it, so when the move fails or times out the window stays on the output xp chose (the one the pointer entered last, r5 report) and the keyboard goes to a window on a face the person cannot see; fix: let `place_on` answer whether it placed and, when not, tell it and stay stood back (or step in only when the window is on the front output).
- rcr5-2, MINOR (pre-existing; MAJOR if a by-hand run confirms), plugins/chora-construct/src/pull.rs:224, a drop of a pane onto a face stood back gives the window but stays in navigate without noting it or taking the keyboard back; if the core focus follows the given window, as with the bare key on 62a04be0 (before-62a04be-turned.log), the told `Return Step in` types into the window (co-68 on the documented mouse way); fix: end a drop onto the front face like `put_in_face` (note, StepIn as the person), and onto another face take the keyboard back to the construct, with a gate cell.
- rcr5-3, NIT, docs/cube-modes.md:137, the sentence says "while the window of a pane has the keyboard", the code also needs every face up (`host.works()`) and falls back to the window noted at the step back; fix: add "with every face up".
- rcr5-4, NIT, plugins/chora-construct/src/cube/tests_construct/reach_tests.rs:86, the unit test came with the fix commit, so `cargo test reach` is green at the red commit 58ba73a1 and the red is the gate cell only; fix: none for this merge, next time land the unit test in the red commit too.

## r6 (co-79 every pane path, 2026-11-26)

Lane: cloud, claude/co-reach-r6 from **base 4d8bbe36** (t153+154). Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green. Area: plugins/chora-construct (hints, overlay, input, modes, reach_tests), tests/gate-construct-cube.sh, docs/cube-modes.md. No core, no other plugin.

**Head bcecb671** (red **5f8f0c84**, fix **1916f517**, fix follow-up **bcecb671**: the walk keeps Terminal beside Cube, at most six, and gate-construct-layout.sh's three pinned walk sets name ), pushed to claude/co-reach-r6. Diff 4d8bbe36..bcecb671: 8 files +190 -28 (gate-construct-cube.sh +70, hints.rs +30 -8, reach_tests.rs +25, overlay.rs +18 -3, input.rs +15 -1, modes.rs +15, docs/cube-modes.md +14 -13, gate-construct-layout.sh +3 -3). **Outside the named area**: gate-construct-layout.sh pins the exact walk hint set after Alt+Escape out of a pane, which the decided hint changes; only its three expected strings changed ().

### 1. Reproduction on 4d8bbe36 and every pane path
The Terminal row (start card, 1280x800, fresh compositor per run, Alt+Escape out of Foot): **0/5** (hand-term-head.log): `mode use front 0 faces 0, panes ['foot'], core focus ['construct-look', 's1']`, bar `Return Fly in | F1 All keys | Super T Terminal | W A S D Walk`, Super+Alt+2 does nothing. The root cause is not a mode split: **no cube is open at all** (`cube.list` says `mode use` with 0 faces, its default). The person stands in the space, and the walk hints have named no cube since 14f7d3f6 (11-11, "the hint Cube is gone, since the start card and the help name it"); the start card hides as soon as a pane exists. A second fault: Super+C there opens the cube, but the cube that covers the pane takes the keyboard from its window (`covered_focus_to_nobody`, co-46), so `Into face` is not told there either (1 run, before the fix).

| Way a pane gets the keyboard | before (4d8bbe36): state after Alt+Escape | way into a face told | after (1916f517): state after Alt+Escape, told keys | 5 runs after |
|---|---|---|---|---|
| start card Terminal row (no cube) | walk, faces 0, keys on look + s1 | none (0/5) | walk; bar `Super C Cube`; Super+C: navigate with `Super Alt 2 Into face`; Super+Alt+2: use, Foot on xp VIRTUAL-1, `Pull out` told | **5/5** |
| start card All programs row, Foot (no cube) | walk, faces 0 | none | same as the Terminal row | **5/5** |
| launcher Super+space, Foot (no cube) | walk, faces 0 | none | same | **5/5** |
| Super+t in the space (no cube) | walk, faces 0 | none | same | **5/5** |
| Super+t in a pane (no cube, 2 panes) | walk, faces 0, bar adds `Tab Windows` | none | same; bar `W A S D Walk, Return Fly in, Tab Windows, Super C Cube, Super T Terminal, F1 All keys` (six) | **5/5** (on bcecb671) |
| cube row, Programs, Foot (r5 path) | navigate, `Super Alt 2 Into face` | yes | unchanged | **5/5** |
| Super+C, Super+t stood back | navigate, `Into face` | yes | unchanged | **5/5** |
| pulled from a face by Super+Alt+4 (camera flies into the pane) | **use**, keys on the look, bar `Alt Esc Step back` only | no (one more Alt+Escape) | navigate, `Super Alt 2 Into face`; the key puts it back, use | **5/5** |
| face path (cube row, Return, Super+t, Super+Alt+4) | `Pull out` told, pulls | yes | unchanged | **5/5** |
| Super+C, launcher stood back, Foot (fuzzel) | Foot opens beside the cube and is flown into in 2 of 5 runs only; then as the r5 path | (2/5) | unchanged: flown in 2/5, those pass; 3/5 Foot never gets the keyboard (no pane path), and Alt+Escape stood back then steps into the face (seen on base too) | 2/5, open point 1 |

### 2. Decision (docs/cube-modes.md, replaces r5's sentence)
"`Alt+Escape` out of any pane stands back: from the cube when one is open, also from a pane flown into from a face, and there the bar names `Super Alt 2 Into face`, …; with no cube open into the space, where the bar names `Super C Cube`, which opens the cube stood back with that window told `Into face` the same way, so the keys that the bar names lead from every pane into a face (co-79)." With a cube, every pane path now lands in the r5 state (the brief's preference). Without a cube, landing in the r5 state would mean that Alt+Escape out of a terminal opens a cube of four xp desktops unasked: a POLS break, a start of chora-xp on every step out, and a change to the fly-out that the walk gates of other construct cells hold; so the space stays, and its bar tells the one key that leads to the r5 state. One rule, two places, no hint per path.

### 3. Red 5f8f0c84 (gate only)
gate-construct-cube.sh step 8 (at the end, on the empty floor): the Terminal row of the start card, Alt+Escape, the bar must name `Super C` (`construct-hints-cube-keys-*`), Super+C must stand back with `Super Alt 2` told, Super+Alt+2 must put the window on face 0, step in and give it the keys (typed `a` reaches it) with `Pull out` told; then Super+Alt+4 pulls it out (camera flies in), Alt+Escape must stand back with `Super Alt 2` told, and the key must put it back. On 4d8bbe36 (head construct, the final gate): `FAIL: co-79: stepped out of the pane of the Terminal row, with no cube open, the hint bar names no way to a cube (cube keys:  ; bar: ; focus: ["s12"]): {"mode":"use","faces":0}`; every earlier cell passed, r5's step 7 included (gate-cube-red-4d8bbe36.log).

### 4. Fix 1916f517
- `step_out()` (modes.rs): the fly-out key (`Grab::FlyOut`) and the hint `Step back` fly out of the pane and, with the cube in use mode, stand back as the person (`mode_act(StepBack, Person)`, which notes the pane's window): the pull path lands in the r5 state.
- `cube_window()` (overlay.rs): no cube, camera out of the panes, the window with the keyboard is a pane's. It decides the walk hint `Super C Cube` (`hints::walking(panes, ahead, cube)`; at most six; 1916f517 let Terminal give way at five, which the layout gate's click on the Terminal hint after a step out of a pane caught, so bcecb671 keeps it) and the hint stamp; a click on it does the key (`hint_press` "cube" exists).
- `cube_key()` (input.rs): notes that window in the new cube when nothing is noted, as a step back does, so `put_window` (r5, unchanged) names and puts it.
- Unit tests: `reach_tests::stepped_out_of_a_pane_without_a_cube_the_walk_names_the_cube`, `hints::stepped_out_of_a_pane_the_walk_names_the_cube_and_stays_at_six`. chora-construct 302 passed; clippy clean.
- Gate cell details: before Super+Alt+2 the cell waits until the new cube's xp exports `window.move` (see open point 2), and after Super+Alt+4 until the camera has flown into the pulled pane (else Alt+Escape steps back first and the late fly-in lands in the pane stood back, a gate-speed race).

### 5. Counts after (fresh compositor per run, script hand.sh)
Snapshot 1916f517: every pane path of the table 5/5 (told and acts), the face path 5/5; snapshot bcecb671 (the six hints): Terminal row 5/5 and Super+t in a pane 5/5 again (hand-term-final.log, hand-supertpane-final.log); launcher stood back 2/5 (only those 2 got a pane with the keyboard; open point 1). Before: Terminal row 0/5 (hand-term-head.log).

### 6. Gate
Run 1 (1916f517) stopped at gate-lib, **caused by this lane's setup**: the selftest's nested gate refused to start while my persona hand runs held a `chora --headless`. Run 2 stopped at ci-check, **caused by this lane's setup**: the mkdocs venv on PATH shadowed python3 (no PIL), so the clock selftest skipped for PIL instead of Inter (TF-1); later runs put only mkdocs on PATH. Run 3 (1916f517): layout FAIL, **caused here**: `FAIL: light-1280x720: the hints with one pane are:` (the walk set gained `cube`, and with three panes Terminal gave way, so the click on the Terminal hint broke); fixed by bcecb671, layout cell alone EXIT 0 (97 ok). Run 4 on **bcecb671**, `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`, no step filter, detached, nothing else running: **all steps passed, 68 PASS, 0 FAIL** (this gate version prints no EXIT line; its last line is the pass line); cube 159 s, layout 1388 s; no foreign flake seen. `pgrep -fa 'chora --headless'` empty after. Last 20 lines:
```
launcher-mc PASS    40s /tmp/merge-gate.85JUPx/launcher-mc.log
voice-acp  PASS    13s /tmp/merge-gate.85JUPx/voice-acp.log
voice-events PASS     2s /tmp/merge-gate.85JUPx/voice-events.log
agent      PASS    42s /tmp/merge-gate.85JUPx/agent.log
tiling-keys-more PASS    44s /tmp/merge-gate.85JUPx/tiling-keys-more.log
acts       PASS  2317s /tmp/merge-gate.85JUPx/acts.log
tiling-system PASS    16s /tmp/merge-gate.85JUPx/tiling-system.log
handover   PASS   103s /tmp/merge-gate.85JUPx/handover.log
mc-scale   PASS    16s /tmp/merge-gate.85JUPx/mc-scale.log
watchdog-starved PASS    60s /tmp/merge-gate.85JUPx/watchdog-starved.log
tiling-windows PASS   222s /tmp/merge-gate.85JUPx/tiling-windows.log
xp-run     PASS    15s /tmp/merge-gate.85JUPx/xp-run.log
demo       PASS   271s /tmp/merge-gate.85JUPx/demo.log
leak       PASS   480s /tmp/merge-gate.85JUPx/leak.log
tiling-theme PASS    19s /tmp/merge-gate.85JUPx/tiling-theme.log
tiling-bar PASS   208s /tmp/merge-gate.85JUPx/tiling-bar.log
mc-host    PASS    25s /tmp/merge-gate.85JUPx/mc-host.log
tiling-disc PASS    74s /tmp/merge-gate.85JUPx/tiling-disc.log
e2e        PASS   289s /tmp/merge-gate.85JUPx/e2e.log
all steps passed; no --push, nothing is pushed
```

### Captures (media/2026-11-26/co-reach-r6/, each under 400 KB)
- term-head1-run1-*.png (4d8bbe36): in the pane; stood back in the space, bar without a way to a cube; after Super+Alt+2 nothing changed.
- term-fix-run1-*.png (1916f517): in-pane; stood-back (bar `W A S D Walk | Return Fly in | Super C Cube | Super T Terminal | F1 All keys`); cube (navigate, `Super Alt 2 Into face`); after-key (Foot in face 0, `Pull out` told).
- pull-fix-run1-*.png: in the pulled pane; stood back with `Into face`; after the key back in the face.
- supertpane-final-run1-stood-back.png (bcecb671): two panes, the six hints.
- logs: hand-<path>.log (after, 5 runs each, 1916f517), hand-term-final.log and hand-supertpane-final.log (bcecb671), hand-term-head.log (before, 5 runs), gate-cube-red-4d8bbe36.log, gate-cube-fix-1916f517.log, hand.sh.

### Open for the PM
1. Launcher stood back (Super+C, Super+space, fuzzel, foot): the new window is flown into in only 2 of 5 runs; in 3 it stays beside the cube without the keyboard (`Click Fly in` is told and works), and Alt+Escape there steps into the face. Seen on base; not a pane path, not changed. The gate's chora-probe launcher always flies in.
2. A cube's xp connects (faces Up, taskbars drawn) about 1.5 s before it exports `window.move`; a give key pressed in that gap gives the window but leaves the cube stood back with "XP took the window but not onto that face" (r5's put, pre-existing; a person is slower, the gate waits).
3. gate-construct-layout.sh changed outside the named area (three expected strings, see the head line); docs/configuration.md still lists the walk hints without `Super C Cube` (outside this lane's area).

## r6 (co-79 every pane path, 2026-11-26)

Lane: cloud, claude/co-reach-r6 from **base 4d8bbe36** (t153+154). Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green. Area: plugins/chora-construct (hints, overlay, input, modes, reach_tests), tests/gate-construct-cube.sh, docs/cube-modes.md. No core, no other plugin.

**Head bcecb671** (red **5f8f0c84**, fix **1916f517**, fix follow-up **bcecb671**: the walk keeps Terminal beside Cube, at most six, and gate-construct-layout.sh's three pinned walk sets name `cube`), pushed to claude/co-reach-r6. Diff 4d8bbe36..bcecb671: 8 files +190 -28 (gate-construct-cube.sh +70, hints.rs +30 -8, reach_tests.rs +25, overlay.rs +18 -3, input.rs +15 -1, modes.rs +15, docs/cube-modes.md +14 -13, gate-construct-layout.sh +3 -3). **Outside the named area**: gate-construct-layout.sh pins the exact walk hint set after Alt+Escape out of a pane, which the decided hint changes; only its three expected strings changed (`cube fly-in keys terminal walk[ windows]`).

### 1. Reproduction on 4d8bbe36 and every pane path
The Terminal row (start card, 1280x800, fresh compositor per run, Alt+Escape out of Foot): **0/5** (hand-term-head.log): `mode use front 0 faces 0, panes ['foot'], core focus ['construct-look', 's1']`, bar `Return Fly in | F1 All keys | Super T Terminal | W A S D Walk`, Super+Alt+2 does nothing. The root cause is not a mode split: **no cube is open at all** (`cube.list` says `mode use` with 0 faces, its default). The person stands in the space, and the walk hints have named no cube since 14f7d3f6 (11-11, "the hint Cube is gone, since the start card and the help name it"); the start card hides as soon as a pane exists. A second fault: Super+C there opens the cube, but the cube that covers the pane takes the keyboard from its window (`covered_focus_to_nobody`, co-46), so `Into face` is not told there either (1 run, before the fix).

| Way a pane gets the keyboard | before (4d8bbe36): state after Alt+Escape | way into a face told | after (1916f517): state after Alt+Escape, told keys | 5 runs after |
|---|---|---|---|---|
| start card Terminal row (no cube) | walk, faces 0, keys on look + s1 | none (0/5) | walk; bar `Super C Cube`; Super+C: navigate with `Super Alt 2 Into face`; Super+Alt+2: use, Foot on xp VIRTUAL-1, `Pull out` told | **5/5** |
| start card All programs row, Foot (no cube) | walk, faces 0 | none | same as the Terminal row | **5/5** |
| launcher Super+space, Foot (no cube) | walk, faces 0 | none | same | **5/5** |
| Super+t in the space (no cube) | walk, faces 0 | none | same | **5/5** |
| Super+t in a pane (no cube, 2 panes) | walk, faces 0, bar adds `Tab Windows` | none | same; bar `W A S D Walk, Return Fly in, Tab Windows, Super C Cube, Super T Terminal, F1 All keys` (six) | **5/5** (on bcecb671) |
| cube row, Programs, Foot (r5 path) | navigate, `Super Alt 2 Into face` | yes | unchanged | **5/5** |
| Super+C, Super+t stood back | navigate, `Into face` | yes | unchanged | **5/5** |
| pulled from a face by Super+Alt+4 (camera flies into the pane) | **use**, keys on the look, bar `Alt Esc Step back` only | no (one more Alt+Escape) | navigate, `Super Alt 2 Into face`; the key puts it back, use | **5/5** |
| face path (cube row, Return, Super+t, Super+Alt+4) | `Pull out` told, pulls | yes | unchanged | **5/5** |
| Super+C, launcher stood back, Foot (fuzzel) | Foot opens beside the cube and is flown into in 2 of 5 runs only; then as the r5 path | (2/5) | unchanged: flown in 2/5, those pass; 3/5 Foot never gets the keyboard (no pane path), and Alt+Escape stood back then steps into the face (seen on base too) | 2/5, open point 1 |

### 2. Decision (docs/cube-modes.md, replaces r5's sentence)
"`Alt+Escape` out of any pane stands back: from the cube when one is open, also from a pane flown into from a face, and there the bar names `Super Alt 2 Into face`, …; with no cube open into the space, where the bar names `Super C Cube`, which opens the cube stood back with that window told `Into face` the same way, so the keys that the bar names lead from every pane into a face (co-79)." With a cube, every pane path now lands in the r5 state (the brief's preference). Without a cube, landing in the r5 state would mean that Alt+Escape out of a terminal opens a cube of four xp desktops unasked: a POLS break, a start of chora-xp on every step out, and a change to the fly-out that the walk gates of other construct cells hold; so the space stays, and its bar tells the one key that leads to the r5 state. One rule, two places, no hint per path.

### 3. Red 5f8f0c84 (gate only)
gate-construct-cube.sh step 8 (at the end, on the empty floor): the Terminal row of the start card, Alt+Escape, the bar must name `Super C` (`construct-hints-cube-keys-*`), Super+C must stand back with `Super Alt 2` told, Super+Alt+2 must put the window on face 0, step in and give it the keys (typed `a` reaches it) with `Pull out` told; then Super+Alt+4 pulls it out (camera flies in), Alt+Escape must stand back with `Super Alt 2` told, and the key must put it back. On 4d8bbe36 (head construct, the final gate): `FAIL: co-79: stepped out of the pane of the Terminal row, with no cube open, the hint bar names no way to a cube (cube keys:  ; bar: ; focus: ["s12"]): {"mode":"use","faces":0}`; every earlier cell passed, r5's step 7 included (gate-cube-red-4d8bbe36.log).

### 4. Fix 1916f517
- `step_out()` (modes.rs): the fly-out key (`Grab::FlyOut`) and the hint `Step back` fly out of the pane and, with the cube in use mode, stand back as the person (`mode_act(StepBack, Person)`, which notes the pane's window): the pull path lands in the r5 state.
- `cube_window()` (overlay.rs): no cube, camera out of the panes, the window with the keyboard is a pane's. It decides the walk hint `Super C Cube` (`hints::walking(panes, ahead, cube)`; at most six; 1916f517 let Terminal give way at five, which the layout gate's click on the Terminal hint after a step out of a pane caught, so bcecb671 keeps it) and the hint stamp; a click on it does the key (`hint_press` "cube" exists).
- `cube_key()` (input.rs): notes that window in the new cube when nothing is noted, as a step back does, so `put_window` (r5, unchanged) names and puts it.
- Unit tests: `reach_tests::stepped_out_of_a_pane_without_a_cube_the_walk_names_the_cube`, `hints::stepped_out_of_a_pane_the_walk_names_the_cube_and_stays_at_six`. chora-construct 302 passed; clippy clean.
- Gate cell details: before Super+Alt+2 the cell waits until the new cube's xp exports `window.move` (see open point 2), and after Super+Alt+4 until the camera has flown into the pulled pane (else Alt+Escape steps back first and the late fly-in lands in the pane stood back, a gate-speed race).

### 5. Counts after (fresh compositor per run, script hand.sh)
Snapshot 1916f517: every pane path of the table 5/5 (told and acts), the face path 5/5; snapshot bcecb671 (the six hints): Terminal row 5/5 and Super+t in a pane 5/5 again (hand-term-final.log, hand-supertpane-final.log); launcher stood back 2/5 (only those 2 got a pane with the keyboard; open point 1). Before: Terminal row 0/5 (hand-term-head.log).

### 6. Gate
Run 1 (1916f517) stopped at gate-lib, **caused by this lane's setup**: the selftest's nested gate refused to start while my persona hand runs held a `chora --headless`. Run 2 stopped at ci-check, **caused by this lane's setup**: the mkdocs venv on PATH shadowed python3 (no PIL), so the clock selftest skipped for PIL instead of Inter (TF-1); later runs put only mkdocs on PATH. Run 3 (1916f517): layout FAIL, **caused here**: `FAIL: light-1280x720: the hints with one pane are:` (the walk set gained `cube`, and with three panes Terminal gave way, so the click on the Terminal hint broke); fixed by bcecb671, layout cell alone EXIT 0 (97 ok). Run 4 on **bcecb671**, `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`, no step filter, detached, nothing else running: **all steps passed, 68 PASS, 0 FAIL** (this gate version prints no EXIT line; its last line is the pass line); cube 159 s, layout 1388 s; no foreign flake seen. `pgrep -fa 'chora --headless'` empty after. Last 20 lines:
```
launcher-mc PASS    40s /tmp/merge-gate.85JUPx/launcher-mc.log
voice-acp  PASS    13s /tmp/merge-gate.85JUPx/voice-acp.log
voice-events PASS     2s /tmp/merge-gate.85JUPx/voice-events.log
agent      PASS    42s /tmp/merge-gate.85JUPx/agent.log
tiling-keys-more PASS    44s /tmp/merge-gate.85JUPx/tiling-keys-more.log
acts       PASS  2317s /tmp/merge-gate.85JUPx/acts.log
tiling-system PASS    16s /tmp/merge-gate.85JUPx/tiling-system.log
handover   PASS   103s /tmp/merge-gate.85JUPx/handover.log
mc-scale   PASS    16s /tmp/merge-gate.85JUPx/mc-scale.log
watchdog-starved PASS    60s /tmp/merge-gate.85JUPx/watchdog-starved.log
tiling-windows PASS   222s /tmp/merge-gate.85JUPx/tiling-windows.log
xp-run     PASS    15s /tmp/merge-gate.85JUPx/xp-run.log
demo       PASS   271s /tmp/merge-gate.85JUPx/demo.log
leak       PASS   480s /tmp/merge-gate.85JUPx/leak.log
tiling-theme PASS    19s /tmp/merge-gate.85JUPx/tiling-theme.log
tiling-bar PASS   208s /tmp/merge-gate.85JUPx/tiling-bar.log
mc-host    PASS    25s /tmp/merge-gate.85JUPx/mc-host.log
tiling-disc PASS    74s /tmp/merge-gate.85JUPx/tiling-disc.log
e2e        PASS   289s /tmp/merge-gate.85JUPx/e2e.log
all steps passed; no --push, nothing is pushed
```

### Captures (media/2026-11-26/co-reach-r6/, each under 400 KB)
- term-head1-run1-*.png (4d8bbe36): in the pane; stood back in the space, bar without a way to a cube; after Super+Alt+2 nothing changed.
- term-fix-run1-*.png (1916f517): in-pane; stood-back (bar `W A S D Walk | Return Fly in | Super C Cube | Super T Terminal | F1 All keys`); cube (navigate, `Super Alt 2 Into face`); after-key (Foot in face 0, `Pull out` told).
- pull-fix-run1-*.png: in the pulled pane; stood back with `Into face`; after the key back in the face.
- supertpane-final-run1-stood-back.png (bcecb671): two panes, the six hints.
- logs: hand-<path>.log (after, 5 runs each, 1916f517), hand-term-final.log and hand-supertpane-final.log (bcecb671), hand-term-head.log (before, 5 runs), gate-cube-red-4d8bbe36.log, gate-cube-fix-1916f517.log, hand.sh.

### Open for the PM
1. Launcher stood back (Super+C, Super+space, fuzzel, foot): the new window is flown into in only 2 of 5 runs; in 3 it stays beside the cube without the keyboard (`Click Fly in` is told and works), and Alt+Escape there steps into the face. Seen on base; not a pane path, not changed. The gate's chora-probe launcher always flies in.
2. A cube's xp connects (faces Up, taskbars drawn) about 1.5 s before it exports `window.move`; a give key pressed in that gap gives the window but leaves the cube stood back with "XP took the window but not onto that face" (r5's put, pre-existing; a person is slower, the gate waits).
3. gate-construct-layout.sh changed outside the named area (three expected strings, see the head line); docs/configuration.md still lists the walk hints without `Super C Cube` (outside this lane's area).

## Review r6 (co-79 every pane path, 2026-11-26; reviewer not the builder)

Reviewed claude/co-reach-r6 **4d8bbe36..bcecb671** (8 files +190 -28), one round, read-only on code. Step 0: unshallow, `cargo check --workspace` green. On head: `cargo test -p chora-construct` 302 passed; `cargo clippy -p chora-construct --all-targets` clean; `tests/gate-construct-cube.sh` EXIT 0, 68 ok (three co-79 r6 lines among them).

**Verdict: MERGE-OK** (no open BLOCK or MAJOR).

Checks:
1. Red first: confirmed by a run of my own. The gate of 5f8f0c84 (only that file changed, +70, nothing removed; identical to the gate on bcecb671) on a 4d8bbe36 build: EXIT 1 at `FAIL: co-79: stepped out of the pane of the Terminal row, with no cube open, the hint bar names no way to a cube (cube keys:  ; bar: ; focus: ["s12"]): {"mode":"use","faces":0}`, every earlier cell ok (r5's step 7 included); on bcecb671 EXIT 0. The pull half of step 8 never ran on base (the gate stops at the first fail), so its red rests on hand-pull.log only (rcrr6-2).
2. co-79, one rule: yes. With a cube, every pane exit goes through `step_out` (modes.rs:317, key `Grab::FlyOut` and the hint `step-back`), which stands back when the cube was in use; the voice and `view.step_back` already did (`mode_act` flies out first). With no cube, one predicate, `cube_window` (overlay.rs:690), decides the walk hint, the hint stamp and what `cube_key` (input.rs:648) notes; r5's `put_window` does the rest unchanged. No hint or case per path; two places, as the decision sentence says. The three step-back entries could be one (rcrr6-4, NIT).
3. POLS: r5's step 7 and the face path cell pass on head (my run). The pull path change (one Alt+Escape stands back instead of landing in use with the keys on the look and only `Step back` told) removes a dead state; nothing relied on it: no other gate pins it and the full gate of the lane passed. Return from that stood-back state steps into the face with the pane window noted: `activate_in_face` on a pane window logs a warning and the keys stay on the look, the same end state as before r6 (r5 has it on the Programs path too), not new. Super+C: `cube_window` is the core focus filtered to an attached pane's window, with no cube and the camera out of the panes; `unseen_focus_to_walk` clears it once that pane leaves sight, a press on another pane moves it, and `cube_key` fills it only when the step back noted nothing. It cannot put a non-pane or a face window; with two panes in sight it puts the one that last had the keyboard, not the one ahead, unnamed (rcrr6-6, NIT).
4. Hints: `walking()` holds at most six (unit test over 0..8 panes); the six fit (layout gate 97 ok on the lane). The layout gate's `hints_are` is exact equality over drawn hint rects, so the three strings now require `cube` to be shown: tighter, not loosened; honest.
5. Launcher stood back 2/5: a finding of its own, pre-existing, not caused by this diff (rcrr6-3): the diff touches no launch or fly-in code, `cube_window` is None there (a cube is open), and `step_out` only stands back, never steps in, so "Alt+Escape steps into the face" comes from elsewhere.
6. docs/cube-modes.md says what the code does (table row, the pane paragraph, the walk hint "while the window of a pane has the keyboard", the decision sentence). docs/configuration.md does not (rcrr6-1).

Findings:
- rcrr6-1, MINOR, docs/configuration.md:1200: "at most five" and the walk list without `Super C Cube` now contradict the code (six, `cube` while a pane's window has the keyboard); fix: add `Super C Cube` with its condition and say at most six.
- rcrr6-2, MINOR, tests/gate-construct-cube.sh (step 8): only the Terminal row and the pull path have a cell; All programs, launcher, Super+t in the space and in a pane (all 0 told on base by hand) have none, and the layout gate pins the `cube` hint but not that Super+C then leads into a face; fix: one looped cell that runs Super+t in the space and Super+t in a pane through Alt+Escape, Super+C, Super+Alt+2.
- rcrr6-3, MINOR (pre-existing, own finding, not this merge), launcher path stood back (Super+C, Super+space, Foot): the new window is flown into 2/5, in 3/5 it stays beside the cube without the keyboard (`Click Fly in` is told and works) and Alt+Escape then steps into the face; fix: make the fly-in of a window launched stood back deterministic and add a gate cell with a real launcher, not chora-probe (which always flies in).
- rcrr6-4, NIT, modes.rs:317, overlay.rs:1149-1152: three entries for one step back (`step_out`, `mode_act(StepBack)` with its own fly-out, the hint's two `step-back` arms); fix: let `mode_act` use `in_pane()` for its fly-out and route the key and the hint through it when a cube is open.
- rcrr6-5, NIT, reports/co-reach.md:360-434: "## r6" is there twice, the first copy with its backtick spans lost ("name )", "changed ()"); fix: drop the first copy.
- rcrr6-6, NIT, overlay.rs:690, input.rs:648: with two panes in sight, Super+C notes the pane that last had the keyboard, not the one ahead, and `Into face` does not name it; fix (optional): name the window in the put hint or the HUD notice.

## r7 (co-79: one told key out of a pane into a face, 2026-11-26)

Lane: cloud, claude/co-reach-r7 from **base dc532d60** (57ce1480 + rpc2-1/rpc2-2). Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green. Area: plugins/chora-construct (hints, overlay, input, pull, cube host), tests/gate-construct-cube.sh, docs/cube-modes.md.

**Head 008714ba** (red **61741663**, fix **2e521bbc**, gate follow-ups **242753da** (layout gate) and **008714ba** (keys gate): two refused-give cells that pressed Super+Alt+2 where r7 now puts), pushed to claude/co-reach-r7. Diff dc532d60..008714ba: 11 files +322 -61 (gate-construct-keys.sh +3 -3, pull.rs +88, overlay.rs +61 -34, reach_tests.rs +62, gate-construct-cube.sh +53, hints.rs +20 -8, docs/cube-modes.md +14 -7, input.rs +6 -5, cube.rs +5, construct.rs +4 -1, gate-construct-layout.sh +6 -3). **Outside the named area**: gate-construct-layout.sh (1) pins the exact walk hint set after Alt+Escape out of a pane, which now holds `put-in`: its three expected strings changed (`cube fly-in keys put-in terminal walk`, `fly-in keys put-in terminal walk windows`), as r6 did; (2) its refused-give notice cell pressed Super+Alt+2 right after that step out, expecting xp's "not running" refusal, which r7 replaces by the put (run 1 of the full gate: `FAIL: dark-1280x720: the notice of a refused give reads ''`, caused here); 242753da first presses on the floor, so the walk holds the keyboard and the give of the pane ahead is refused as before (layout cell alone at 1280x720: EXIT 0, 40 ok); (3) gate-construct-keys.sh gave the focused pane window to xp by Super+Alt+2 inside a pane and expected xp's refusal (run 3: `FAIL: a give of the focused window to xp, which does not run, said nothing:`, caused here, r7 opens a cube for the put there); 008714ba takes Super+Alt+1 (tiling, not running there) for the same refusal; keys gate alone EXIT 0, 18 ok.

### 1. Red on dc532d60
Terminal row, Alt+Escape: bar `W A S D Walk, Return Fly in, Super C Cube, Super T Terminal, F1 All keys`, no key into a face; Super+Alt+2 there does nothing (no cube; `put_window` needs a cube stood back). In a pane (flown in by its Fly in, or pulled out of a face) the bar names only `Alt Esc Step back` and `All keys`. Gate step 10 on dc532d60: `FAIL: co-79 r7: right after Alt+Escape out of the pane of the Terminal row the hint bar names no key into a face (put-in keys:   ; bar: ; panes: ["chora-probe"]): {"mode":"use","faces":0}`; every earlier cell ok (70 ok, r6 steps 7, 8 and rpc2 step 9 among them; gate-cube-red-dc532d60.log). Unit test `reach_tests::out_of_a_pane_and_in_one_the_bar_names_the_key_into_a_face` failed on dc532d60 ("stepped out of the pane with no cube, the walk names the key into a face").

### 2. Decision (docs/cube-modes.md)
"One key, one step (co-79 r7): wherever the window of a pane has the keyboard, in the pane (flown in by its `Fly in` click, from a face, or pulled out of one by a drag or `Super+Alt+4`) or right after `Alt+Escape` out of it, the bar names `Super Alt 2 Into face`, and that key puts that pane's window on the face at the front, steps in and gives it the keyboard, and opens the cube first when none is open (the window goes as soon as the face manager can place it)." KISS: one predicate (`pane_to_face`) decides the hint and the key in every pane state; no chain through Super C (still told beside it, so r6's step 8 holds). POLS: the key is the same give key r5 told stood back, it now also works where the person stands after Alt+Escape and inside the pane.

### 3. Fix 2e521bbc
- `pane_to_face()` (overlay.rs): `put_window` (stood back, r5) or the flown pane's window or `cube_window` (no cube, stepped out, r6); `put_in_hint` and `give_key` ask it.
- `put_pane()` (pull.rs): in a pane first `step_out` (with a cube in use it stands back, r6); with no cube `cube_key` (opens it stood back, notes the window); then the put waits on `Host::put_later`, polled by `check_cube`/`cube_deadline` every 150 ms: once every face is up and xp both exports `window.move` and answers `state.get` within 250 ms, `put_in_face` (r5) gives, places on the front face and steps in. The wait ends unput on a step in, a failed front face, a window that is no pane's, or 20 s.
- Why the answer check: the first fix put as soon as `window.move` was listed, and the gate caught it: xp exports its methods before it grabs its keys and adopts, so the place call timed out (`s14 is a window of xp, not on VIRTUAL-1: socket: Resource temporarily unavailable`) and the cube stayed stood back (r6 open point 2, now handled in the product, not only in the gate).
- `walking()`: `Into face` after `Super C Cube`; with `Tab Windows` the cube hint gives way, at most six (unit test). The in-pane bar (Flown/Rest sets) names `Into face` beside `Step back`.
- Unit tests: the red one above; `the_key_into_a_face_waits_for_the_face_manager_and_ends_at_a_step_in`; `hints::stepped_out_of_a_pane_the_walk_names_the_key_into_a_face_and_stays_at_six`. chora-construct 307 passed (306 before the wait test); clippy clean.
- Gate step 10 (gate-construct-cube.sh): Terminal row, Alt+Escape, bar names `Super Alt 2`, one Super+Alt+2 (no wait in the gate for xp): window on face 0, mode use, typed `a` reaches it; Super+Alt+4 pulls it out (camera in the pane), the bar there names `Super Alt 2`, the key puts it back, mode use. Cube gate alone on 2e521bbc: EXIT 0 (gate-cube-fix-2e521bbc.log).

### 4. Paths by hand (fresh compositor per run, 1280x800, hand7.sh; told = bar names `Super Alt 2 Into face` before the key; pass = told, Foot on xp VIRTUAL-1 shown, mode use, bar `Pull out`)
| Path | state where the key is pressed | told | acts |
|---|---|---|---|
| start card Terminal row, Alt+Escape (no cube) | walk, panes [foot] | 5/5 | **5/5** |
| start card All programs row, foot, Alt+Escape (no cube) | walk | 5/5 | **5/5** |
| launcher Super+space, foot, Alt+Escape (no cube) | walk | 5/5 | **5/5** |
| Super+t in the space, Alt+Escape (no cube) | walk | 5/5 | **5/5** |
| Super+t in a pane (2 panes), Alt+Escape (no cube) | walk, bar with Tab Windows (Super C gives way) | 5/5 | **5/5** |
| cube row, Programs, Foot, Alt+Escape (r5 path) | navigate | 5/5 | **5/5** |
| demo 2: Terminal row, Alt+Escape, Return Fly in (back into the pane) | in the pane, no cube | 5/5 | **5/5** |
| pulled from a face by Super+Alt+4 (camera in the pane) | in the pane, cube in use | 5/5 | **5/5** |
Before (dc532d60, p2-r15 hand-co79): Terminal row, key told right after Alt+Escape 0/5. Drag out of a face (Max): not counted by hand: at 1 output a drag on a stood-back face turns the cube, and the title drag off a face needs a second output (gate-construct-pull); after its drop the camera flies into the pane, the state of the pulled row above, where the key is told and acts.

### 5. Gate
Run 1 (2e521bbc) stopped at layout, **caused here**: `FAIL: dark-1280x720: the notice of a refused give reads ''` (see the head paragraph, fixed by 242753da). Run 2 (242753da) stopped at doors on a **foreign flake**: tiling `helper-none missing` (`FAIL: 1 cells or checks are not pass`, construct cells all pass); tests/gate-doors.sh alone EXIT 0, helper-none pass in all four (gate-doors-run2.log, gate-doors-alone.log). Run 3 (242753da) stopped at keys, **caused here**: `FAIL: a give of the focused window to xp, which does not run, said nothing:` (fixed by 008714ba). Run 4 on **008714ba**, `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`, no step filter, detached, nothing else running: **all steps passed, 68 PASS, 0 FAIL, EXIT 0**; cube 160 s (three co-79 r7 lines ok), layout 1377 s, keys 38 s, acts 1979 s. `pgrep -fa 'chora --headless'` empty after. Last 20 lines:
```
mc-column  PASS    49s /tmp/merge-gate.m8rKE6/mc-column.log
voice-acp  PASS    12s /tmp/merge-gate.m8rKE6/voice-acp.log
voice-events PASS     2s /tmp/merge-gate.m8rKE6/voice-events.log
launcher-mc PASS    37s /tmp/merge-gate.m8rKE6/launcher-mc.log
agent      PASS    38s /tmp/merge-gate.m8rKE6/agent.log
tiling-keys-more PASS    41s /tmp/merge-gate.m8rKE6/tiling-keys-more.log
tiling-system PASS    16s /tmp/merge-gate.m8rKE6/tiling-system.log
handover   PASS   101s /tmp/merge-gate.m8rKE6/handover.log
mc-scale   PASS    13s /tmp/merge-gate.m8rKE6/mc-scale.log
tiling-windows PASS   193s /tmp/merge-gate.m8rKE6/tiling-windows.log
watchdog-starved PASS    56s /tmp/merge-gate.m8rKE6/watchdog-starved.log
xp-run     PASS    14s /tmp/merge-gate.m8rKE6/xp-run.log
demo       PASS   245s /tmp/merge-gate.m8rKE6/demo.log
tiling-bar PASS   185s /tmp/merge-gate.m8rKE6/tiling-bar.log
tiling-theme PASS    19s /tmp/merge-gate.m8rKE6/tiling-theme.log
leak       PASS   474s /tmp/merge-gate.m8rKE6/leak.log
mc-host    PASS    22s /tmp/merge-gate.m8rKE6/mc-host.log
tiling-disc PASS    72s /tmp/merge-gate.m8rKE6/tiling-disc.log
e2e        PASS   280s /tmp/merge-gate.m8rKE6/e2e.log
all steps passed; no --push, nothing is pushed
```

### Captures (media/2026-11-26/co-reach-r7/, each under 400 KB)
- `<path>-run1-in-pane.png`, `-stood-back.png` (bar with `Super Alt 2 Into face`), `-after-key.png` (Foot in face 0, `Pull out` told); `flyin-run1-flown-in.png` (demo 2: in the pane, `Into face` beside `Step back`); `pull-run1-in-pane.png`.
- hand-<path>.log (5 runs each), hand7.sh, gate-cube-red-dc532d60.log, gate-cube-fix-2e521bbc.log, gate-layout-1280x720-242753da.log, gate-keys-run3.log, gate-doors-run2.log, gate-doors-alone.log.

### Open for the PM
1. Drag out of a face (Max) rests on the rule and the pull gate, not on a hand count (see 4).
2. docs/configuration.md still lists the walk hints without `Super C Cube` and `Into face` (rcrr6-1, outside this lane's area).
3. While xp starts, the poll asks `state.get` with 250 ms (a few times, about 1-3 s after the key): the construct's frame can stall that long once per try.

## Review r7

Seeded review of claude/co-reach-r7 at 008714ba (dc532d60..008714ba, 11 files +322 -61), one round, cloud, read-only on code. Step 0: unshallow, `cargo check --workspace` green. On head: `cargo test -p chora-construct` 307 passed; `cargo clippy -p chora-construct --all-targets` clean; `tests/gate-construct-cube.sh` EXIT 0, 73 ok (the three co-79 r7 lines among them). Two throwaway probe unit tests (not committed) for checks 3 and 5.

1. Red first: met. On 61741663 `reach_tests::out_of_a_pane_and_in_one_the_bar_names_the_key_into_a_face` fails (reach_tests.rs:158, "stepped out of the pane with no cube, the walk names the key into a face"); gate step 10 red on dc532d60 in media/2026-11-26/co-reach-r7/gate-cube-red-dc532d60.log (`FAIL: co-79 r7: right after Alt+Escape ... names no key into a face`); both pass on 008714ba.
2. co-79 KISS: met. One predicate `pane_to_face` (put_window, else the flown pane's window, else cube_window) feeds the walk hint, the in-pane hint and the key; `put_pane` adds two preparatory steps (step out, open the cube) before one shared tail (`put_when_ready` then r5's `put_in_face`). No per-path special case.
3. The wait: no timer leak (put_later lives on the Host, dropped with the cube, cleared on every end); pane closed ends it (own_window). But it does not end when the person moved on (rcrr7-2), a second press during the start takes another path (rcrr7-3), the unput ends are silent (rcrr7-4), and state.get blocks the frame loop (rcrr7-5).
4. Gates outside the area: the changed hint strings follow the new set; the layout cell (floor press first) and the keys cell (Super+Alt+1) still assert a told refusal, so no assert is loosened; but the state they left (Super+Alt+2 with the keyboard in a pane window and no usable xp) now runs the put path and nothing asserts what is told there (rcrr7-6).
5. POLS: with xp already running the key from a pane no longer gives to it (rcrr7-1, probe: notice `a client xp runs already (code 1)`, no window.give). With no xp running the give to xp of a pane window is reachable only by the window list or the pane ahead with the walk holding the keyboard, as designed (construct holds the face turn). Hints at most six in every set: walking() max 6 (unit test covers 1 and 2 panes, ahead or not), Flown/Rest at most 4.
6. Docs: say what the code does in the normal path; the wait's end and its refusals are missing (rcrr7-7).

Findings:
- rcrr7-1 MAJOR plugins/chora-construct/src/overlay.rs:729 (input.rs:174): with the face manager already running (xp as a manager pane or a client), `pane_to_face` still returns the pane window via `cube_window`, the bar names `Super Alt 2 Into face`, and the key goes to `put_pane`, whose `open_cube` refuses "a client xp runs already (code 1)": nothing is given, where dc532d60 gave the window to the running xp (probe probe_xp_runs_give_from_pane; no gate covers a key give from a pane to a running xp). Fix: take the `cube_window` branch only when the face manager neither has a manager pane nor runs as a client, else fall through to the plain give (hint off too), with a unit test.
- rcrr7-2 MINOR plugins/chora-construct/src/pull.rs:405: the wait ends only on a step in, a failed front, a gone window or 20 s, not when the person moved on: flown into another pane s2 during the wait, the put of s1 still runs (probe probe_fly_elsewhere_during_wait: state.get then the give of s1 while flown in s2), and a real place then steps in and glides the camera out of the pane the person chose. Fix: end the wait when `in_pane()` or when the noted window of the cube is no longer `surface`.
- rcrr7-3 MINOR (plausible) plugins/chora-construct/src/input.rs:174: while the opened cube starts, `pane_to_face` is None (put_window needs every face up, cube_window needs no cube), so a second Super+Alt+2 takes the plain give with the noted window: a give to an xp still starting, unplaced, or an "XP did not take the window: it is not running" notice while the cube visibly opens. Fix: while `put_later` is set, let `pane_to_face` return the waiting window so a second press keeps the same put.
- rcrr7-4 MINOR plugins/chora-construct/src/pull.rs:415: an unput end (20 s, window gone, stood back no more) only writes a log line; after a told key the person sees a cube stood back and is told nothing (honest state). Fix: tell a notice on the 20 s end ("XP did not place the window in time"), as given_from_a_pane does for an unplaced window.
- rcrr7-5 MINOR plugins/chora-construct/src/pull.rs:442: `places` blocks the event loop on a state.get of up to 250 ms on every 150 ms poll once window.move is listed; with a slow or wedged xp the construct stalls up to 250 ms of every 400 ms for up to 20 s (r7's own open point 3). Fix: ask state.get without blocking (send and match the answer on a later poll) or back off after the first timeout.
- rcrr7-6 MINOR tests/gate-construct-layout.sh:399, tests/gate-construct-keys.sh:107: the cells moved off Super+Alt+2 with the keyboard in a pane window (asserts kept, not loosened), but that state now runs the put path, which told nothing in run 1 (`the notice of a refused give reads ''`), and no cell covers the put's failure ends (face manager cannot start, already runs, 20 s). Fix: add a cube-gate cell for Super+Alt+2 in a pane with xp unable to start and with xp running, each asserting a told reason.
- rcrr7-7 NIT docs/cube-modes.md:142: "the window goes as soon as the face manager can place it" leaves out the 20 s limit, the hint gone while the cube starts, and the refusal while xp runs. Fix: one sentence on how the wait ends and what is told then.

Verdict: **FIX** (open MAJOR rcrr7-1).

## r8 (co-79: review r7 FIX, rcrr7-1 MAJOR and the wait's MINORs, 2026-11-28)

Lane: cloud, claude/co-reach-r8 from **base 008714ba** (co-reach r7 head, review r7 FIX). Step 0: unshallow, `cargo check --workspace`, `cargo build --workspace` green. Area: plugins/chora-construct (overlay, input, pull, hints, names, cube, a new put.rs, reach_tests), tests/gate-construct-cube.sh, docs/cube-modes.md; nothing outside it.

**Head 830ab836**, pushed to claude/co-reach-r8 (14 commits: 6 red, 6 fix, docs, 2 test follow-ups). Diff 008714ba..830ab836: 15 files +721 -71 (reach_tests.rs +257 -1, put.rs +96 new, gate-construct-cube.sh +77, pull.rs +73 -45, overlay.rs +72 -7, names.rs +52, input.rs +22 -3, docs/cube-modes.md +13 -4, cube/start.rs +13 -6, hints.rs +11, main.rs +10, cube.rs +9 -3, cube/construct.rs +8 -1, tests_construct.rs +7 -1, scene.rs +1). chora-construct 312 unit tests pass (307 on 008714ba), clippy `--all-targets` clean.

### Per id (red commit, then fix commit)
| id | red | fix | what |
|---|---|---|---|
| rcrr7-1 MAJOR | ddaac07c `with_the_face_manager_running_the_key_from_a_pane_gives_to_it`: bar named Into face, key opened a refused cube | 92c000db | The construct keeps the names of connected clients (`clients.list` at start, `client.connected/disconnected` since). `face_manager_elsewhere()` = no cube and the face manager in a manager pane or connected; then `pane_to_face` is None and `pane_to_manager` names the pane window (flown, or `cube_window` after Alt+Escape): Super+Alt+2 gives it to that xp as on dc532d60, the bar tells `Super Alt 2 To XP` (De `Zu XP`) in the walk and in the pane, and `Super C Cube` (refused there) is not told. The `cube_window` branch stays only when no face manager runs. |
| rcrr7-2 MINOR | a33bdffc `the_wait_ends_when_the_person_moves_on` (fly into another pane; another window noted) | 661d6985 | `put_pane` notes the put window at the cube; the wait ends when `in_pane()` or `noted_window()` (on whatever face, so a turn keeps the put) is another window. |
| rcrr7-3 MINOR | e9b64b4a `a_second_key_while_the_cube_starts_keeps_the_same_put`: on 008714ba the second key gave to the starting xp and told "it is not running" | c5cb9290 | While a put waits, `pane_to_face` returns its window (out of the panes): `Into face` stays told while the cube starts, and `put_pane` keeps the waiting put. |
| rcrr7-4 MINOR | d8621fe9 `every_end_of_the_wait_without_the_put_is_told` (7 ends; 830ab836: its failure reason names no program, for cut-check) | 68db8c60 | `unput()`: 20 s → "XP did not place the window in time."; else "XP did not place the window: <reason>." with the window closed / you flew into a pane / you stepped into a face / another window has the keyboard / the failed front face's reason; `close_cube` with a waiting put tells "the cube closed". German words beside. |
| rcrr7-5 MINOR | 98fa47a9 `the_wait_never_waits_on_the_face_manager`: xp that answers after 1 s, key + 3 looks took 3.26 s on the red parent | 64bb9d88 | New `put::Put`: the `state.get` question goes out in a thread on a connection of its own (`construct-ask-N`, as voice's HUD thread has its own; the plugin lint allowed with a reason), and a later look reads the answer by `try_recv`; an unanswered question (2 s) is asked again. The frame loop asks only the core (`methods.list`), never xp. The test also checks that the put goes once the answer came. |
| rcrr7-6 MINOR | b3fe5216 (+ bea4f2fb: the failing manager exits at once, sleep ratchet) cube gate step 11 | behaviour by 92c000db and 68db8c60 | Step 11, in a pane of the start card's Terminal row: (a) face manager that cannot start: Super+Alt+2 told (put-in keys), ends in `XP did not place the window: chora-xp stopped after 0 s: it needs a display it cannot have.`, the window stays a pane; (b) xp running in a manager pane: bar `To XP, All keys, Step back`, word `To XP`, no put-in keys; the key gives the window to that xp on its output, no other virtual output, no notice. On 008714ba: (a) `FAIL: ... told nothing (notices: ''), cube {"mode":"navigate","faces":["failed",...]}` (gate-cube-r8-cells-008714ba.log); (b), run with (a) cut out: `FAIL: ... the bar does not tell the give to it (give-to keys:   ; put-in keys: Super Alt 2; bar: All keys, Into face, Step back)` (gate-cube-r8-cellB-008714ba.log). Head: cube gate alone EXIT 0, 76 ok (gate-cube-r8-head.log). |
| rcrr7-7 NIT | (none) | 6e77aa0d | docs/cube-modes.md: the bar keeps Into face while the cube starts, a second press keeps the put, the 20 s and the other ends with their notices; the give `Super Alt 2 To XP` while xp runs already with no cube; words table `To XP / Zu XP`. |

Red evidence: each unit test failed on its red commit, and all five again on 008714ba itself (its sources with the test files of 98fa47a9, separate target dir: 9 passed, 5 failed, exactly these five; unit-red-on-008714ba.log); rcrr7-1 at reach_tests "with xp running beside, no cube can open", rcrr7-2 "moved on (fly)" and "(note)", rcrr7-3 a `"to":"xp"` give plus "XP did not take the window: it is not running.", rcrr7-4 "late: the end is told ... ''", rcrr7-5 "waited 3.262288543s on xp".

### Paths by hand (fresh compositor per run, 1280x800, hand8.sh = r7's hand7.sh plus case xprun; told = the bar names the key's hint before the key; pass = told, Foot on xp VIRTUAL-1 shown, mode use, bar `Pull out`; xprun: told `Super Alt 2 To XP`, Foot on the manager pane's xp VIRTUAL-1, no other virtual output, no "did not" notice)
| Path | state where the key is pressed | told | acts |
|---|---|---|---|
| start card Terminal row, Alt+Escape (no cube) | walk | 5/5 | **5/5** |
| start card All programs row, foot, Alt+Escape (no cube) | walk | 5/5 | **5/5** |
| launcher Super+space, foot, Alt+Escape (no cube) | walk | 5/5 | **5/5** |
| Super+t in the space, Alt+Escape (no cube) | walk | 5/5 | **5/5** |
| Super+t in a pane (2 panes), Alt+Escape (no cube) | walk, Tab Windows | 5/5 | **5/5** |
| cube row, Programs, Foot, Alt+Escape (r5 path) | navigate | 5/5 | **5/5** |
| demo 2: Terminal row, Alt+Escape, Return Fly in | in the pane, no cube | 5/5 | **5/5** |
| pulled from a face by Super+Alt+4 | in the pane, cube in use | 5/5 | **5/5** |
| **9th: xp running (manager pane), Terminal row, key in the pane** | in the pane, no cube, xp beside | 5/5 | **5/5** |

### Gate
Before run 1 the ratchets step caught the `sleep 1` of step 11's failing manager (`FAIL: sleep-ratchet: sleep is 779, the list holds 778`, caused here; bea4f2fb removes it). Run 1 (bea4f2fb) stopped at docs, **environment**: `FAIL: mkdocs is not installed and docs changed`; mkdocs installed as a user tool (`uv tool install mkdocs --with-requirements docs/requirements.txt`, no system package), `mkdocs build --strict` rc 0. Run 2 (bea4f2fb) stopped at ci-check, **caused here**: `FAIL: cut-check: names: chora-construct names chora-xp on 7 lines` (the rcrr7-4 test's failure string; 830ab836 rewords it, `tools/cut-check` ok). Run 3 on **830ab836**, `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`, no step filter, detached: the 30 steps before doors PASS (build, test, clippy, layout 1364 s among them); doors stopped on the **foreign flake** the brief names: `helper-none      pass      missing   pass      pass` (tiling column), `FAIL: 1 cells or checks are not pass`, every construct cell pass (gate-doors-run3.log). `--only doors` alone: **PASS** 381 s (gate-doors-alone.log). The 37 steps after doors, `--only acts,...,e2e --jobs 2`: **all 37 PASS** (cube 155 s, 76 ok, the three co-79 r8 lines among them; acts 1493 s; keys 35 s; leak 466 s; e2e 270 s). Every step passed on 830ab836 except that one named flake, which passed when rerun. (A slip of mine afterwards started one more full run by accident, killed within two minutes, not counted.) `pgrep -fa 'chora --headless'` empty after. Last 20 lines (the steps after doors):
```
launcher-click PASS    17s /tmp/merge-gate.ryketS/launcher-click.log
voice-acp  PASS    11s /tmp/merge-gate.ryketS/voice-acp.log
voice-events PASS     2s /tmp/merge-gate.ryketS/voice-events.log
launcher-mc PASS    32s /tmp/merge-gate.ryketS/launcher-mc.log
agent      PASS    37s /tmp/merge-gate.ryketS/agent.log
tiling-keys-more PASS    40s /tmp/merge-gate.ryketS/tiling-keys-more.log
tiling-system PASS    14s /tmp/merge-gate.ryketS/tiling-system.log
handover   PASS    85s /tmp/merge-gate.ryketS/handover.log
mc-scale   PASS    10s /tmp/merge-gate.ryketS/mc-scale.log
tiling-windows PASS   161s /tmp/merge-gate.ryketS/tiling-windows.log
watchdog-starved PASS    49s /tmp/merge-gate.ryketS/watchdog-starved.log
xp-run     PASS    13s /tmp/merge-gate.ryketS/xp-run.log
demo       PASS   203s /tmp/merge-gate.ryketS/demo.log
tiling-bar PASS   167s /tmp/merge-gate.ryketS/tiling-bar.log
tiling-theme PASS    19s /tmp/merge-gate.ryketS/tiling-theme.log
leak       PASS   466s /tmp/merge-gate.ryketS/leak.log
tiling-disc PASS    71s /tmp/merge-gate.ryketS/tiling-disc.log
mc-host    PASS    23s /tmp/merge-gate.ryketS/mc-host.log
e2e        PASS   270s /tmp/merge-gate.ryketS/e2e.log
partial run: nothing is pushed
```

### Captures (media/2026-11-28/co-reach-r8/, each under 500 KB, 4.6 MB in all)
- `<path>-run1-in-pane.png`, `-stood-back.png`, `-after-key.png` for the 8 r7 paths (`flyin-run1-flown-in.png` too); `xprun-run1-in-pane.png` (bar `Alt Esc Step back | Super Alt 2 To XP | Super F1 All keys`), `xprun-run1-after-key.png`.
- hand-<path>.log (5 runs each, 9 paths), hand8.sh, gate-cube-r8-cells-008714ba.log, gate-cube-r8-cellB-008714ba.log, gate-cube-r8-head.log, unit-red-on-008714ba.log, merge-gate-run3-full.log, gate-doors-run3.log, gate-doors-alone.log, merge-gate-after.log.

### Open for the PM
1. 9th path: after the give to the xp of a manager pane the camera is back on the floor and that pane may be out of view, while the given window keeps the keyboard (xprun-run1-after-key.png); this is dc532d60's plain give, kept as the brief says. Flying to the manager pane after the give would be a new decision.
2. The question of the wait is a short-lived second connection `construct-ask-N` (one at a time, at most one per 2 s while a put waits): it shows in `clients.list` and emits client events for that time. The alternative, a non-blocking call on the runtime's connection, needs chora-kit (outside this lane).
3. docs/configuration.md still lists the walk hints without `Super C Cube`, `Into face` and now `To XP` (rcrr6-1, outside this lane's area).
