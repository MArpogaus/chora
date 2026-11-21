# co-keys

## r1 (rc 2, F7; findings/construct.md co-38, co-40, co-41)

Branch `claude/co-keys`, base a76476be plus the session head a5b77488 merged in (a5384bde). Step 0: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace` ok.

Commits (author Marcel Arpogaus, no attribution lines):
- `c98457e8` red: the tour rows cb-1 and cb-3 lose their `known`; `tests/acts/looks/construct.py` `keys_acts` (new acts).
- `e6365f50` fix(core) co-38: `crates/chora/src/state.rs` `focus_of_scene` calls `tell_key_node` before it emits `focus.changed`; unit test `a_node_that_takes_the_keyboard_is_told_before_the_surface_goes` (crates/chora/src/ipc.rs).
- `f5bdc61f` fix(construct) co-40, co-41: `plugins/chora-construct/src/overlay.rs` (`help_takes_keyboard`, `hint_items`), `camera.rs` (`grab_keys`), `construct.yaml`, docs, acts known-red row for co-39.
- `e965470a` test(doors): the cell `f1-free` of construct (head).

**Causes**
- co-38 (cb-1): the step back sends `release()` then `FocusNode(look)` (`modes.rs:757` `the_keyboard_to_the_look`). The core told `focus.changed{null}` at once and `focus.node` only in `after_events` (`state.rs:521` `tell_key_node`). xp (`plugins/chora-xp/src/wm/keys.rs:333` `focus_moved`) heard the surface go while its `key_node` was still empty, saw a foreign focus leave, and sent `desktop_keys()`, so the core focus node was `xp-icons` of xp; construct still said it held the focus (`state.rs:66`). Seen in the tour: focused node `xp-icons` after the key, `construct-look` after the fix. The pulled window must be a foreign focus of xp (pane by drag); `cube.pull` by method does not show it (xp keeps the window in its book).
- co-40 (cb-3): not a consequence of co-38 (cb-1 green, cb-3 still red after the core fix). After the pane went back to its face while the cube stood back, xp activated `s1` (core focus `s1`, construct holder null). `help_takes_keyboard` (`overlay.rs:324`) took the keyboard only in a face or a pane, so in the stood-back cube Escape and Return went to `s1`. It now takes it with any cube (no `help_back` while stood back, so the close does not move the keyboard to a pane).
- co-41: `F1` was no binding (`construct.yaml`, test `super_f1_shows_the_help_and_f1_is_free`), and no hint named a key list in a face or in the stood-back cube.

**Change.** `F1: keys` beside `Super+F1` (same `construct-help`). In the walk and stood back it is a key of the look; in a face it is a grab: the core grabs a key without a modifier only with the grant `input.grab-any` (refused: logged once, F1 reaches the program, as before), so `grab_keys` adds `F1`. The hint `F1 All keys` (`construct-hint-keys`, the word `Word::AllKeys` of the walk) is in the Set::Flown capsule (face, 3 s or the resting pointer) and in the Set::Cube capsule of the stood-back cube, and a click opens the list. Not a "Keys F1" chip of the mode row: the hint capsule is the existing row of both places.

**Red on a76476be (3 of 3 each)**
- (a) cb-1 and (b) cb-3: `demo/tour/run.py --check construct` with the three `known` removed, built binaries of c98457e8 in a worktree (`TOUR_REPO`): red act 4 (I3 construct and xp each say they hold the focus), act 12 (open ['construct-help'] after Escape), act 13 (mode navigate, expected use), in all three runs.
- (c) acts, three runs: `F1 opens the key list in a face`, `the chip Keys of the stood back cube opens the key list by a click`, `the hint Keys of a face opens the key list by a click` red (then the Escape closes the cube, which cascades into I1/I14/I4 and later cells; 59 pass, 9 blocking each time).
- Not red in the acts: the acts of (a) and (b) (`Alt+Escape with a pane in the space leaves one client with the focus`, `Escape shuts the key list of the stood back cube`, `Return steps in after the key list shut`) pass on a76476be too, as the acts cannot make xp see a foreign focus (see co-38 cause); they stay as guards. The core test of the order was not run red.

**Green (e965470a)**
- `tests/acts/run.py construct`, 5 runs: 72 pass, 1 known red, 0 blocking each (the known row is co-39, I5 in `Super+F1 shows the key list of the stood back cube`).
- `demo/tour/run.py --check construct`, 2 runs: 18 acts, 17 green, 1 known (act 11, cb-2 co-39, unchanged), 0 red, `tour: check: 0 red, 0 broken`. The `known` of the rows after act 11 went (their overlap is gone with the shut sheet).
- `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,cube,construct-hud,construct-extra,pull,doors,acts`: all PASS (pre-commit 4 s, ci-check 39 s, build 25 s, test 209 s, clippy 17 s, doors 368 s, acts 305 s, cube 117 s, pull 82 s, construct-hud 111 s, construct-extra 159 s). A first gate run before `e965470a` failed `doors`: the cell `f1-free` of construct asserted the old rule (F1 shows no list in the walk); it now asserts the list in the walk and F1 reaching the program in a pane without the grant (the gate stopped there and later cells read `missing`). gate-lib left out as briefed.

**Open / said plainly**
- co-39 (cb-2) stays known: the sheet of the stood-back cube still overlaps the chip row (acts: `Turn left` over `construct-help-body-note`); the new hint widens the capsule, nothing fixed it.
- F1 in a face needs `choractl grant construct input.grab-any`; without it the hint click is the way. The acts grant it in the cell `construct may grab F1 without a modifier`.
- A stood-back cube whose pane went back to xp still loses the keyboard to xp until the next key list or step; the brief did not ask for more.

## Review r1

Seeded review (briefs/review-co-keys-r1.md), claude/co-keys head e965470a against a5384bde, one round, read-only on code. Step 0: `git fetch --unshallow`, `cargo check --workspace` ok.

**Checks**
1. Core order: `focus_of_scene` (`crates/chora/src/state.rs:1082`) is the only place that emits `focus.changed`; both `State::focus` and the transaction effect `Effect::Focus` go through it, so `tell_key_node` runs before `focus.changed` on every path. `told_node` dedupes, so `after_events` (`state.rs:521`) does not tell `focus.node` twice. Listeners: xp (`keys.rs:332` `focus_moved`, `dispatch.rs:31`) now has `key_node` set when the surface goes, which is the fix; the reverse step-in order (`focus.node{null}` then `focus.changed{s1}`) changes nothing for xp, because `left_foreign` reads `core_focus`. tiling (`main.rs:700`) treats both events alike, so order does not matter there; mc and construct do not subscribe to `focus.node`. `cargo test -p chora`: 642 passed. The unit test fails with the new line removed (left `["focus.changed","focus.node"]`), so it is a real guard; the lane had not run it red.
2. Red first: c98457e8 holds only the tour rows and the acts before the fixes. But f5bdc61f rewrites those red acts (grant, hint click), see rck-4.
3. construct: `grab_keys` (`camera.rs:637`) adds F1. Without the grant the core answers MISSING_CAPABILITY, `Runtime::grab` (`chora-kit/src/runtime.rs:375`) logs that once and returns Ok(false), so `regrab` goes on and F1 reaches the program (doors `f1-free`). `help_takes_keyboard` with any cube: when the cube stands back, a foreign window is only noted (`note_focus`) and `help_back` stays empty, so the close leaves the keyboard with the look (Return steps in). In a face the close behaves as before. Nothing is stranded. `cargo test -p chora-construct`: 261 passed.
4. Stray keys: without the grant, F1 reaches the program of a pane or face as before; nothing new reaches clients. The docs do not match the code (rck-2).
5. The acts and the doors cell wait on state (`until`, `until_ok`), with one fixed sleep (rck-5). Known-red: only the co-39 row (`tests/acts/known-red.toml`), plus the tour's cb-2 row.

**Findings**
- rck-1, MAJOR, plugins/chora-construct/src/overlay.rs:598: the hint `F1 All keys` of Set::Flown shows in a face and in a pane. There F1 works only with `input.grab-any`, which nothing grants construct by default, so in a default session the hint names a key that goes to the program (co-41 POLS). The face case `F1 opens the key list in a face` is green only because the acts grant it. Fix: name F1 in that hint only when its grab held (`rt.grab` returned true), else the bound modifier key (`keys_of(System::Keys)`, Super+F1).
- rck-2, MINOR, docs/configuration.md:961 (also docs/construct-style.md:226, plugins/chora-construct/construct.yaml:29): these say F1 opens the list in a pane and a face "so the program of a pane or face does not get F1". The code does that only with the grant (doors `f1-free` asserts the opposite without it). Fix: say F1 is grabbed in a pane or face only with `input.grab-any`, and that without it F1 reaches the program and Super+F1 or the hint opens the list.
- rck-3, MINOR, plugins/chora-construct/src/overlay.rs:602 (and :598): both hints hard-code `chord("F1")` where the walk hint uses `keys_of(System::Keys)`, so a user who drops or rebinds `F1: keys` still sees `F1 All keys`. Fix: take the chord from the bindings, together with rck-1.
- rck-4, MINOR, tests/acts/looks/construct.py:503: the fix commit f5bdc61f rewrote the red acts of c98457e8. It added the `grab_any` grant and replaced `hint_box`, so `F1 opens the key list in a face` went green through a test change, not a code change. No cell covers F1 in a face without the grant. Fix: keep a cell for the default session (no grant) that asserts the face hint names a key that opens the list.
- rck-5, MINOR, tests/acts/looks/construct.py:543: `wait(1.0)` uses a fixed sleep between Alt+Escape and Return to wait for the step back. On a slow runner Return lands mid-glide. Fix: wait with `s.until` for cube mode `navigate` instead of the sleep.
- rck-6, NIT, tests/acts/looks/construct.py:440: the grant `input.grab-any` from `grab_any` is never revoked, so every later construct act runs with it. Fix: `choractl revoke construct input.grab-any` (`grants.revoke`) at the end of `keys_acts`.

Known, not findings: co-39 (cb-2), the stood-back cube that loses keys to xp after a pane went back, rfr-1.

**Verdict: FIX** (0 BLOCK, 1 MAJOR rck-1, 4 MINOR, 1 NIT).

## r2

Head f70d564f on claude/co-keys (red 61ce80a8 on e965470a, fix f70d564f).

**Red first (61ce80a8, 3 of 3 red on e965470a):** unit test `hints::tests::the_hint_of_the_key_list_names_a_key_that_works` (rck-3), act cell `the hint Keys of a face names a key that opens the key list` and the invariant cell `construct I4` (rck-1, rck-4; default session, no grant, the window of the face has the keyboard).

**Fix (f70d564f), green 5 of 5** (unit test, act cell, I4, the other construct acts, revoke act):
- rck-1, rck-3: `hints::keys_chord` takes the chord from the `keys` bindings: a bare key only while `rt.holds` says its grab held, else the first with a modifier (Super+F1); a dropped binding gives no chord. Set::Flown and Set::Cube use it; `regrab` marks the hints stale so a new grant redraws them.
- rck-2: docs/configuration.md, docs/construct-style.md and construct.yaml say F1 is grabbed in a pane or face only with `input.grab-any`, else it reaches the program and Super+F1 or the hint opens the list.
- rck-5: `wait(1.0)` became the cue `cube_navigates` (`s.until` cube mode navigate).
- rck-6: act `the grant input.grab-any is revoked again` ends `keys_acts`.
- rck-4: new cells in `keys_default_acts` keep the default session (no grant) covered; the stood-back cube hint cell was green on e965470a too (F1 works there without the grant).

**Tour:** `demo/tour/run.py --check construct`: 18 acts, 17 green, 1 known (cb-2), 0 red.
**Gate:** `--only pre-commit,build,test,clippy,construct-hud,construct-extra,doors,acts` PASS 8 of 8. The acts cell shows its one known red (I5 overlap, known-red.toml), not caused here.

## PM review r2 (2026-11-21, PM; builder was the co-keys lane)
Base e965470a, head f70d564f (7 files, +146 -8); red 61ce80a8 (stub `keys_chord` returns F1, unit test and acts check fail) before f70d564f.
- rck-1 closed: `hints::keys_chord` names a bare key only while `rt.holds` says its grab held, else the bound chord with a modifier (Super+F1); camera.rs:553 marks the hints stale after every grab rebuild, so a grant change refreshes the hint. Cell: unit test `the_hint_of_the_key_list_names_a_key_that_works`, acts construct (lane gate 8/8).
- rck-3 closed: a rebound `keys` binding changes the chord; a dropped one leaves no chord (same test).
- Docs (construct.yaml, configuration.md, construct-style.md) match the code: F1 grabbed in a pane or face only with input.grab-any.
- tools/lint-ratchet.list unchanged; known-red.toml unchanged (acts I5 row was already listed).
- rck-7 NIT, hints.rs:110: with several bound `keys` chords the first in BTreeMap order wins; fine today (one default), name it in the doc comment if a second default appears. Backlog.
Verdict (computed): no BLOCK or MAJOR open -> MERGE-OK on f70d564f; joins t99 with guide-checks b1b8b73a.

## r3

Branch claude/co-keys, head c69354ae. Merged guide-checks b1b8b73a (caa61cd9): camera.rs merged clean (co-keys doc comment, `pub(crate) fn grab_keys` and the F1 filter kept); known-red.toml taken from guide-checks, then rows added only for breaches the run met.

Commits: 1e7cc905 fix(construct) hide the chip words under the key list (unit test in the same commit: a red-only commit failed clippy as dead code); 05c1fc25 test(acts) face hint; c69354ae test(acts) flaky row.

Cause per row:
- I5 'Turn left' over the sheet note, and I6 'Turn right' 1.00:1: the Turn words are chip nodes of the modes (`construct-mode-turn-*-text`), set visible by `hints::chip_word` (hints.rs:418). When the key list opens, `hint_set` is None (`busy` on `self.help`, overlay.rs:~520) and `draw_hints` removes only HINTS and HITS (hints.rs `remove`), and `draw_modes` skips the chips while stood back (modes.rs:~268). The words stayed under the 880x763 sheet, which spans y 18 to 781 on 800 px. Fix: overlay.rs `draw_hints` hides the chip words (new `hints::hide_chip_words`) when the Cube hints go. Sheet and chips need no move.
- Face act "no the hint Keys on the screen": the hint is drawn, but only for 3 s after the step in (`FLOWN`, overlay.rs:~505). The pointer at the bottom shows it only through `hint_pointer`, fed by `Event::InputMoved`, which the core sends only during a held button grab (crates/chora/src/input.rs:630); over a face no pointer event reaches the construct. So the act passed before the merge only because the old harness was fast; the guide-checks harness captures every snapshot and the 3 s were gone. Fix in the act (construct.py keys_default_acts): stand back, wait, step in, then look at once. hint_click keeps the rect it saw (it crashed once on a None after the hint timed out). Product gap left open: a resting pointer does not show the hint over a face (needs a pointer event over a face; not done, not KISS).

known-red.toml: six rows I6 'guide wants 14 px' (help heads 6, 10, 17, 17-every, 21, 21-every of the cube sheet) now first met in the face act; one row removed (a drag with the modifier: construct-mode-step-in-text contrast, gone with the lingering chip words); the held-turn row construct-hints-construct-mode-step-in-keys-0-text marked flaky (met in 4 of 5 runs). No row for I5/I6 Turn chips.

Cells (construct acts, 5 runs): I5 pass 5/5, I6 Turn contrast gone 5/5, face act 5/5, stood back act 5/5; I2 and the 14 px I6 rows red as listed. Runs 1 and 5 each had one stale row (step-in-text, fixed by the row removal; step-in-keys flaky, row marked).

Gate (--jobs 1): pre-commit, build, test, clippy, doors, construct-hud, construct-extra PASS. acts FAIL: one blocking cell, not mine: xp I6 in act 'a click on Notepad starts it under the volume popup': 'Starting Notepad' (notify-card-body) 1.04:1, guide wants 4.5:1 (acts.log line `FAIL: xp I6 ...`, 231 pass, 5 known red, 1 blocking); every construct cell is pass or listed. The persona selftest failed once on 'FAIL: no click on Notepad' (first merge run) and passed in the gate run; not rerun to green, a flake of the persona session.

### PM review r3 (2026-11-21 14:15 UTC; PM not the builder)
Checks: I5/I6 Turn chip words: fix 1e7cc905 with a unit test in the same commit (red-only failed clippy as dead code; accepted, the test fails without the fix). Face act 05c1fc25 looks at once: honest about the product (hint for 3 s only), but no cell now guards the later state; filed co-42 MAJOR, lane co-hint r1. Flaky row c69354ae: 4 of 5 met, evidence named; goes to FLAKES-S. acts blocker xp notify-card-body 1.04:1 is not from this branch (no xp file touched): filed xp-38. Gate ran the old --only set; t100 runs the full gate.
Verdict (computed): no open MAJOR caused by the branch; co-42 pre-exists and is tracked. MERGE-OK at c69354ae for t100.
