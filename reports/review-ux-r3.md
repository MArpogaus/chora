# review ux r3 (claude/ux-r3 42d5d4b2)

Scope: sh-10 only (ff9e68d7 test, 42d5d4b2 fix, on be205f89). ti-1 not reviewed.
Probes: scratch copies under /tmp/claude-0/rev-ux3 (red/, probe/), logs red-units.log,
fix-units.log, probe.log. No tracked file was changed.

## BLOCK

None.

## MAJOR

1. The plain header now wraps every header, not only the no-helper words, and it has no limit.
   `plugins/chora-assistant/src/chat/view.rs:627` wraps `self.header(language)` in all states.
   In a chat that header is `name · state · task`, and `task` is the whole prompt the user sent
   (`services/voice/src/agents.rs:1054` `s.task = text`; coder: "the task in full"). Newlines in
   the prompt are kept by `sanitize` and become extra lines.
   Probe `probe_review_long_task_and_narrow` (probe.log), plain look, 1024x768, 400 wide:
   - idle or short task: 1 header line, buttons at y 40, transcript at 74.
   - a 280-char prompt: 6 header lines, buttons at y 140, transcript at 174 (5 rows lost).
   - same prompt at chat.width 240: 10 lines, buttons at y 220.
   - by the formula `PAD + LINE * n + GAP` (view.rs:535), about 36 lines (a pasted prompt of
     roughly 1800 chars) put Stop and Close below the bottom of a 738 high panel: the click
     targets are gone and the transcript has no room.
   Before the fix this header was one line with an ellipsis. Now Stop and the transcript jump
   down each time a run starts with a longer prompt, and back up when it changes. This hits
   tiling, mc and the bubble balloon (all plain), in the picked-helper state the brief asked to
   keep. Fix: wrap only headers without a helper state (no-helper, offline, waiting, launcher),
   as glass does; or cap the header at 2 lines and cut the rest with an ellipsis.
   Earliest gate: none. Missing: a unit that a chat header with a long task keeps the buttons
   and body_top where a short one has them (or at most 2 lines).

## MINOR

1. Glass header without a state word overflows its 56 px band at 3 lines.
   `plugins/chora-assistant/src/chat/glass.rs:453-458`: no cap on the lines; `top` goes to
   (56 - 60) / 2 = -2, the first line starts above the panel and the third crosses the
   separator at y 56. `glass_width` keeps the configured width when it is below 400
   (chat.width allows 1..max). Probe `probe_review_glass_narrow`: width 240, en and de
   no-helper and de offline give 3 lines (60 px in 56). The meta still says whole, so the doors
   gate would pass. Widths 300 to 480 are fine (2 lines). Not a default look.
   Earliest gate: none. Missing: the glass unit only checks 400 wide; check the narrowest width
   the config allows, or cap at 2 lines.
2. The meta reads lines across the width only. A vertical cut (a line drawn outside its band or
   the panel, as in MAJOR 1 and MINOR 1) still reports whole. The doors gate has no way to see it.
3. The header height now differs between states in the plain look (no-helper 2 lines,
   launcher and chat 1 line): the buttons move 20 px when the list opens with no helper and then
   a helper is picked. Small, but it is the jump the brief warned about; it goes away with the
   fix of MAJOR 1 only if the no-helper header keeps its own height.
4. The doors cells run the helper-none check at 1280x800 1x only. 1024x768 and 2x rest on the
   units (1x widths in logical px) and on the lane's probes, which are not in the gate.

## Checked and fine

- Red first: on ff9e68d7 (sources touched so cargo rebuilds) both units fail with the widths
  the report gives: plain 417.5 in 372, glass 406.9 in 368 (red-units.log). On 42d5d4b2 both
  pass (fix-units.log).
- Wrap of the no-helper header at the default sizes: plain 400 wide at 1280x800 and 1024x768,
  glass 400 wide (glass_width floor) at both: 2 lines, en and de, nothing cut. Offline and the
  waiting word fit (1 or 2 lines). Buttons keep their hit rects: `plain_buttons` places them at
  `top_row`, which follows the header. The transcript keeps 644 px or more at 1024x768.
- 2x: widths come from the theme in logical px and the renderer cuts with the same
  `fonts.width`, so the wrap and the meta hold at any integer scale. No extra probe run at 2x.
- Meta is true for width: plain `as_shown` uses the same width function as `Text::fit`, so a
  line the meta calls whole is not cut when drawn; a too-long single word is broken by `wrap`,
  which turns the gate red (false red, not false green). Glass meta uses the same room as the
  drawing (text_x is PAD when there is no state word, since the ring needs a word).
- The old construct hole (clip at the panel edge, meta whole) is closed for the no-helper
  header: the meta is now built from the same wrap.
- says_whole joins lines with spaces and matches any node; no other node carries the full
  sentence in these looks (the hint node only in a slot/margin, and it wraps). tiling's `why`
  returns 1, so `|| why ... || return 1` fails the cell.
- xp: header and meta unchanged (xp.head as before). mc: plain, wraps like tiling. Glass
  header with a state word (picked helper) unchanged.
- KISS: one wrap helper (`chora_draw::text::wrap`), same type size, docs/hud.md updated.

## Gates missing

- A unit for the plain chat header with a long task: buttons and body_top stay put, or the
  header is capped (MAJOR 1).
- A glass unit at the narrowest allowed width: header lines * 20 <= HEADER (MINOR 1).
- A vertical check of the meta lines against their band (MINOR 2).
- The helper-none cells at 1024x768 and 2x in the doors gate (MINOR 4).

## Verdict

FIX. MAJOR 1: the wrap reaches every plain header, and a long prompt pushes Stop, Close and
the transcript down or off the panel.
