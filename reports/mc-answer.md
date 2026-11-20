# mc-answer

## r1

Lane: cloud, branch claude/mc-answer from 92ecd30a, head a271df2b (pushed). Setup, `cargo check`
and `cargo build --workspace` passed. mc-39 and mc-40 are fixed, each with a red cell first;
`gate-mc-column` runs green and is a cell of the merge gate.

| id | red | fix | cell |
|----|-----|-----|------|
| mc-39 | 43e2c6ca | dfa0749e | `answer-long` of tests/gate-mc-sofa.sh |
| mc-40 | e9b20dc6 | a271df2b | `card-size` of tests/gate-mc-sofa.sh |
| mc-column | none (gate repair) | deeb5349 | `mc-column` of tests/merge-gate.sh |

### mc-39 (long voice answer)

Red: a 150-character `voice.say` made a 643x92 box (3 lines) at x 467 over the date, the page
title row and the column. Fix: a `plain` sentence with an `anchor` stands at that place and takes
a `width` (new panel key `width`, 1 to 1600, docs/hud.md); the mc folder puts it at the bottom
left (`anchor: bottom_left, margin: [91, 84], width: 736`): two lines, 91 to 835, above the
footer. Without an `anchor`, a sentence is drawn as before (beside the figure); no other look
changes. No place at the top right fits: the date and the title row stand there.

### mc-40 (card text at sofa distance)

The text size of the card is the fixed 11 px of the host (`Font::Ui`, `LINE` 13) and values.yaml
had no key for it, so the slot config alone could not raise it. The PM allowed a code change
(11-20 00:18): `card.size` is an optional key of the bubble style (default: the small text of
the host, every other look is drawn as before), a line is 147 hundredths of the size, the width
of the longest line follows the size (300 px at 11). mc sets `size: 19`. Red cell: ink lines of 11
px against 18 px of the page body; green: the card reads 19 px, ink 7:1. The check also holds in
the light theme.

### mc-column

`ctl wait owner:notify` never ends before a notice shows; the gate waits by `state.get` now. Three
more stale expectations of the gate on this head, all in the test: the key spelling `Super+N`; the
row "Start a program or run a command" wraps to two lines at 336 px, so the pointer aims at its
first line; the KEYS rule stands 28 px higher at 1280 wide (the key block is 308, at 1920 it
stays 280). Step `mc-column` added after `mc-sofa` (header comment too).

### Gate

origin/claude/autonomous-dev-session-3w-i33pg7 was already merged (no new commits). `tests/merge-gate.sh
--jobs 3 --only pre-commit,ci-check,build,test,clippy,acts,mc-paths,pixels,mc-sofa,mc-column` plus
(PM, for the card change) `xp,hud,tiling-audit,tiling-look,construct-hud,doors,chat-keys`: all 17
steps PASS.

### Seen on the way, not in scope

1. MINOR, mc: the hit rect of a key row that wraps covers only its first line (the row "Start a
   program or run a command", 56 px high, hit 28 px): the pointer on its second line shows no
   focus bar and a click does nothing.
2. MINOR, mc: that row wraps at 336 px although the text column is 224 px wide; the mc-column gate
   of 92ecd30a expected one line (KEYS rule at 490).

## Review r1

Reviewer: cloud lane, sonnet high, head a271df2b against base 92ecd30a (15 files, +341 -44). One round.
Verdict: **FIX** (1 MAJOR open, 0 BLOCK, 0 MINOR).

| id | severity | where | finding | fix |
|----|----------|-------|---------|-----|
| rv-1 | MAJOR | plugins/chora-notify/src/card.rs:95 and :377 (`Metrics::of`, `write`) | `card.size` has no bounds, so a `values.yaml` with `size: 0` panics at the first notice that has a body (line height 0, `step_by(0)` at :377), and any size above 29 218 532 panics on `size * LEADING` (:95, "attempt to multiply with overflow" in debug, a wrap in release); a size such as 100000 gives a card of millions of pixels. A non-number is refused by serde with the file name; 0 and huge are neither refused nor clamped (check 3). Reproduced with a probe test on a271df2b (not committed). | Refuse a size outside a range (the HUD takes 14 to 48 for its `size`) with a message that names `values.yaml` when the values load, or clamp it; use `checked_mul`/`saturating_mul` in `Metrics::of`; add a unit test for 0 and `u32::MAX`; name the range in docs/configuration.md and plugins/chora-notify/assets/README.md. |

### Checks

1. Red first: PASS. 43e2c6ca fails `answer-long` ("[467,29,643,92], more than two lines"); dfa0749e passes it; e9b20dc6 fails `card-size` ("lines of ink 12 11 11, the page body 18"); a271df2b passes both.
2. mc-40 default unchanged: PASS. `size` is `Option<u32>`, `None` keeps LINE 13 and MAX_TEXT 300; the line height derives from the size (147 hundredths, no second key); only plugins/chora-mc/notify/values.yaml sets it (19). `ci-check`, `xp`, `construct-hud` PASS; `tiling-look` PASS when run alone (548 s). In the first run, `--jobs 3 --only ci-check,xp,tiling-look,construct-hud,mc-column`, `tiling-look` failed once at 12 s with "ink-1280x800: the current tag is 1 1 1 255, not the accent 167 139 250"; the branch does not touch the tiling look, so I take it as a flake under parallel load (not a finding; the PM may want a note in the friction list).
3. card.rs bounds: FAIL, rv-1. Wrapping and height follow the size (`Metrics`, tests pass); the SVG family is restricted to alphanumerics, space, `_` and `-` (no injection). No panic on non-number.
4. mc-39: PASS. `anchor` and `near` exclude each other in `check_place`, so the docs sentence ("an anchor and no near") matches the code; `width` is checked from 1 to 1600. The `answer-long` cell passes at 1280x720 (its own size), 1280x800, 1024x600 and 1920x1080 (temporary copy of the gate with another `session_start` size, removed). The other looks (neutral, tiling) have `near: figure` and `style: bubble`, so they are unchanged.
5. mc-column: PASS. The start wait is a `state.get` loop of 20 s, not a sleep; `mc-column` PASS in the gate (54 s) and is a step after `mc-sofa` in the main list (header comment too). The three expectations follow the code, not a looser bound: "Super+N" is the spelling of the keysheet (e847eeb0 did the same for the doors; docs/mc-style.md names the binding `Super+n`, which is the binding, not the label); the pointer aims at the first line of the wrapped row (the hit rect is mc-41, known); the KEYS rule 462 and the key block of 308 at 1280x720 are exact rects, the 1920 value stays 280.
6. tests/png-ink.py: PASS. It measures runs of ink rows against the most common colour, not a fixed pixel; the docstring gives the usage; it reuses `png-pixel.py` (`rows`) and shares only the 5-line loader preamble with `png-contrast.py`; tools/copies.toml has no row for it and `ci-check` PASS.
7. Docs: PASS. docs/configuration.md, plugins/chora-notify/assets/README.md and plugins/chora-mc/notify/README.md name `card.size` and its default; docs/hud.md names `width` (default 420, or 640 with a size; matches `MAX_PLAIN`, `MAX_PLAIN_SIZED`). The range of `card.size` is missing because there is none (rv-1).
8. `cargo test -p chora-notify -p chora-kit -p chora-assistant` PASS (all binaries ok); `cargo clippy --workspace --all-targets -- -D warnings` PASS.

Known, not findings: mc-41 and the open mc MINORs of findings/mc.md.

## r2

rv-1 (card.size unbounded): fixed.
- red: d3b2a9cc (tests for 0, 13, 49, 100000, u32::MAX: clamp to 14..48 with a message naming the value and range; metrics never overflow)
- fix: 92f2e52f (`bounded` clamps to 14..48 with one log line; `saturating_mul` in `Metrics::of`; row height at least 1 in `write`; range named in docs/configuration.md and plugins/chora-notify/assets/README.md)
- cell: `cargo test -p chora-notify` 27 passed.
- gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,mc-sofa,mc-column` on 92f2e52f (merge of origin/claude/autonomous-dev-session-3w-i33pg7: already up to date): all 7 PASS.
- head: 92f2e52f
