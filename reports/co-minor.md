## r1

Lane: cloud, branch claude/co-minor-r1 from 6f37120b. Step 0: `git fetch --unshallow`, `cargo check --workspace` green. Area: plugins/chora-construct (camera.rs, adopt.rs, plugin.rs, overlay.rs, main.rs), unit tests, gate-construct-cube. 8 files +227 -10.

**Head: dd1ac7e4** (red 6c040c09, fix 7becf38f, then dd1ac7e4 drops a sleep that the sleep-ratchet counted). Pushed to claude/co-minor-r1.

### Cause (co-78 and rcr1-1, one chain)
With the key sheet open over a face the construct holds the keyboard (a node focus on its look). A new window goes to the holder of the keyboard (core rule: "to the focus, else the smallest claim under the pointer"), so the window of Super+t was reserved for the construct, which adopted it as a pane (`terminal_beside`, flown in); the face stayed empty (rcr1-1). The window then took the keyboard from the sheet, so Escape went to it, not to the sheet (co-78). Hand repro on 6f37120b: after Super+t `s1 owner construct`, rect 0,0,1280,800, Escape leaves `open: construct-help` (as co-repro r1).

### Fixes
- rcr1-1: in a face (`in_a_face()`: cube in use mode, camera in no pane) `terminal_beside` starts the terminal for the face (`start_terminal_in_face`): the token is kept apart from the pane waits, no pane is made; when the window comes (`created`) the construct takes what the core gave it and lets it go to the manager of the faces (`Remove {to: xp}`); xp shows it on the face.
- co-78: a focus on a non-pane window while the key sheet is open in a face (`focus_heard`) is noted for the close and the keyboard goes back to the sheet (`help_retakes_keyboard`); Escape closes the sheet (existing path) and the close gives the noted window the keyboard (`activate_in_face`).

### Red then fix (pasted red lines)
| Item | Red | Fix | Red line |
|---|---|---|---|
| rcr1-1 | 6c040c09 | 7becf38f | unit `face_start_tests.rs:19`: `the terminal of Super+t over a face is no pane of the construct: 1 waits for a pane`; cell: `FAIL: rcr1-1: Super+t with the key sheet open over a face started a window of construct, not of the face manager xp: s2 construct HEADLESS-1 566 326 148 148` |
| co-78 | 6c040c09 | 7becf38f | unit `face_start_tests.rs:37`: `the keyboard goes back to the sheet: []`; cell (owner check skipped to reach it): `FAIL: rcr1-1: the window of Super+t is on HEADLESS-1, not on the face VIRTUAL-1` (the pane; the Escape line behind it is hidden until the window is in the face) |
| co-77 | none (pin, see below) | - | - |

The two items share one red commit and one fix commit: the gate cell is one scenario (sheet, Super+t, Escape) in one file. Green after: `ok: rcr1-1: Super+t with the key sheet open over a face starts a window of the face (xp, VIRTUAL-1)`, `ok: co-78: Escape closes the key sheet after Super+t opened a window of the face, and that window has the keyboard`. The cell asserts nodes, not a look: owner `xp` and output of the window by `scene.query`, `shown` in `state xp`, the sheet node `construct-help` opacity back to 0, and the keys 'a' of the probe window log after Escape.

### co-77: no fault; contract right, brief reading differs (one line for the PM)
A program from Programs (the launcher) in a face that holds a window is held by **xp**, as the faces of a hosted cube are (D106: xp holds the windows of its faces, the construct holds the cube and the turn; WM7b), and that is what the contract C3 already accepts (`focus_with_flavor`: the manager of the front face on a window that shows): hand run in a face with a Super+t window, then Programs, Foot: `xp focus s2`, `construct holder null`, both windows owner xp, windowed 700x500 inside the face, `check.py end C3`: `task C3 pass [scene: cube in a face, face 0]` (5 of 5 in co-repro r1 and in the cell below). I did not make construct hold it: that would break D106 and the faces' window manager (taskbar, title, pull). The r12 C3 FAIL (stood back, xp s3, "shown" probably false) did not reproduce (stood back with no face window Foot is a construct pane and C3 passes; stood back with a face window the launcher started nothing in my run). **Decision for the PM:** if a stood-back cube should also give Programs windows to the construct, that is a product call; the contract is not wrong and needs no change.
Cell (green, a pin, no red): `ok: co-77: a program of the launcher in a face that holds a window is shown on that face, held by the face manager xp, and takes keys, 5 of 5 runs` (gate-construct-cube; launcher is a stand-in that starts a chora-probe window, no child of the launcher, ends at once as fuzzel does).

### Captures (media/2026-11-25/co-minor-r1/)
- rcr1-1-super-t-sheet-open-window-in-face.png: sheet open, the terminal window behind it on the xp face (not a pane).
- co-78-escape-closes-sheet-window-has-keys.png: sheet closed by Escape, the window focused (blue frame), typed `abc` in it, hint bar names Pull out.
- co-77-foot-from-programs-in-face.png: Foot from Programs in the face that holds a terminal, two xp windows.
- hand.sh: the hand run (cube row, Return, Super+F1, Super+t, Escape; and Super+t, Super+space, foot).

### Gate
`tests/merge-gate.sh --jobs 2` (no step filter), detached, polled in the turn, on dd1ac7e4: **EXIT 0, 65 PASS, 0 FAIL**. A first run on 7becf38f stopped at `ci-check` (sleep-ratchet: sleep 779 vs 778, my `sleep 3` in the stand-in launcher), fixed in dd1ac7e4 (no new sleep in the tree). Unit: chora-construct 288 passed, clippy clean. `pgrep -fa 'chora --headless'` empty after. Last 20 lines:
```
voice-acp  PASS    13s /tmp/merge-gate.lV6cwD/voice-acp.log
voice-events PASS     2s /tmp/merge-gate.lV6cwD/voice-events.log
agent      PASS    42s /tmp/merge-gate.lV6cwD/agent.log
tiling-keys-more PASS    44s /tmp/merge-gate.lV6cwD/tiling-keys-more.log
tiling-windows PASS   210s /tmp/merge-gate.lV6cwD/tiling-windows.log
tiling-system PASS    16s /tmp/merge-gate.lV6cwD/tiling-system.log
handover   PASS    28s /tmp/merge-gate.lV6cwD/handover.log
mc-scale   PASS    18s /tmp/merge-gate.lV6cwD/mc-scale.log
watchdog-starved PASS    58s /tmp/merge-gate.lV6cwD/watchdog-starved.log
acts       PASS  2351s /tmp/merge-gate.lV6cwD/acts.log
xp-run     PASS    15s /tmp/merge-gate.lV6cwD/xp-run.log
demo       PASS   279s /tmp/merge-gate.lV6cwD/demo.log
leak       PASS   483s /tmp/merge-gate.lV6cwD/leak.log
tiling-theme PASS    19s /tmp/merge-gate.lV6cwD/tiling-theme.log
tiling-disc PASS    71s /tmp/merge-gate.lV6cwD/tiling-disc.log
mc-host    PASS    20s /tmp/merge-gate.lV6cwD/mc-host.log
tiling-bar PASS   201s /tmp/merge-gate.lV6cwD/tiling-bar.log
e2e        PASS   287s /tmp/merge-gate.lV6cwD/e2e.log
all steps passed; no --push, nothing is pushed
EXIT 0
```

### Not done / open
- co-77 owner stays xp by design (above); the r12 stood-back FAIL is not reproduced and not explained (the xp start menu click path of Max was not replayed exactly).
- Stood back, Super+t with the key sheet open still starts a pane (K-mode navigate, unchanged by design: the fix is for a face only).

## PM decision r1 (2026-11-25 10:20 UTC)
- co-77: no fault. D106 and WM7b hosted (owner 11-19 10:52) hold: a Programs window in a face is held by xp; C3 pin cell passes 5/5. Closed as by design after the review confirms the pin cell.
- Diff 227 lines, construct focus path: seeded opus review (briefs/review-co-minor-r1.md).

## Review r1

Reviewer: seeded review (briefs/review-co-minor-r1.md), cloud, read-only on code. Head dd1ac7e4, base 6f37120b (8 files +227 -10). Step 0: `git fetch --unshallow`, `cargo check --workspace` green. No merge gate run (the lane ran it, EXIT 0).

Checks:
1. Red first: yes. 6c040c09 has tests only, 7becf38f has the fix. At 6c040c09, `cargo test -p chora-construct face_start` fails both new tests by their own asserts (`face_start_tests.rs:19: ... 1 waits for a pane`; `face_start_tests.rs:37: the keyboard goes back to the sheet: []`, 2 failed). The cell's red lines are pasted in r1.
2. rcr1-1: in a face, `terminal_beside` makes no pane and `created` hands the window to the face manager (Attach, then Remove to xp). `face_tokens` keeps at most 8 tokens (`drain(..len-8)`). Each token is unique, so a token that never gets a window steals no later one and is pushed out after 8 more. When `free` is false the token is dropped and the window stays with its holder, which is correct. Outside a face (`in_a_face` false), Super+t takes the old path and makes a pane.
3. co-78: `focus_heard` takes the keyboard back for a face window while the sheet is open. Escape closes the sheet, and `give_the_keyboard_back` then activates the noted window through xp. No loop: xp does not refocus on release (`keys.rs focus_moved`), and the sheet sets `help = false` before it gives the keyboard back. The filter is too wide, see rcm1-1.
4. Gate: no weakened assert. Every wait is a bounded `until_ok`. `tools/sleep-ratchet` gives `ok (sleep 778, sleep-all 929)`. Each launcher probe is daemonised, then `end_others` ends it and waits until it is gone; on a fail, `gate_cleanup` ends it through `gate_probes`.
5. co-77: D106 is the contract and C3 (`focus_with_flavor`) accepts xp with a shown window. The pin checks owner xp, `shown`, the window inside the face and that keys reach it, 5 of 5. It does not check C3's single holder; see rcm1-3.
6. Beyond: dropping the `Mode` import leaves no dead path (`in_a_face` names it in full). No doc says Super+t in a face makes a pane.

Findings:
- rcm1-1 MAJOR plugins/chora-construct/src/plugin.rs:216: `focus_heard` takes the keyboard back from any window that is not a pane (`own_window` covers only panes). The window of the launcher is not a pane. With the sheet open in a face, Super+space (the sheet lists it as "Start a program", among the keys that work now) shows the launcher with `focus::give`, and the sheet takes the keyboard back at once. A unit probe on dd1ac7e4 (`show_launcher("s9")`, then `focus_changed(s9)`) gives `focuses [None]`. Escape then calls `activate_in_face(s9)` on xp, which does not hold s9, so the launcher has no keyboard. This is a regression from 6f37120b. Fix: filter with `!self.owns(s)` (panes, hidden, launcher window) and add a red unit test for the launcher.
- rcm1-2 MINOR tests/gate-construct-cube.sh:350: the co-78 cell types no key while the sheet is open, so "no key reaches the window while the sheet is open" has no assert (Escape closing the sheet only shows that Escape does not reach the window). Fix: before Escape, type `a` and assert that the `key 30` count of term.log does not change.
- rcm1-3 NIT tests/gate-construct-cube.sh:358: the co-77 pin does not assert C3's single holder (no node focus of the construct). It covers only the in-face case, with a stand-in launcher that has no window of its own. Fix: assert after each run that the construct holds no focus (or run `check.py` C3), and keep the stood-back r12 case named as open in findings.

Counts: 0 BLOCK, 1 MAJOR, 1 MINOR, 1 NIT.
Verdict (computed): FIX (rcm1-1 MAJOR open).

## r2

Branch claude/co-minor-r2, head 466fcd11 (merged 6b89b56f first). Full gate `tests/merge-gate.sh --jobs 2`: EXIT 0, all steps PASS (cube 141s). No chora process left.

- rcm1-1 MAJOR: red 87963e9d, fix fbbd8e31 (`focus_heard` filters with `!self.owns(s)`). Red line: `face_start_tests.rs:64: the sheet takes the keyboard back from the launcher: [None]`. Green: face_start 3 of 3 (the co-78 face-window test still passes).
- rcm1-2 MINOR: test commits f7b3707a and 466fcd11 (gate-construct-cube.sh). The cell types `a` with the sheet open and asserts the window's `key 30` count stays (bounded `until_ok`, no new sleep), then Escape (first Escape clears the search the sheet took from `a`, so a second closes it), then `a` reaches the window. The product leaks no key, so the new assert passes at once (no red). First attempt failed at the Escape step because the typed `a` is the sheet's search text; fixed in the test, not the product.
- Not done: rcm1-3 (NIT, backlog).

## PM review r2 (dd1ac7e4..466fcd11, own commits +34 -1 plus merge 6b89b56f; PM not the builder)
- Red first: 87963e9d (face_start_tests: sheet takes keyboard back from the launcher, red `[None]` per lane) before fbbd8e31 (`!self.own_window(s)` to `!self.owns(s)`). rcm1-1 closed: `owns` (lifecycle.rs:138) covers panes, hidden windows and launcher_window, so the sheet no longer pulls focus from the launcher.
- rcm1-2: f7b3707a, 466fcd11 test only; the product leaked no key, so no red is expected; bounded `until_ok ... 8`, no sleep. Closed.
- rcm2-1 NIT: `owns` also spares hidden windows from the sheet's retake; a hidden window cannot take focus in a face, so no fault today; name it in the fn doc if that changes.
- Full gate EXIT 0 on 466fcd11 (lane). Verdict computed: 0 BLOCK, 0 MAJOR: MERGE-OK 466fcd11.

## r2 (brief co-minor-r1: rpc2-1, rpc2-2, rcrr6-1)

The brief names this "## r1"; that heading is the lane above (co-78, rcr1-1), so this one is "## r2" to keep the headings apart (rcrr6-5 family).

Lane: cloud, branch claude/co-minor-r1 at 57ce1480 (session head 7c6955c8 with claude/co-reach-r6 bcecb671). Step 0: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace` green (the local branch of that name pointed at 6f37120b9 of the earlier lane; reset to origin 57ce1480 first). Area: plugins/chora-construct (config.rs, overlay.rs, keys.rs, camera.rs, main.rs, command.rs), reach_tests.rs, tests/gate-construct-cube.sh (step 9), docs/configuration.md. 9 files +268 -27.

**Head dc532d60** (red 3d312128, fix dc532d60). Pushed to claude/co-minor-r1.

### Cause
- rpc2-1: `help_groups(cube)` built the key sheet from every binding and its own rows, whatever held the keyboard. The grabs (`grab_keys`) already knew which keys reach a window (a chord with a modifier, a media key, F1) and which do not (a bare key types into it), but the sheet did not read that rule.
- rpc2-2: `help_takes_keyboard` left the window of a pane unremembered when the cube stood back (`_ if stood_back => {}`): the close then gave nothing back, the core focus stayed empty and `put_window` (core focus, else noted) named no window, so the bar lost `Super Alt 2 Into face`.

### Fixes (one rule each)
- rpc2-1: `config::works_in_window(key)` is the one rule; `grab_keys` and `help_groups(cube, window)` both read it. With a pane's window on the keyboard (`pane_window_has_keys`: camera in the pane, or the core focuses a pane window; read before the key of the list flies out of the pane) only held bindings make rows, the Cube group and `Back out one step Escape` go, empty groups drop. No special case per path: the hint `All keys` and the key `Super+F1` use the same flag.
- rpc2-2: `help_takes_keyboard` notes a pane window as `help_back` in every mode; the close gives it the keyboard back when it is flown or in sight, so core focus is the window again and the bar names `Into face` as without the sheet.
- rcrr6-1: docs/configuration.md says at most six hints, lists `Super C Cube` with its condition (the window of a pane has the keyboard, no cube open) and says what the sheet lists in a pane; matches docs/cube-modes.md.

### Red then fix
| Item | Red (3d312128, on 57ce1480) | Fix | Red line |
|---|---|---|---|
| rpc2-1 | unit `the_key_sheet_in_a_pane_lists_only_the_keys_that_work_there`, `..._opened_by_its_key_in_a_pane_lists_the_same_keys`; cell step 9 | dc532d60 | `"Fly into the pane ahead" reaches the window, not the sheet: Esc|Close|WALK|CUBE|...`; gate `FAIL: rpc2-1: the key list in a pane lists "Fly into the pane ahead", which types into the window: Esc|Close|WALK|CUBE|...` |
| rpc2-2 | unit `after_the_key_sheet_in_a_pane_the_step_back_names_the_way_into_the_face` | dc532d60 | `closing the sheet gives the pane's window the keyboard back left: Some(None) right: Some(Some("s1"))` |

The gate cell stops at its first FAIL, so on 57ce1480 only rpc2-1's line shows; rpc2-2's red is the unit test (its cell line was not seen red).

### Cells (green twice)
`ok: rpc2-1: the key list in a pane lists only the keys that work there` and `ok: rpc2-2: after the key list in a pane the step back names Super Alt 2 as it does without it`, gate-construct-cube step 9, 2 of 2 full runs after the last change (and in the full gate as step `cube`). Unit: chora-construct 305 passed, clippy clean. Co-79 r6 cells (steps 7, 8) green in every run.

Cell design: step 9 opens a cube, ends the leftover face window of step 8, starts a program from the launcher (a pane flown into), clicks the hint All keys (the path of the report), reads the texts of `construct-help*` (no walk or cube row, the way out listed), Escape, clicks the hint Step back, reads the put-in key caps. Three earlier shapes flaked and were dropped: the key `Super+F1` right after the fly-in (the key flies out first: the flag is now read before), Alt+Escape with a face window left (the keyboard then belongs to that window, rcrr6-3), and a click before the hint was drawn. `SHEET_SHOT` and `BACK_SHOT` keep captures.

### Captures (media/2026-11-26/co-minor-r1/)
- rpc2-1-key-sheet-in-a-pane-only-keys-that-work.png: the sheet in a pane: PANES, HELPER AND CHAT, GIVE A WINDOW, EVERYWHERE; no WALK, no CUBE; footer "Only the keys that work now are listed."
- rpc2-2-stood-back-after-sheet-names-into-face.png: after the sheet and Step back, the bar `Click Fly in | Super Alt 2 Into face | F1 All keys`.

### Gate
Run 1 of `tests/merge-gate.sh --jobs 2 --allow-skip docs` (no step filter): `docs` FAIL "mkdocs is not installed and docs changed" (host: mkdocs missing; I installed mkdocs and mkdocs-material in a venv in the scratchpad, `mkdocs build --strict` rc 0). Run 2 (venv on PATH): `ci-check` FAIL `FAIL: without Inter the clock selftest did not print a skip line naming Inter (TF-1)` after `skip: clock selftest, the Python module PIL is missing`: foreign, host (PIL missing), nothing of this diff; with Pillow in the venv `ci-check` alone PASS. Then every step after it in one run, `--only build,...,e2e --jobs 2`: **61 PASS, 0 FAIL, 0 SKIP** (red-first to gate-lib passed in run 2, ci-check alone). `pgrep -fa 'chora --headless'` empty. Last 20 lines:
```
mc-column  PASS    49s
voice-acp  PASS    12s
voice-events PASS     2s
launcher-mc PASS    41s
agent      PASS    40s
tiling-keys-more PASS    42s
tiling-system PASS    15s
handover   PASS    99s
mc-scale   PASS    13s
tiling-windows PASS   181s
watchdog-starved PASS    52s
xp-run     PASS    13s
demo       PASS   236s
tiling-bar PASS   178s
tiling-theme PASS    19s
leak       PASS   476s
mc-host    PASS    21s
tiling-disc PASS    72s
e2e        PASS   280s
partial run: nothing is pushed
```

### Not done / open
- rpc2-3, rpc2-4, rpc2-5, rpc2-6, rcrr6-2..6 stay in the backlog as briefed.
- Seen while building the cell, not filed before: with a face window left on the cube and a pane flown into from the launcher stood back, the keyboard can stay with the face window (rcrr6-3 family); Alt+Escape then does not stand back.
- The gate lane needs mkdocs and PIL on the host for `docs` and `ci-check`; the venv is a host fix, nothing in the repo.

## Review r2 (co-minor r1: rpc2-1, rpc2-2, rcrr6-1; reviewer not the builder)

Reviewed claude/co-minor-r1 **57ce1480..dc532d60** (9 files +268 -27), one round, read-only on code. Step 0: `git fetch --unshallow`, `cargo check --workspace` green. On head: `cargo test -p chora-construct` 305 passed; `cargo clippy -p chora-construct` clean; `tests/gate-construct-cube.sh` EXIT 0, 70 ok (rpc2-1, rpc2-2 and the five co-79 lines among them).

**Verdict: MERGE-OK** (0 BLOCK, 0 MAJOR).

Checks:
1. Red first: confirmed by runs of my own. 3d312128 (tests only, +193 in reach_tests.rs and the gate) builds on 57ce1480's code: `cargo test -p chora-construct` 302 passed, 3 failed (`the_key_sheet_in_a_pane_lists_only_the_keys_that_work_there`, `..._opened_by_its_key_in_a_pane_lists_the_same_keys`: `"Fly into the pane ahead" reaches the window, not the sheet: Esc|Close|WALK|CUBE|...`; `after_the_key_sheet_in_a_pane_the_step_back_names_the_way_into_the_face`: `closing the sheet gives the pane's window the keyboard back`). Head gate script on a 57ce1480 build: EXIT 1 at `FAIL: rpc2-1: the key list in a pane lists "Fly into the pane ahead"` (every earlier cell ok); on dc532d60 all 305 pass, gate EXIT 0. rpc2-2 has no cell red on base (the gate stops at rpc2-1), its red is the unit test, as the lane says.
2. rpc2-1: yes. `help_groups(cube, window)` drops held bindings by `works_in_window`, the Cube group, `Back out one step Escape`, and empty groups; `help_window` is read before the key of the list flies out (`grabbed_key`). Face path: `window` is false (`own_window` is panes only). One rule only in part: rcmr1-1, rcmr1-2.
3. rpc2-2: `help_takes_keyboard` now notes a pane's window in every mode (`stood_back` arm gone), so the close gives it back; unit test and cell step 9 (bar `Super Alt 2`) green.
4. POLS: step 7, 8 and the face-path cells green on head; no walk or face sheet changes (`window` false there); the walk hints hold at most six (`walking`, Walk, FlyIn, Windows, Cube, Terminal, Keys).
5. Docs: configuration.md "at most six", `Super C Cube` with its condition (pane window has the keyboard, no cube open) and the pane sheet paragraph match `hints::walking`, `cube_window` and docs/cube-modes.md:124, :140.
6. ci-check and docs: confirmed host only. merge-gate.sh:261 fails `docs` with "mkdocs is not installed and docs changed" (docs/configuration.md changed, `command -v mkdocs` empty; mkdocs absent on this host too). tests/tiling-scripts-selftest.sh:98-111 prints `skip: ... PIL is missing` and its TF-1 case needs a skip line naming Inter, so it fails without PIL; no code path of this diff.

Findings:
- rcmr1-1, MINOR, plugins/chora-construct/src/config.rs:443 vs camera.rs:656: the rule is not one: `grab_keys` also filters by action (only Terminal, Close, Run, `in_every_look` and Look Cube/Windows are grabbed) while the sheet filters by key form only, so a ricer binding `Super+Return: fly_in` is listed as "Fly into the pane ahead" in a pane though it is not grabbed and types into the window; fix: let `works_in_window` take key and action and have the grabs and the sheet call it.
- rcmr1-2, NIT, config.rs:669, hints.rs:116: `works_in_window` counts `F1` as working, but the core grabs it only with `input.grab-any` (the hint's `keys_chord` asks `holds`, and keeps its own `contains('+')` copy), so a lone `F1: keys` binding without the grant is listed in a pane; docs/configuration.md says "the chords with a modifier and the media keys", no F1; fix: `keys_chord` calls `works_in_window` with `holds` for F1, the sheet takes the same held check, and the docs name F1 with its grant.
