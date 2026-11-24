# tiling-reach (RC 2: ti-47 double top bar, ti-48 handover not findable)

## r1

Lane: cloud, branch claude/tiling-reach from cae62f50, head **4e257c060f5e3a4548c6f071aa51385b1e7e0810**. Step 0: `git fetch --unshallow`, `cargo check --workspace` green. One round.

### Commits (cae62f50..4e257c06)
| sha | what |
|---|---|
| 7fb9db06 | test: tests/gate-tiling-one-bar.sh (RED for ti-47), wired into the `tiling-bar` step of merge-gate.sh |
| f358771c | fix ti-47, first form: waybar.sh holds `panel.lock`; session.sh stops starting its own panel |
| 451e6a83 | test: key list and launcher words for ti-48 (RED: 4 unit tests, launcher rows of gate-tiling-apps.sh) |
| 64b60b8a | fix ti-48: `kit::give::move_words`, `give.sh`, one launcher row per other desktop, three key list rows |
| f0557aa3 | fix ti-48: the key list keeps ONE row (three rows broke the 800 px fit, see below) |
| d6e3f6ed | test: the title test names the look from the keys (tools/cut-check copies row `to != NAME`) |
| 0591856d | test: doors cell keys-names reads `Super+Alt+2-4` for tiling |
| 689bff45 | fix ti-47, final form: the session script owns the bar again (see "ti-47" below); launcher-click row y 207 |
| 4e257c06 | test: gate-tiling-apps-click clicks the second row (Aa alpha / Ab Zebra, measured y) |

### ti-47: the cause, and which sessions were affected
Cause: since a4badd8e `session.sh` exports `CHORA_FLAVOR_DIR` to the core, so `chora-tiling` (no `--no-companions`) finds `waybar/waybar.sh` and starts its **own** panel (`theme::start_companions`) while `session.sh` still started a second `waybar.sh` as its companion: two waybars on one config (`/proc` shows both on `chora-tiling-waybar.json` of the session runtime). Before a4badd8e the core found no script outside a checkout, so only the script's bar ran (r5: one bar).
- **Harness: affected in every tiling session** since a4badd8e (r6: all three personas).
- **Real session: not affected by `chora-tiling` alone** (cell: 1 bar for 10 s). Possible with the units: `chora-tiling-waybar.service` starts after `chora-tiling.service` and ran `waybar.sh` without asking whether the plugin's panel already ran; the plugin's check (`panel.pid` names a running waybar.sh) only covers the other order. The cell starts the unit's command beside the plugin's panel and shows two bars (red). So a login session could draw two bars depending on the start order; I did not run systemd here.
Fix (one owner): (a) the product: `waybar.sh` takes `flock -n` on `chora-tiling/panel.lock` of the runtime directory before it writes `panel.pid`; a second start (unit beside plugin) says so on stderr and ends with 0, the lock stays held while waybar runs. (b) the harness: the tiling of a persona session starts with `--no-companions`, the script starts the panel and the wallpaper with the session environment (first form, 689bff45 undid dropping them: the plugin's panel got the core environment, not `XDG_DATA_DIRS`/`XDG_CACHE_HOME` of the cell, and launcher-click lost its rows).

Red (7fb9db06 on cae62f50 code, `tests/gate-tiling-one-bar.sh`):
```
ok: chora-tiling alone: one waybar (1) for 10 s
FAIL: chora-tiling and the panel unit: 2 waybar processes on one config: 24481 waybar --config /tmp/chora-hl.GmTnuy/chora-tiling-waybar.json ...;24746 waybar --config ... (same)
FAIL: persona session.sh tiling: 2 waybar processes on one config: 25386 waybar --config /tmp/chora-hl.fOIBvH/chora-tiling-waybar.json ...;25549 ...
```
Green on the head: `ok: chora-tiling alone: one waybar (1) for 10 s`, `ok: chora-tiling and the panel unit: one waybar (1) for 10 s`, `ok: product session: none left after the stop`, `ok: persona session.sh tiling: one waybar (1) for 10 s`, `ok: persona session: none left after the stop`. The cell counts by the runtime directory in `/proc/PID/environ` (`session_pids -x waybar`), samples 20 times over 10 s and fails on any sample of two; no new `sleep` line (`until_ok`, `pace` of gate-lib).

### ti-48: the words
- Launcher (Apps list): one row per other desktop, `chora-give-{xp,mc,construct}.desktop` in flavors/tiling/applications: **Move window to the xp desktop**, **...the media center desktop**, **...the construct desktop** (German: `Fenster auf den xp-Desktop verschieben` and so on, `Name[de]`); `Exec=chora-tiling --launch give xp`, which runs the new `flavors/tiling/give.sh` (`choractl give TO`, a notice `Window not moved` when it refuses). The words come from `chora_kit::give::move_words`; a unit test ties the .desktop names to it. No row for tiling (a window is with it already).
- Key list: the group is now titled **Other desktops: xp, media center, construct** (was "Other looks") with one row `Super+Alt+2-4 | Move window to another desktop` (was `Super+Alt+1-4 | Give window to a look`; German `Andere Desktops: xp, Mediencenter, Construct`, `Fenster zu einem anderen Desktop`). The word "look" is gone from both; the key is on its cap.
- Why one row, not three: three rows (64b60b8a) made the list 53 lines and 3 existing tests went red at 800 px (`rows_keep_a_pitch_of_the_grid_and_caps_leave_a_gap`: caps 2 px apart; `no_box_of_the_list_overlaps_another` in German; `the_titles_and_the_header_read_at_7_to_1`): the grid holds 26 lines per column at pitch 22 in 696 px, a split of 53 lines gives 27. So the names stand in the title, not in rows. German "Fenster auf den Mediencenter-Desktop verschieben" (350 px) also overlapped the second column.
- Docs: docs/configuration.md (key list groups, launcher entries, file tree with give.sh), docs/api.md (keysheet words). Other looks keep `Super+Alt+1-4` / "Give window to a look" (xp, mc help; construct help rows): not touched, said here.
- Proof the launcher row works: in a tiling+xp persona session a window of tiling, `chora-tiling --launch give xp` -> `choractl query kind:surface` owner `tiling` -> `xp` (rc 0).

Red (451e6a83 on f358771c code): `keys::tests::the_give_keys_have_a_group_of_their_own_above_the_windows`, `...the_give_keys_and_the_focus_keys_are_both_in_the_top_part`, `...the_rows_follow_the_order_of_a_person_not_of_the_keys`, `theme::tests::a_launcher_row_per_other_desktop_moves_the_window_in_the_words_of_the_key_list` FAILED; `tests/gate-tiling-apps.sh`: `FAIL: no launcher row "Move window to the xp desktop": alpha|Dark look|Edit settings|Light look|Screenshot|Screenshot of an area|System look|Zebra|Chat with a helper`. Green on the head: gate-tiling-apps ok (`the launcher moves a window to the xp, media center and construct desktop in the words of the key list`), the unit tests above (the key list ones re-pointed to the one-row form in f0557aa3, with a new test that the title names every other desktop).

### Gate (tests/merge-gate.sh, no --only, --jobs 2) on 4e257c06
`all steps passed; no --push, nothing is pushed`, 65 steps PASS, 0 FAIL, no foreign red. Runs before it, all my own: (1) `test` red: 3 layout tests (the three-row form, above); (2) `ci-check` red: copies row `to != NAME` (d6e3f6ed); (3) `doors` red: tiling keys-names `missing` (0591856d); (4) `launcher-click` red: the Apps list shifted the Mid row (y 219 -> 207; and the session.sh form above, 689bff45); (5) `e2e` red: gate-tiling-apps-click, Zebra off the nine rows (4e257c06). One transient: `tiling-audit` failed once (`fullscreen paper-2560x1600@2: the bar does not hide over the fullscreen window within 20 s`) under load on the head of that run; passed alone (282 s) and in the three runs after, not touched by the diff (the gate starts `--no-companions` and one waybar.sh).

### Captures (media/2026-11-24/tiling-reach-r1/)
- bar-one.png: tiling+xp persona session, 1280x800 (capture 800x500): ONE top bar (`pgrep -x waybar` of the session = 1), a terminal, the xp taskbar at the foot.
- keys-other-desktops.png: the key list: the title `Other desktops: xp, media center, construct` and the row `Super+Alt+2-4 | Move window to another desktop`.
- launcher-move-rows.png: the Apps list of tiling (cell data: Aa Alpha, Ab Mid, Ac Zebra) with the rows `Move window to the construct / media center / xp desktop` (nine rows, the most fuzzel.ini shows).

### Open for the PM
1. ti-47: closed by the cell (3 cases) and the lock; a login session's two-bar race was shown only as a command beside the plugin, not under systemd.
2. ti-48: closed as to words (launcher rows exact, key list title + row); the persona T7 should be re-run once on this head (ricer, tiling+xp): the launcher row is the path for a person who types "xp" or "move" into the Apps list; the key list names xp in its title.
3. The give wording of xp, mc and construct (`Give window to ...`, `Super+Alt+1-4`) is unchanged; one wording for all four looks would fit the key list of each (their grids are not tiling's).
4. The apps-click and launcher-click cells compute a click y from a constant (list of nine rows, 37 px pitch); a flavor that adds rows no longer moves them.

## Review r1

Seeded review (sonnet, rc 1, 2) of claude/tiling-reach at **4e257c06** against base cae62f50 (18 files, +339 -29). Read-only on code; one round. Note: origin/claude/tiling-reach has since moved to ad0093ba (a merge of claude/autonomous-dev-session-3w-i33pg7); that merge was not reviewed.

**Verdict: MERGE-OK.** BLOCK 0, MAJOR 0, MINOR 8.

### Checks
1. Red first, each cell run on the red commit, green at 4e257c06.
   - ti-47, 7fb9db06 `tests/gate-tiling-one-bar.sh`: `ok: chora-tiling alone: one waybar (1) for 10 s`, then `FAIL: chora-tiling and the panel unit: 2 waybar processes on one config: 19003 19323`; persona only: `FAIL: persona session.sh tiling: 2 waybar processes on one config: 24779 24941`. Head, 3 runs, each rc 0: `ok` x5 (product 1 bar, unit 1 bar, product none left, persona 1 bar, persona none left).
   - ti-48, 451e6a83: `cargo test -p chora-tiling -p chora-kit`: `keys::tests::the_give_keys_and_the_focus_keys_are_both_in_the_top_part`, `...the_give_keys_have_a_group_of_their_own_above_the_windows`, `...the_rows_follow_the_order_of_a_person_not_of_the_keys`, `theme::tests::a_launcher_row_per_other_desktop_moves_the_window_in_the_words_of_the_key_list` FAILED (188 passed, 4 failed); `tests/gate-tiling-apps.sh`: `FAIL: no launcher row "Move window to the xp desktop": alpha|Dark look|Edit settings|Light look|Screenshot|Screenshot of an area|System look|Zebra|Chat with a helper`. Head: all test results ok (no FAILED), gate-tiling-apps rc 0 incl. `ok: the launcher moves a window to the xp, media center and construct desktop ...`.
2. ti-47 exact. The lock is `exec 9>panel.lock; flock -n 9` before `panel.pid` is written, in the script's own shell; waybar is its background child and inherits fd 9 (checked: waybar and its exec-module child show `/proc/PID/fd/9 -> .../chora-tiling/panel.lock`), so the lock lasts as long as the script and waybar. Second start (fake waybar, scratch runtime): stderr `waybar.sh: the panel of this session already runs; this start ends`, rc 0, panel.pid untouched. A kill -9 of script and waybar with no surviving child leaves no stale lock (flock dies with the fd). Both paths go through it: `theme::start_companions` runs `waybar.sh` (sh or dbus-run-session), the unit `chora-tiling-waybar.service` runs `/bin/sh .../waybar.sh`. The cell counts by `XDG_RUNTIME_DIR` in `/proc/PID/environ` (`session_pids`), so per session, not per machine; samples 20x over 10 s.
3. ti-48 exact. Launcher names come from three `.desktop` files, key list row from `keysheet::move_row`, title from `keys::title`, German from `move_words`/`desktop_name`; the rows tie by tests (see rtr-4). `give.sh` refuses honestly: with no window `chora-tiling --launch give xp` -> stderr `give.sh: choractl: no focused window`, rc 1, notify-send "Window not moved" (de: "Fenster nicht verschoben"); `give.sh bogus` -> usage, rc 2. German fit: the key list layout tests (no overlap, pitch, 800 px, English and German) are green on the head. docs/configuration.md and docs/api.md match the code except rtr-8. The launcher is a layer-shell client, not a surface node: with it open the scene shows only the window (`kind:surface`), so a give from the row sees the window as the focused one; I did not click the give row end to end (my probe entry did not start; see rtr-6).
4. Harness. `--no-companions` is only in `startup_of` of tests/persona/session.sh; no product file starts chora-tiling with it. The product path (`chora-tiling` alone) keeps its companions: cell "chora-tiling alone: one waybar". The y values of gate-launcher-click (207) and gate-tiling-apps-click (first_row_y 294, row_h 37) are constants of a full list of nine rows (rtr-7).
5. `cargo test -p chora-tiling -p chora-kit` green; `tests/gate-tiling-apps.sh` rc 0; `tests/doors/tiling.sh` rc 0 (every cell and check pass).
6. Scope: tiling flavor and plugin, `chora-kit` give/keysheet, the named tests, `docs/api.md`, `docs/configuration.md`; merge-gate.sh: the one-bar wiring only. Nothing else.

### Findings
- rtr-1, MINOR, flavors/tiling/waybar/waybar.sh:40-45 (and the `waybar ... &` line): waybar's children inherit the lock fd, so a long-lived child that outlives a crashed waybar keeps `panel.lock` and the next start ends with 0 and "already runs" and no bar (reproduced with a fake waybar whose exec child sleeps; the shipped config's exec modules are short polls, so latent). Fix: start waybar with `9>&-` so the lock lives with the script (which ends with waybar), or check that the holder is a running waybar.sh.
- rtr-2, MINOR, flavors/tiling/chora-tiling-waybar.service:10: the unit is `After=chora-tiling.service`, so where the plugin's panel is up first the unit's `waybar.sh` ends 0 and `Restart=on-failure` never supervises; a waybar crash is then restarted by nothing. Fix: say in the unit comment and docs that the plugin's panel is the owner when it starts first (or give the plugin's panel a restart).
- rtr-3, MINOR, flavors/tiling/waybar/waybar.sh:40: without `flock` (util-linux) the lock is skipped silently and the double bar of ti-47 is back. Fix: one stderr line in that branch (or a package dependency on util-linux).
- rtr-4, MINOR, plugins/chora-tiling/src/theme.rs:1068-1105: the test compares the `.desktop` names with literals, not with `chora_kit::give::move_words`, so the report's "a unit test ties the .desktop names to move_words" holds only through equal literals; `keysheet::move_row` words and the title in `keys::title` are literals too. Fix: take the expected English and German name from `move_words(to, Language::En/De)` in that test.
- rtr-5, MINOR, flavors/tiling/applications/chora-give-{mc,construct,xp}.desktop:5-6,10: `Comment[de]` says "mc-Desktop" and "construct-Desktop" (Name[de] says Mediencenter, Construct), and `Keywords` carry "look", the word ti-48 removed from the visible text. Fix: use the `desktop_name` words in the comments; keep "look" only if typing it into the Apps list is meant to find the row.
- rtr-6, MINOR, tests/gate-tiling-apps.sh:57-62: no cell runs `give.sh`: the gate checks that the rows exist, the success path is shown by hand in r1, the refusal notice ("Window not moved") is untested. Fix: one step running `chora-tiling --launch give xp` with no window, expecting rc 1 and the line `give.sh: choractl: no focused window`.
- rtr-7, MINOR, tests/gate-tiling-apps-click.sh:47-49 and tests/gate-launcher-click.sh:127: the click y is a constant of a list that is full (nine rows, centred), so a flavor that adds rows does not move it, but one that drops below nine does (lane note 4). Fix: derive the first row from a capture or assert the row count of the list in the cell.
- rtr-8, MINOR, docs/configuration.md:1612: the tree comment of `applications/` still reads "the entries of the three looks"; it holds the looks, the three move rows and settings. Fix: "the entries of the launcher: looks, move window to another desktop, settings".

Cleanup: `pgrep -fa 'chora --headless'` empty at the end; no files written outside this report (probe scripts removed).
