# ux r4 (rc 1): close review-ux-r3 (FIX M1)

Lane: opus, cloud, branch claude/ux-r3 continued from 42d5d4b2 (origin/claude/autonomous-dev-session-3w-i33pg7
already merged, nothing new). Pushed, not merged. Full `tests/merge-gate.sh` on c434bc34: 28 steps PASS, exit 0 (e2e: the five listed skips of tests/e2e-skips.txt:
session-units, fkeys option, screencast, x11, xp-token). `tests/ci-local.sh`: msrv, docs, pre-commit, deny,
machete, packaging PASS (no SKIP).

| Item | Red commit | Fix commit | Cell | State |
|---|---|---|---|---|
| M1 header grows without limit | 737ec5af | 7078b8a7 | unit `a_long_prompt_keeps_the_header_of_every_plain_look_at_two_lines`; gate-hud `chat-long-task` (30bd7135, c434bc34) | fixed |
| MINOR 1 glass header at width 240 | 737ec5af | 7078b8a7 | units `the_glass_header_without_a_state_word_keeps_two_lines_at_the_narrowest_width`, `a_long_prompt_keeps_the_name_and_ends_the_task_beside_the_glass_state_word`; gate-hud `construct-chat-long-task` | fixed |
| MINOR 2 meta height | 737ec5af | 7078b8a7 | unit `the_meta_calls_a_header_line_cut_at_the_bottom_not_whole` | fixed |

## Fix
One rule, `capped` (plugins/chora-assistant/src/chat/view.rs), lays out the header of plain, bubble and glass:
it wraps at the words into at most `HEADER_LINES` = 2 lines (one line beside the glass state word), line
breaks of the text count as spaces, and longer text ends in `…` on its last line. The no-helper words keep
their two whole lines at the default widths (the r3 units still pass). A picked helper reads
`name · state · task…`: name and state on line one, the task cut at the end of line two. Glass draws the
`name · task` line beside the state word cut to the room left of it (it overlapped the word before). The
meta of `hud-chat` calls a header line whole only if it fits its width and its band lies inside the panel
(glass: inside the 56 px header band); a line cut at the bottom is `…`. docs/hud.md says both.

## Red and green evidence
- Units on 737ec5af (fix not yet in): 4 failed: plain `Plain None [0, 0, 1280, 770]: 59 lines`; glass
  240 `["No helper is set up. Ask the", "person who set up this", "computer."]` (3 lines); glass word line
  `["Working", "coder · Read this first.", "Then P…"]` (the task split into a third line); meta
  `left: "computer." right: "computer."` (cut line called whole). On 7078b8a7: 220 of 220 pass.
- Units cover plain and bubble, chat.width default and 240, 1280x800 and 1024x768, for a 1800-char prompt
  with line breaks: at most 2 lines, starts with `coder · working · `, ends in `…`, each line fits; Stop and
  Close (`top_row`) and the transcript (`body_top`) stand exactly where the two-line no-helper header puts
  them, and inside the panel.
- gate-hud cells, pre-fix sources built with the new cells: `chat-long-task` missing (the chat did not
  settle in 5 s under the 90-line header), `construct-chat-long-task` missing (meta `Working / atlas · Read
  this first. / Please tidy the notes folder and …`). With the fix: both pass; Close and Stop keep the
  same rect for the 1800- and the 3600-char task (`[1178,60,88,28]`, `[1084,60,88,28]` in the plain chat).

## Open / deviations
- State: no flag, grab or timer added; the cap is pure layout. The cells restore the fake voice and its
  figure (c434bc34; shown by voice-state-rate and layers-dropped-empty passing after them).
- Item 1 "stay where they are": read as "never further down than under a two-line header". A short task
  (one line) still puts the buttons 20 px higher than a long one; review MINOR 3 (the 20 px jump between a
  one-line and a two-line header) is not removed, because a fixed two-line band was not asked and costs a
  transcript row in every chat. One line to change if the PM wants it (`tops` uses `HEADER_LINES`).
- The gate cells run at 1280x800 1x with the default widths; chat.width 240 and 1024x768 rest on the units.
  Review MINOR 4 (helper-none cells at 1024x768 and 2x) not taken up: not in this brief.
- The long-task cells restart the fake voice of gate-hud with FAKE_VOICE_ENTRIES (chat.open must answer,
  else the chat says offline) and restore it after; c434bc34 waits for the figure after the restore.

## Captures at 1x (media/2026-11-15/ux-r4/)
chat-long-task-1800.png and -3600.png (plain, neutral folder), construct-chat-long-task-1800.png and
-3600.png (glass): two header lines ending in `…` with Stop and Close under them; WORKING and one line
`atlas · Read this first. Please tidy the …` beside it.
