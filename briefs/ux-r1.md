# ux r1 (rc 1): the last open MAJOR findings

Lane: Fable, cloud, new branch claude/ux-r1 from the session branch
claude/autonomous-dev-session-3w-i33pg7 (d349a6da or newer). One round. Stop at 250 k context and
hand over. Read dev-rules.md and conventions.md (with the skills it names) first. Setup as
briefs/doors-r3.md (setup.sh now installs mako and notify-send). Read reports/ledger-r1.md and the
findings lines named below. Another lane (flakes r4) swaps the regrab calls in xp, tiling, mc and
construct; keep away from those grab calls.
Items, red first (a gate cell or a capture check), in this order:
1. mc-30: at 1280x800 with voice, host and a helper, the mc header overlaps (helper name under Mute,
   date cut by the island, clock under the dot). One layout rule, checked at 1280x800 and 1024x768.
2. sh-10: the xp no-helper balloon title is cut ("No helper is set up. Ask the pers..."): the
   balloon fits its text (xp balloon rules), or the text is shorter and still honest.
3. sh-11: with exactly one helper, the chat picks it; typed text before a pick is sent once a
   helper is picked (not only kept).
4. sh-7: the give keys Super+Alt+1..4 appear in the tiling Keys sheet and the mc All keys list.
5. sh-5: the cube stood back shows a clock and a way to lock/power (a minimal strip or the walk
   strip's clock), consistent with the construct look; decide with POLS and say why.
6. ti-4: verify the screenshot notice in tiling with mako (gate-tiling-notice); fix if it fails.
Done when: tests/merge-gate.sh passes as one run; captures at 1x per item (list the paths). Report to
the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 20 lines) and as reports/ux-r1.md on claude/session-log.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution lines;
never --no-verify; stage by name; no force push; push only claude/ux-r1 and that report; never commit
Microsoft files; never end a turn to wait for a long background job.
