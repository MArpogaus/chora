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
