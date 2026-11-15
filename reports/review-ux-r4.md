Verdict: MERGE-OK

# review ux r4 (claude/ux-r3 c434bc34)

Scope: 737ec5af (red), 7078b8a7 (fix), 30bd7135 + c434bc34 (gate-hud cells), on top of the r3
commits. Probes ran on scratch exports (`git archive`) under /tmp/claude-0/rev-ux4/{src,red};
CARGO_TARGET_DIR=/home/user/target-review, removed at the end. No tracked file changed.

## BLOCK

None.

## MAJOR

None. M1 of review-ux-r3 is closed (see "Checked and fine").

## MINOR

1. Stop, Close and the transcript still move 20 px when the header changes between one and two
   lines (the lane's stated deviation; it is review-ux-r3 MINOR 3, still open).
   `plugins/chora-assistant/src/chat/view.rs:534-537` (`tops` uses the actual line count).
   Probe `probe_plain_r4` (plain and bubble, en/de, default width and 240, 1280x800 and 1024x768):
   `researcher · thinking` gives top_row 40 / body_top 74; the same helper with a long task
   gives 60 / 94. At chat.width 240 a short task with steps (`Tidy\nnotes (3/5)`) already
   takes two lines, so the step count arriving during a run can move Stop by itself.
   POLS judgement: bounded (one 20 px step, at most once per start/stop of a run or once when
   the step count appears), no click target leaves the panel, glass and xp do not move (fixed
   bands). Still it moves Stop at the moment a user aims at it (the run just started), and it
   reflows the transcript by one row. Not MAJOR; recommend the one-line change the lane names
   (reserve `HEADER_LINES` in `tops` for plain/bubble), costing one transcript row. PM call.
   Earliest gate: unit. `a_long_prompt_keeps_the_header_of_every_plain_look_at_two_lines`
   compares the long task with the two-line no-helper header only; add the assert that a
   one-line chat header (short task) has the same `top_row`. gate-hud `chat-long-task` only
   compares 1800 vs 3600 (both two lines) and a ceiling of 88 px, so it cannot see this.

2. Layout work grows with the prompt, not with the two lines shown.
   `view.rs:561-567` `capped` flattens and wraps the whole task, then drops all but two lines;
   glass does the same at `glass.rs:459` for one line. Each word costs a `text_width` of the
   current line; a word without spaces costs one per character. Probe `probe_review_r4`
   (release): 1800 chars 0.09 ms (cached), 100 000 chars with spaces 3.2 ms, 20 000 chars
   without spaces (a pasted blob or long path) 272 ms per redraw (debug 425 ms); the theme's
   width cache (4096 entries, cleared when full, chora-draw theme.rs:410) thrashes on such text.
   `chat_draw` runs on every chat event (keys, messages), so a pasted blob makes the chat
   plugin lag for the whole run. Not a per-frame path of the compositor (plugin process).
   Introduced by this lane (base drew one fitted line). Fix: stop wrapping after `most + 1`
   lines (or cut the flat text to a bound before wrap).
   Earliest gate: unit with a long unbroken task and a work bound (e.g. count calls of the
   width closure), none today.

## COSMETIC

1. The ellipsis cut is per character and keeps a trailing space: plain capture
   `chat-long-task-1800.png` ends `Please tidy the n…`; glass `construct-chat-long-task-1800.png`
   ends `tidy the …` (space before `…`); probe at 480 glass `notes …`, plain `Pleas…`.
   `view.rs:575-587` `as_shown` pops chars and never trims. The brief only asks the task to end
   in `…`, and a mid-word cut at the end is usual; but trim trailing whitespace before `…`,
   and optionally back up to the last word boundary when one lies within ~1/3 of the room.
   Earliest gate: unit on `as_shown` (no line ends in ` …`).
2. A long task hides the step count `(3/5)`: it is appended after the task
   (`plugins/chora-assistant/src/chat.rs:1127`, glass `glass_head`), so the cap cuts it away in
   every look (probe: plain default `... Read thi…`, no `(3/5)`). Same as before r3 (one fitted
   line), so not a regression; putting the steps before the task would keep them.
   Earliest gate: unit (long task with steps keeps `(n/m)`).
3. Glass at chat.width 240 leaves little task: `researcher · R…` (de, `Denkt nach`),
   `coder · Read this f…`. Name and state whole as asked; a helper name longer than the room
   is cut (`capped(... 100.0, 1)` gives `researcher-…`), the meta then says `…`, so the doors
   gate would see it. Helper names are short ids today.

## Checked and fine

- M1 closed in every look. Units on HEAD: 88/88 chat units pass (incl. probes). Probe over
  plain + bubble x {default, 240} x {1280x800, 1024x768} x {en, de} x {no task, short + steps,
  ~2400-char task}: never more than 2 header lines; `name · state` whole on line 1; long task
  ends in `…`; top_row <= 60, body_top <= 94. Glass {240, 300, 480} x {en, de} x 3 helpers:
  exactly state word + one line, starting with the whole name, ending in `…`, inside the room
  left of the word (unit `a_long_prompt_keeps_the_name_and_ends_the_task_beside_the_glass_state_word`).
  Glass without a state word at 240: <= 2 lines in the 56 px band (r3 MINOR 1 closed).
- Captures: chat-long-task-1800/-3600 (plain) show two header lines, `atlas · working · ...`,
  Stop/Close at y 60-88 under them; construct-chat-long-task-1800/-3600 show `WORKING` and one
  line `atlas · Read this first. Please tidy the …` clear of the word and of the separator.
  The 1800 and 3600 captures are byte-identical per look (same md5), as expected for a cap.
- Red first: on 737ec5af the four new units fail (two glass, two view; panics at
  glass.rs:710, 744, view.rs:1735, 1770); on HEAD they pass.
- Gate cells: `header_capped` needs the meta to start with `atlas · working · Read this first.`
  (glass: `Working\natlas · Read this first.`), line 2 to end in `…`, and no `Please tidy` on
  line 3+. On the old build plain wraps to ~90 lines (line 2 has no `…`) and glass prints the
  task's line breaks as more meta lines (`atlas · Read this first.` without `…`), so both cells
  fail on the old build (also the lane's pre-fix run). `buttons_under_header` bounds Stop/Close
  to the top 88 px of the panel; 1800 vs 3600 must give the same rects. The glass cell's
  button comparison is weak (key-line hits, independent of the header) but harmless. Cells
  restore the fake voice and wait for the figure (c434bc34).
- Meta height (r3 MINOR 2): `shown_lines` marks a header line whose band ends below the panel
  as `…` (view.rs, unit `the_meta_calls_a_header_line_cut_at_the_bottom_not_whole`); glass
  checks the band against the 56 px header.
- Safety: `rest[..2]` in `capped` is safe (lines.len() > most implies rest.len() >= 2);
  negative or zero room gives `["…"]` / `["c","…"]`, no panic, no loop; `try_from` with
  `unwrap_or`, no bare unwrap in non-test code. A char-broken long word joined with a space in
  `capped` does not show the space (the cut lands before it; probe URL case).
- KISS: one rule `capped` + `HEADER_LINES` for plain, bubble and glass; no option, flag,
  grab or timer. docs/hud.md describes the cap and the meta. xp unchanged.

## Gates missing

- Unit: a one-line chat header keeps the same `top_row` as a two-line one (MINOR 1, if the
  PM wants the fixed band).
- Unit: a long unbroken task is laid out with bounded width calls (MINOR 2).
- gate-hud: the long-task cells run at 1280x800 1x and default width only; 240 and 1024x768
  rest on units (stated by the lane).
