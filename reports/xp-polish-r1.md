# xp polish r1 (head 1df6d8ad, branch claude/xp-polish, 2026-11-16)

Commits (author as the brief says; the stop hook wants noreply@anthropic.com, not done):
65f4510 test paperclip cell (red) | 6a87702 fix figure without agent | 51b1cb1 test field cells (red)
33c9502 fix fields on the 4 px grid | 666eb7f ci gate help | 1df6d8a fix: idle figure only in the xp form

xp-23 cause: sync_assistant removed the figure whenever no voice agent was connected, so a fresh xp
session (and tour v3 until its fake agent spoke) had no paperclip; tests/xp.sh only ran with an agent.
Red: tests/gate-xp-paperclip.sh "no helper: a fresh xp session shows no paperclip". Green: at 1192,658 80x104.
First fix showed the figure in every look; the hud gate caught it (3 cells), so it is limited to form xp.
xp-24: chat field was 36 px, Send 28 (off level). Red: "1x: the edit box is 36 px high, not 28".
Now pad 4 in the chora-chat look: 28 px, 1 px #7F9DB9, level with Send at 1x and 2x (pixel assert,
tests/xp-field-rows.py). Run box (fuzzel): pads 12/8, line 28, label and input black; red was vertical-pad=10.
Captures: media/2026-11-16/xp-polish-r1/xp-chat-field-{1x,2x}-1df6d8ad-2026-11-16.png

Gate: tests/merge-gate.sh all steps passed on 1df6d8a (run /tmp/merge-gate.5mardZ). Earlier run failed in
2d-guard (shallow clone, fixed by unshallow) and in hud (above).
Open: fuzzel draws no box around its input, so the Run box has no XP edit box; no other text fields exist in
xp (no search, rename or dialog input found). The open chat balloon covers the paperclip's old place only
as designed; I did not re-check tour v3 stills 01-13/01-19 against a re-record. No 2x capture of the
paperclip or Run box. Gate cells added: xp-paperclip, xp-fields.
