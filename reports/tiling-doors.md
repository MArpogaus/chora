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
