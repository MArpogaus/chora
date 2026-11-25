# tiling-doors

## r1

Lane: cloud, branch claude/tiling-doors-r1 from b02dbea9 (TILING-HONEST r1 head). Head **b1ba6743**. Step 0 green (`git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`). Plan and words as in reports/tiling-doors-plan.md "## r1"; Super+q stays unbound.

### Commits
| sha | what |
|---|---|
| ba7755f6 | red: tests only (ti-55, ti-56, ti-50) |
| eda266a5 | ti-55: bar word Looks, menu, theme rows renamed |
| 2b6c775a | ti-56: hint on the tiling chip for a swallowed chord |
| f69e7771 | ti-50 / rpt1-6: `tiling_order` replaces `helper_last` |
| 2e42ff0b | tour t2 expected words |
| c7d88215 | one `cannot_start` message (eprintln ratchet 47 -> 46) |
| b447a2cf | Looks gets the door padding; gates that read x positions follow |
| b1ba6743 | launcher-click gate clicks the third row |

### Red (ba7755f6), the failing lines
```
launcher::tests::the_helper_row_stands_first_and_the_flavor_rows_last
  left: ["Alpha", "Dark theme", "Edit settings", "Zed", "Chat with a helper"]
 right: ["Chat with a helper", "Alpha", "Zed", "Dark theme", "Edit settings"]
tests::an_unbound_chord_shows_the_hint_on_the_chip_ti_56      assertion `left == right` failed: []  (left 0, right 1)
tests::a_second_unbound_chord_within_ten_seconds_shows_no_hint_ti_56   once in ten seconds (left 0, right 1)
  (an_unbound_chord_shows_no_hint_while_the_key_list_shows_ti_56 passes red too: it guards the rule)
gate-tiling-apps.sh: FAIL: the first row is not the helper row "Chat with a helper (atlas)": alpha|Dark look|Edit settings|Light look|Move window to the construct desktop|...|Zebra|Chat with a helper (atlas)
doors tiling (checks beside the cells, tests/doors/tiling.sh):
tiling: the first row is "Dark look", not the helper          check tiling launcher-order fail
tiling: Super+q gave no hint on the chip                      check tiling stray-hint fail
tiling: Looks has no row "No other look is running":          check tiling looks-none fail
tiling: Looks has no row "Move window to the xp desktop":     check tiling looks-door fail
gate-tiling-apps-click.sh: green red and fixed (it computes the Zebra index from the rows the launcher gives a stand-in: 1 before, 2 after)
```
The doors cells are checks beside the cells (`check tiling looks-door pass`), as `launcher-order` is: the shared cell list of tests/doors/lib.sh is the same for every look. gate-doors.sh requires `stray-hint looks-none looks-door` for tiling.

### What changed
- **ti-55**: `custom/looks` after `custom/apps` (word Looks, no tooltip, `choractl run -- chora-tiling --launch looks`). `--launch looks` (new plugins/chora-tiling/src/looks.rs, `launch_looks` in main.rs): `clients.list` -> the running ones of xp, mc, construct -> rows `give::move_words` under the prompt `Move this window to `, or the one row `No other look is running`; a chosen row runs `give.sh` through `launch_line("give", [look])`. Words (en / de) in words.rs: `Move this window to ` / `Dieses Fenster verschieben nach `, `No other look is running` / `Kein anderer Look läuft`. Theme rows: Dark, Light, System theme (de: Dunkles Design, Helles Design, Design des Systems); the keyword `look` left the theme entries (the move rows keep it), the notice `Look switch not installed` became `Theme switch not installed`.
- **ti-56**: `act` calls `hint_unbound` for `Own::Swallow`: chip text `Super+Q does nothing here. Super+F1 shows all keys.` (de `Super+Q ist hier nicht belegt. Super+F1 zeigt alle Tasten.`), 3 s, once per 10 s, not while the key list shows; the key stays swallowed. `show_chip` became `show_words(word, time)`; the layout chip keeps 700 ms. The help key is the first binding of `System::Keys`. docs/configuration.md: the hint, and that Super+q stays unbound (`MOD+q: close` binds it).
- **ti-50**: `tiling_order`: helper row first, programs, then the flavor rows (command `chora-tiling --launch look|settings|give`; the shared Screenshot rows stay programs). docs sentence updated.

### Choices the brief did not spell out
1. words.rs `avoids_jargon` forbade the word "Look" in any word; the plan's German `Kein anderer Look läuft` needs it, so the test exempts `NoOtherLook` only.
2. Looks widened the left island by 52 px (12 of it is the door padding: `#custom-looks` joined the rules of the other doors in style.css, else the gap Apps-Looks-separator was 6 px, not 12 and gate-tiling-spacing failed). Gates that read fixed x positions followed: gate-tiling-bar-palette (pill of tag 1 at x 173), gate-tiling-bar (gap check x 545/557, separator scan to x 170), gate-tiling-bar-spec (clock scan from x 545), gate-launcher-click (the Mid row is the third row: y 230 in its scale). Looks is clicked at x 116.
3. eprintln ratchet: `launch_looks` needed three failure messages; the two `cannot start` lines of `launch` and the three new ones became one `cannot_start`, so the count fell 47 -> 46 and tools/lint-ratchet.list follows.
4. gate-tiling-apps: "Enter on the first row starts alpha" became "Down, Enter" (the first row is the helper now, which is the decision).

### Tour (fresh compositor per chapter, final build b1ba6743)
```
chapter tiling: 28 acts, 28 green, 0 inv, 0 known, 0 red, 0 todo, 0 unchecked
tour: record: 0 red, 0 broken
chapter handover: 11 acts, 11 green, 0 inv, 0 known, 0 red, 0 todo, 0 unchecked
tour: record: 0 red, 0 broken
```
Only t2's expected text changed: `... no key of a chord reaches it, and the chip says Super+F1 shows all keys`; its still (media 6) shows the chip `Super+D does nothing here. Super+F1 shows all keys.` Note for the tour reviewer: act 24 prints `known act 24 is green: drop known` (not from this lane); still 02-26-apps (media 7) has the helper row first.

### Full gate (final head b1ba6743), last 20 lines
```
launcher-click PASS    17s
voice-events PASS     1s
voice-acp  PASS    11s
agent      PASS    36s
tiling-keys-more PASS    39s
tiling-system PASS    13s
handover   PASS    23s
mc-scale   PASS    11s
watchdog-starved PASS    53s
tiling-windows PASS   158s
xp-run     PASS    12s
demo       PASS   200s
leak       PASS   310s
tiling-theme PASS    18s
tiling-disc PASS    70s
mc-host    PASS    19s
tiling-bar PASS   163s
e2e        PASS   264s
all steps passed; no --push, nothing is pushed
EXIT 0
```
No SKIP step. Three earlier runs of the gate stopped (fail-fast) on reds that this lane caused and fixed: lint-ratchet (eprintln 50 > 47), tiling-look (gate-tiling-bar-palette: pill pixel moved), launcher-click (Mid row index). `tools/sleep-ratchet` ok (sleep 779, sleep-all 930). `pgrep -fa 'chora --headless'` empty.

Foreign red seen once, not in the gate: the doors check `helper-beside-tiles` failed in my standalone runs of tests/doors/tiling.sh at b02dbea9 (before any change of this lane) and after: `tiling: closing the chat gave no width back: 854 -> 854`; it passed in all three gate runs (step `doors`). Looks like load timing; not touched here.

### Captures (media/2026-11-24/tiling-doors/)
1-bar-with-looks, 2-looks-menu-tiling-only, 3-hint-chip, 4-looks-menu-tiling-and-xp, 5-apps-list (real fuzzel, 1280x800 headless), 6-tour-t2-stray-hint, 7-tour-apps-list.

### Open for the PM
- The 3 s chip sits in the middle of the screen over the tiles; rth-4 (failed reload without chora-notify) can use `show_words` now.
- Persona round r9 should keep the ricer's T7 words unchanged (plan open question 2); the Looks word now sits at x 116 of the bar.

## Review r1

Reviewer: cloud, read-only on code, claude/tiling-doors-r1 head b1ba6743, base b02dbea9 (24 files, +513 -81). Step 0 green (`git fetch --unshallow`, `cargo check --workspace`). `cargo test -p chora-tiling -p chora-kit`: all ok (chora-tiling 328 passed). No merge gate run (the lane's gate is green: EXIT 0, no SKIP). `tools/sleep-ratchet` ok at 779 (base counted 778).

Checks:
1. Red first: ba7755f6 (tests only) comes before eda266a5, 2b6c775a, f69e7771. c7d88215 is a refactor (one `cannot_start`, eprintln 47 -> 46, which is true) but also renames a notice title without a test (rtd1-2). b447a2cf adds the Looks padding; its red was the existing gate-tiling-spacing (6 px gap). b1ba6743 changes only a test. No fix ships without a red test: no MAJOR here.
2. ti-55: `looks::running` takes xp, mc and construct from `clients.list` in a fixed order. The rows come from `give::move_words`, or there is the one row `No other look is running`. `chosen` matches whole rows only, so that row and any typed text choose nothing. A chosen row runs give.sh through `launch_line("give", [look])`, which is the path the Apps rows use. The key list closes first. The words in en and de match the plan, and the theme rows match it in both languages. The bar module has no tooltip and is styled like the other doors. Captures 2 and 4 show the prompt and the rows. The doors cells check one row with tiling only, and that the owner becomes xp with xp running.
3. ti-56: `act` routes `Own::Swallow` to `hint_unbound`. The hint shows for 3 s, then not again for 10 s, and never while the key list shows. The grab still swallows the key, which the doors check proves (the window hears no key). The help key is the first `System::Keys` binding, resolved through MOD, and the layout chip keeps 700 ms through `show_words`. POLS: a rebound help key is named, because the code reads the user's bindings. But no test proves it, and with no key bound to keys the hint names Super+F1, which is then a swallowed key itself (rtd1-3).
4. ti-50: `tiling_order` only runs when `run_ordered(.., ordered=true)`, which only tiling uses. mc (main.rs:460) and xp (launch.rs:43) call `run` with `false`, and the construct uses no ordering. Their order does not change. The Screenshot rows stay programs. `cargo test -p chora-kit` is ok.
5. Beyond the brief: `avoids_jargon` exempts `NoOtherLook` only. The moved x positions still assert what they asserted: the palette pill is still inside tag 1 (173); the gap probes 545/557 sit in the new gap; the separator and island-end scans start further right; the launcher-click y follows the row count. None of these checks is weaker. One new fixed sleep was added (rtd1-1). The lint ratchet 47 -> 46 is true.

Findings:
- rtd1-1 | MINOR | tests/gate-tiling-apps.sh:137 | a new fixed `sleep 0.5` between `key Down` and `key Return` uses up the sleep ratchet's slack (778 -> 779) | drop it (fuzzel reads keys in order) and run `tools/sleep-ratchet --update` to 778.
- rtd1-2 | MINOR | plugins/chora-tiling/src/theme.rs:731-735 | the refactor commit c7d88215 changes the notice title "Look switch not installed" with no test; the German "Theme-Umschalter" does not match the rows "Dunkles Design" and look.sh "Design nicht geändert" | write "Design-Umschalter nicht installiert" and add a unit test that pins the en and de titles.
- rtd1-3 | MINOR | plugins/chora-tiling/src/overlay.rs:197-200 | when no key is bound to keys, the hint falls back to "Super+F1", itself a swallowed chord; no test proves the hint names a rebound help key | without a keys binding say only the first sentence, and add a unit test with `MOD+slash: keys` only that expects Super+/ in the hint.
- rtd1-4 | MINOR | plugins/chora-tiling/src/main.rs:367-370 | if `clients.list` fails, `--launch looks` writes only to stderr, so a click on Looks shows nothing | show a notice as `tell_launch_error` does (or open the menu with the one row), not silence.
- rtd1-5 | COSMETIC | docs/configuration.md:1757 | "The looks set `look.theme`" still calls the theme rows looks, the ambiguity ti-55 removes | "The theme rows set `look.theme`".

Verdict: **MERGE-OK** (0 BLOCK, 0 MAJOR, 4 MINOR, 1 COSMETIC).

## r2

Lane: cloud, branch claude/tiling-doors-r2 from 721be592 (session head t134). Head **fa676de8**. Step 0 green (`git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`). Decisions of the PM (11-25) as briefed: ti-55 the bar word names where it leads, ti-56 HINT_TIME 3 s -> 8 s, L2 compares the value of gap_inner. Nothing reopened.

### Commits
| sha | what |
|---|---|
| b0ec5f10 | red: tests only (ti-55, ti-56, L2) |
| 9d416c21 | ti-56: HINT_TIME 8 s, docs/configuration.md |
| 2ae1e3ff | L2: verify_reloaded reads the value of gap_inner (disk at begin vs disk now, and the running tiling) |
| 3142aea3 | ti-55: custom/looks word "Looks" / "Looks: xp" / "Looks: xp, mc", the T7 goal text |
| 45914b6b | the Looks word without a lint allow (lint-ratchet 361 > 360 in my first gate run) |
| a8d2ce57 | a launch connects under a name of its own (found by the first gate run, see below) |
| fa676de8 | tests/test_acts.py planted the old digest pair for L2; now the three values |

### Red (b0ec5f10), the failing lines
```
(a) tests/doors/tiling.sh (cells beside the checks)
tiling: tiling only: the word of custom/looks is "", not "Looks"            check tiling looks-word-none fail
tiling: tiling and xp: the word of custom/looks is "", not "Looks: xp"      check tiling looks-word fail
(b) cargo test -p chora-tiling the_hint_shows_7_seconds_after_the_key_and_is_gone_at_9_ti_56
    FAILED: the hint is gone 7 s after the key
    (end_chip(key + 7 s): chip_until is None with HINT_TIME 3 s; the clock is the `now` that end_chip takes, no sleep)
(c) python3 -I tests/persona/test_check.py
    FAIL: test_the_editor_opened_without_a_change_is_red     AssertionError: 0 != 1 : task L2 pass
    FAIL: test_a_changed_file_with_the_same_gap_is_red       AssertionError: 0 != 1 : task L2 pass
    FAIL: test_a_gap_written_but_not_reloaded_is_red         AssertionError: 0 != 1 : task L2 pass
    (test_a_gap_changed_and_reloaded_passes: 12 -> 20 and reload, passes red and fixed)
```
After the fixes: doors `check tiling looks-word-none pass`, `looks-word pass`; the 4 `ti_56` tests ok; `test_check.py` 61 tests OK; `test_acts.py` 178 OK.

### What changed
- **ti-55**: `custom/looks` runs `waybar/looks.sh` every 2 s (`interval: 2`, no signal): `chora-tiling --launch looks-word` prints `{"text": ...}` from `clients.list` through `looks::running` and the new `looks::word` (xp, mc, construct in that order); the word is `Looks` when none runs or the core does not answer. A look that joins or leaves shows within 2 s. docs/configuration.md says so. The persona T7 goal is not stored in the repo (as r9 said); it is now in tests/persona/README.md under "Tiling goals": "Send the terminal over to the xp look (the other desktop that is running)".
- **ti-56**: `HINT_TIME` 8 s. Once per 10 s and not while the key list shows are unchanged.
- **L2**: `check.py` reads gap_inner from `config/chora/chora-tiling.yaml` (12, the default, while the file is absent or lacks the line); the begin value is kept with `begin`; the running value is `get_layout` of tiling. FAIL when the disk value equals the begin value ("gap_inner is still 12 on disk"), or the running value is not the disk value ("the running config reports gap_inner 12, not 20 after the reload"), or the look does not hold the focus. `has_config`: a tiling look in the session or a non-empty tiling config; a session with neither says "not measured (no config)". The digest and `config_digest` are gone; the README of tests/persona says so.

### A defect the first gate run found (mine, fixed)
The first full gate failed the `doors` step on `check tiling looks-none` ("Looks has no row ..."): the word module asks the core every 2 s under the client name `chora-tiling.launch`, the name every launch uses, and two connections of one name lose to each other. Measured on a tiling+xp session, six reads at once: 69 of 180 said `Looks: xp`, 111 said `Looks`; a click on Looks can meet a read and list no rows. `launch_conn` now takes the process id into the name (`chora-tiling.launch-PID`; nothing else names it). Cell `looks-word-parallel` (six reads at once, all `Looks: xp`): red with the old name (`six words at once ...: 3 Looks|3 Looks: xp|`), pass after. It passed standalone three times before the fix, so the load of the gate (two jobs) is what showed it.

### Tour (fresh compositor per chapter, build fa676de8 for tiling and handover)
```
chapter tiling: 28 acts, 28 green, 0 inv, 0 known, 0 red, 0 todo, 0 unchecked
tour: record: 0 red, 0 broken
chapter handover: 11 acts, 11 green, 0 inv, 0 known, 0 red, 0 todo, 0 unchecked
tour: record: 0 red, 0 broken
```
No expected bar word changed: no row of either chapter names the word of Looks. The `known act 24 is green: drop known` line is not from this lane (as in r1).

### Captures (media/2026-11-25/tiling-doors-r2/, 1280x800, persona sessions)
- `bar-tiling-only.png`: the bar word `Looks`.
- `bar-tiling-plus-xp.png`: `Looks: xp`.
- `hint-at-7s.png`: the hint chip `Super+Q does nothing here. Super+F1 shows all keys.` still on screen 6.2 s after Super+q (`hint-actions.jsonl`: key at 1795583794.441, look at 1795583800.678). A look 9.3 s after the key was taken too (not copied).
Note for the reviewer: `Looks: xp` widens the left island by about 25 px; the island ends 1 px before the clock (x 563 and 564). With all three other looks (`Looks: xp, mc, construct`) it would reach into the clock; not changed here (the decision names the word, not a limit).

### Full gate (final head fa676de8), last 20 lines
```
launcher-click PASS    15s
voice-acp  PASS    11s
voice-events PASS     2s
agent      PASS    36s
tiling-keys-more PASS    39s
tiling-system PASS    13s
handover   PASS    23s
mc-scale   PASS    10s
watchdog-starved PASS    48s
tiling-windows PASS   161s
xp-run     PASS    12s
demo       PASS   188s
tiling-bar PASS   156s
tiling-theme PASS    18s
leak       PASS   462s
mc-host    PASS    19s
tiling-disc PASS    70s
e2e        PASS   265s
all steps passed; no --push, nothing is pushed
EXIT 0
```
Three earlier gate runs stopped on: lint-ratchet (allow 361 > 360, fixed in 45914b6b), doors looks-none (the name collision, a8d2ce57), acts selftest (test_acts.py planted the digest pair, fa676de8). `tools/sleep-ratchet` ok (sleep 778, sleep-all 929), `tools/size-ratchet` ok (0 files over 1500), `tools/lint-ratchet` ok (allow 360, no-reason 0, eprintln 46). `pgrep -fa 'chora --headless'` empty.

### Not done / open
- The persona round (r10) has not run: whether a persona now reads "Looks: xp" as the door is not measured here.
- The T7 goal wording lives in the README only; the driver brief must copy it.
- Out of scope and untouched: rth-*, rtd-*, rv9j-*, ti-31, ti-49, Super+q binding, I6, the L1 contract. rtd1-4 (a click on Looks with a failing `clients.list` shows nothing) still stands.

## Review r2

Reviewer lane (cloud, read-only on code): claude/tiling-doors-r2 head fa676de8, base 721be592. Step 0 green (`git fetch --unshallow`, `cargo check --workspace`). Ran: `cargo test -p chora-tiling` (205 ok, the 4 `ti_56` and the `ti_55` word test ok), `python3 -I tests/persona/test_check.py` (61 OK), `python3 -I tests/test_acts.py` (OK). No merge gate (the lane ran it, EXIT 0).

Checks:
1) Red first: b0ec5f10 (tests only) precedes 9d416c21, 2ae1e3ff, 3142aea3. 45914b6b changes no behaviour (println -> writeln, lint). a8d2ce57 (launch name with pid) brings its only test, `looks-word-parallel`, in the same commit: rtd2-1.
2) ti-55: `Looks` / `Looks: xp` / `Looks: xp, mc` from `looks::word`, `interval: 2`, no core -> `Looks` (connect error -> Null, and the `|| echo` of looks.sh). Bar with xp at 1280: the left island ends at the clock island (capture), no overlap; two or three looks not captured (rtd2-6). The click leads to "Move this window to"; whether a beginner reads "Looks: xp" as the door is the r10 persona's to measure.
3) `chora-tiling.launch` is named nowhere else (core, tests, docs, chora-kit). The core's per-name maps (graces, emits) do not grow: emits are pruned per window, graces only with resume. The leak gate starts chora-tiling without waybar, so its asserts are unchanged and the poll is not in it (rtd2-5). No core log line per connect.
4) ti-56: HINT_TIME 8 s; `hint_unbound` unchanged apart from its comment (once per 10 s, not while the key list shows); the new test passes `key + 7 s` / `key + 9 s` to `end_chip`, no sleep.
5) L2: FAIL when disk value == begin value, or running (`get_layout` gap_inner) != disk; digest gone; a comment-only change is red (test). In a real tiling session `Session.looks` holds tiling, so a missing file is measured (red), not "not measured"; that branch has no test (rtd2-2).
6) tests/gate-doors.sh only adds names; test_acts.py's L2 test is stricter (a third case); doors/tiling.sh adds cells, no new sleep (`until_ok`). docs/configuration.md matches (8 s; 2 s read); tests/persona/README.md line 141 does not (rtd2-3).

Findings:
- rtd2-1 | MAJOR | plugins/chora-tiling/src/main.rs:286 (a8d2ce57) | the launch-name fix lands with its only test (`looks-word-parallel`) in the same commit, no red before it; and the cell is no reliable red (the lane saw it pass 3x standalone with the old name) | add a red commit with a deterministic test (e.g. a unit test that two `launch_conn` names differ, or a doors cell that holds a launch connection open while the word is read), then the fix on top.
- rtd2-2 | MINOR | tests/persona/test_check.py:322 | FakeSession has no `looks`, so the branch of `has_config` that makes a tiling session measured is untested, and `test_a_missing_config_is_not_measured` models a run labelled tiling as "not measured", exit 0 | give NoConfig's run `looks = ["tiling"]` and add: tiling session, no file -> L2 FAIL "gap_inner is still 12".
- rtd2-3 | MINOR | tests/persona/README.md:141 | says L2 is not measured when the session has no non-empty config; now it is: no tiling look and no non-empty chora-tiling.yaml, and an xp-only session (xp.yaml) is never measured | say that in the README.
- rtd2-4 | MINOR | tests/persona/check.py:847 | GAP_LINE misses `gap_inner: 20  # wider` and `gap_inner: "20"` and reads the default 12: a real change is a false red | allow a trailing comment and quotes (or read look.gap_inner with a YAML parse).
- rtd2-5 | MINOR | flavors/tiling/waybar/config.jsonc:105 | every 2 s a launch connects under a new name, so the core sends client.connected and client.disconnected to every subscriber (the swallow runtime of each plugin, chora-assistant) for the whole session; no gate runs the module for long | drive the word from tiling on a look's client events (waybar signal), or let the leak gate run looks.sh beside its rounds.
- rtd2-6 | MINOR | flavors/tiling/waybar/config.jsonc:21 | `Looks: xp, mc` at 1280 is not captured; with xp alone the left island already meets the clock island, so two looks push the clock off center (waybar fixed-center pushes; overlap not expected but not measured) | capture tiling+xp+mc at 1280.
- rtd2-7 | NIT | plugins/chora-tiling/src/overlay.rs:185 | doc comment line of 131 chars | rewrap.

Counts: BLOCK 0, MAJOR 1, MINOR 5, NIT 1. Verdict: **FIX** (rtd2-1).

## r3

Head 35e31662 on claude/tiling-doors-r3 (history kept, no force push).

- rtd2-1: A revert of the launch name (the name function ignores the pid), B test `two_launches_of_one_machine_connect_under_different_names`, C fix (name `chora-tiling.launch-<pid>`). Red: `test tests::two_launches_of_one_machine_connect_under_different_names ... FAILED` (assertion `left != right` failed). Green after C: cargo test -p chora-tiling 206 passed.
- rtd2-4: D test `test_a_comment_or_quotes_after_the_value_are_read`, red: `AssertionError: '12' != '20'` (FAILED failures=1). E GAP_LINE allows quotes and a trailing comment; test_check.py 62 OK.
- Full gate (--jobs 2): all steps passed, EXIT 0 (last lines: e2e PASS 278s; "all steps passed; no --push, nothing is pushed"). Ratchets sleep/size/lint ok. No `chora --headless` left.
