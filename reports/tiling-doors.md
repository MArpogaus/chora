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
