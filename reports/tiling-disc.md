# tiling-disc

## r1

Lane: cloud, claude/tiling-disc from 230ea0e5, merged with origin/claude/autonomous-dev-session-3w-i33pg7 (5b34d4b1) at b4e762f6; head **8df47cc8**, pushed. Step 0 green (unshallow, cargo check, cargo build). Authors: Marcel Arpogaus, no attribution lines. fuzzel 1.9.2 on this host.

### Fixes (test commit, then fix commit; red lines are the output of the test commit)

| # | Cause (r5 row) | Test (red) | Fix (green) |
|---|---|---|---|
| 1 L1 | No focus key was visible: the list said Next window / Previous window low in Windows, Alt+Tab did nothing | 5abd20f8. `cargo test -p chora-tiling`: `test result: FAILED. 181 passed; 4 failed` (alt_tab_moves_the_focus..., the_focus_keys_are_the_first_row_in_plain_words, the_give_keys_have_a_group_of_their_own_near_the_top, the_rows_follow_the_order_of_a_person_not_of_the_keys). Cell tests/gate-tiling-focus-key.sh: `FAIL: after Alt+Tab the other window did not get the text: ''` | efe0aa9d. `Alt+Tab: focus_next` in tiling.yaml (dropped when `modifier: Alt`, where MOD+Tab is Alt+Tab). focus_next and focus_prev have one wording, "Move focus", so the first row of the list is `Move focus: Alt+Tab / Super+J / Super+K`; "Swap window" joins the two swap rows. The cell types into two foot windows: after Alt+Tab the other window gets the text, again Alt+Tab it goes back. |
| 2 L2 | No settings door | 8158e4d8. `FAILED. 184 passed; 4 failed` (super_s_opens_the_settings_in_an_editor, edit_settings_stands_beside_the_reload_key, the_settings_door_is_a_script_and_an_entry_of_the_flavor, a_missing_script_is_an_error_and_never_true). Cell tests/gate-tiling-settings.sh: `FAIL: the shipped bindings have no MOD+s: {run: chora-tiling --launch settings}` | d5151e38. `MOD+s: {run: chora-tiling --launch settings}` (label Edit settings), flavors/tiling/settings.sh (writes the defaults when the file is missing, then `chora-tiling --launch foot -e sh -c 'exec ${EDITOR:-vi} "$1"'`), entry flavors/tiling/applications/chora-settings.desktop (`Exec=chora-tiling --launch settings`), the row sits in System directly above "Read the settings again: Super+Ctrl+R". The cell asserts the binding, the label, the entry and its Exec line, then vi (EDITOR unset) and $EDITOR started with the file as argument, then the row order in the list. No settings GUI. |
| 3 T6 | The "Nothing found" row is ours and shows after Return; while typing, fuzzel draws an empty list | none (closed with evidence) | cdea0bb7. `fuzzel --help` (1.9.2) has no option for a text of an empty list, and the list is drawn by fuzzel alone, so the shipped config cannot show it. Backlog: **ti-31** in docs/known-issues.md (needs a feature of fuzzel or a launcher of our own). ti-12 now names the focus keys. |
| 4 T7 | The contract passed when no window changed look (a tag move); the give keys stood low in the list | 45742402. `python3 -I tests/persona/test_check.py`: `AssertionError: 0 != 1 : task T7 pass` (tag move inside tiling); cargo: `FAILED. 185 passed; 4 failed` | 4c239af4. T7 fails with `no window changed look: every window has the owner it had at the begin` unless a window present at the begin has another owner at the end. The give row is the group "Other looks" (`Super+Alt+1-4: Give window to a look`; the keys are 1-4, not 1-6) as the first group, at y 87 of the panel, Windows with Move focus right below at y 139 (the panel needs TOP 84, not 88, to keep both columns in 696 px). |
| 5 harness | Capture folder of a tiling+xp session missing after `round.sh stop` | 7f49752e. tests/gate-persona-keep.sh: `FAIL: stop of a session that kept nothing said ok: stopped gone` | 9f84d3e4. I could not reproduce the missing folder (a tiling+xp session with two actions kept shots, calls.log, evidence). The causes it can have: session.sh copied the captures only after it had stopped all processes, `--stop` waited 10 s, and `round.sh stop` printed "stopped" without looking. Now the session copies first and again at the end, `--stop` waits 40 s, `round.sh stop` waits for the folder (ROUND_STOP_WAIT, default 30 s), lists it and fails naming the folder when it stays empty. Listing after `round.sh stop` of a tiling+xp session: `actions.jsonl breaches.jsonl calls.log evidence shots`. |
| 5b harness | Super+s (and any key that runs `chora-tiling --launch settings`) found no settings.sh in a persona session: the snapshot binary sits in /tmp | bab7a89d. `FAIL: the core of a tiling session has no CHORA_FLAVOR_DIR of the checkout, so Super+s finds no settings.sh:` | a4badd8e. session.sh exports CHORA_FLAVOR_DIR=flavors/tiling for a tiling session. (A small step into session.sh beyond round.sh; needed to drive L2.) |

Two test commits of mine fixed my own tests: 4934d916 (the keep cell left its tiling+xp core alive when it failed, and the gate refuses to start with a chora --headless of an earlier run alive) and 8df47cc8 (tests/test_acts.py held the old T7: the same scene before and after passed; the cell `acts` was red on it, my cause).

Also changed: tests/gate-tiling-keys.sh and gate-tiling-power.sh look for "Move focus", no longer "Next window" (list_open). New cell `tiling-disc` in tests/merge-gate.sh (focus-key, settings, persona-keep).

### Counts
Every new cell 5/5 (a loop of 5 runs each): gate-tiling-focus-key 5/5, gate-tiling-settings 5/5, gate-persona-keep 5/5; also `python3 -I tests/persona/test_check.py` 5/5 (50 tests) and `cargo test -p chora-tiling` 5/5 (189 tests).

### The contracts driven by script steps (act.py and check.py end, tiling+xp session, snapshot binaries of 9f84d3e4)
```
L1  begin; Super+Return x2; Alt+Tab; Super+Shift+c; end L1   -> task L1 pass
L2  begin; Super+s; type :%s/gap_outer: 12/gap_outer: 20/ Return; :wq Return; Super+Ctrl+r; end L2
                                                              -> task L2 pass   (gap_outer: 20 in config/chora/chora-tiling.yaml)
T7  begin; Super+Shift+2 (tag move); end T7  -> task T7 FAIL: no window changed look: every window has the owner it had at the begin
T7  begin; Super+Alt+2 (give to xp, window open at the begin); end T7  -> task T7 pass
```

### Gate: tests/merge-gate.sh --jobs 2 (no --only), head 8df47cc8, with fresh origin/claude/autonomous-dev-session-3w-i33pg7
The gate stops at its first red, so it ran in four parts (/tmp/gate.log, gate2..4.log); an earlier run stopped on `acts` (my T7 test, fixed in 8df47cc8) and one on a leaked core of my own cell (fixed in 4934d916).
- Full run: red-first, identity, pre-commit, gate-lib, ci-check, build, test, clippy, 2d-guard, package-*, xp, matrix, virtual, xp-faces, xp-paperclip, xp-question, xp-chat-idle, xp-helper, xp-notepad, hud, layout, tiling-audit, tiling-look, tiling-empty, chat-keys, doors, acts, last-words, cube, pull, placed, construct-hud, construct-extra, out, winit, wm-pane, pane-keys, walk: 38 PASS, xp-fields SKIP (host), then **keys FAIL**.
- The rest with `--only` (the steps after the stop): pixels, mc-paths, mc-column, mc-handover, voice-acp, voice-events, agent, tiling-keys-more, tiling-system, handover, mc-scale, watchdog-starved, tiling-windows, tiling-bar, tiling-theme, tiling-disc, demo PASS; mc-sofa, launcher-click SKIP (host, fuzzel); **xp-run, mc-host, e2e FAIL**.

Reds I did not cause:
- keys: `tests/gate-construct-keys-persona.sh:88  FAIL: the Foot row of Programs started no foot`. The same line fails on a clean worktree of origin/claude/autonomous-dev-session-3w-i33pg7 (5b34d4b1, own build). fuzzel 1.9.2 click on a row (ti-1 class).
- xp-run: `FAIL: the Run box does not show` (tests/gate-xp-run.sh:86, real fuzzel 1.9.2; my diff touches no xp file).
- mc-host: `FAIL: dark: the field gets no keyboard after the click` (light passed in the same run; my diff touches no mc file).
- e2e: `FAIL: a click on the Zebra row did not start zebra`, `scripts failed: gate-tiling-apps-click` (fuzzel 1.9.2, ti-1; the same gate does not run need_fuzzel 1.11). Nothing was rerun to green.

### Open for the PM
- ti-31 is my number for the T6 note; check it against the ledger.
- Alt+Tab is also the switcher of xp. In a tiling+xp session one key has one holder; I did not test which look gets Alt+Tab there (the gate cells handover/matrix passed).
- The R5 persona rerun (L1, L2, T7, T6) is for the next round; T6 stays red by fuzzel.

## Review r1

Reviewer: cloud lane, head 8df47cc8 vs base 5b34d4b1. Step 0 green (unshallow, cargo check).

Verdict: **MERGE-OK** (0 BLOCK, 0 MAJOR, 1 MINOR, 0 NIT).

Checks:
1. Red first: confirmed on each test commit. 5abd20f8 cargo FAILED (alt_tab..., focus keys first row, give group); 8158e4d8 FAILED (super_s_opens_the_settings..., edit_settings_stands_beside_the_reload_key, missing script); 45742402 python test_check FAILED (1) and cargo FAILED; 7f49752e gate-persona-keep `FAIL: stop of a session that kept nothing said ok: stopped gone`; bab7a89d `FAIL: the core of a tiling session has no CHORA_FLAVOR_DIR`. Each fix commit has its test commit as parent chain predecessor.
2. L1: Alt+Tab is a plain binding (config.rs drops it under `modifier: Alt`, test covers); client grabs and inhibitors run before bindings in input.rs, so no stray key reaches a client; gate-tiling-focus-key.sh 3/3 green; Super+Left/Right untouched.
3. L2: settings.sh passes the path as `$1` to `sh -c 'exec ${EDITOR:-vi} "$1"'` (no injection through the path); MOD+s Edit settings, row beside Super+Ctrl+R (tiling.yaml:186 reload); gate-tiling-settings.sh 3/3 rc=0.
4. T7: check.py only adds a failure when no window changed owner; the lost/stray line is unchanged; no other contract touched; test_check.py 50 tests OK.
5. Harness: gate-persona-keep.sh 3/3 green at head, no new sleep in merge-gate.sh (one `tiling-disc` row added, no known-red row); session.sh copies captures before and after the stop.
6. cargo test -p chora-tiling: 189 passed; clippy --all-targets clean.

Findings:
- rtd-1, MINOR, tests/persona/round.sh:121, when `session.sh --stop` fails after 40 s, round.sh only prints a line and leaves the session running (brief: kill the session on failure); fix: after the failed --stop send `kill -KILL` to the PERSONA_SESSION_PID of the env file.

## Train116 (PM, 11-23 14:48)
- train116 (sim-long-r2 + tiling-disc 8df47cc8) red at launcher-click: "FAIL: tiling: a click on the Mid row of the launcher did not start mid (alpha, the first row: 0, the chat: 0)".
- Cause: tiling-disc adds flavors/tiling/applications/chora-settings.desktop ("Edit settings"); the row lands between Alpha and Mid, so the cell's fixed Mid row (tests/gate-launcher-click.sh:123, row tiling 35 15 400 259) now holds Edit settings.
- The lane gate could not see it: launcher-click needs fuzzel >= 1.11 and lane containers have 1.9.2 (sh-34).
- rtd-2, MAJOR (verdict FIX): tiling-disc breaks a gate cell; fix in r2 (keep the Mid row where the cell expects it, or the cell finds the row by name), red first. tiling-disc leaves the train; train117 runs without it.
