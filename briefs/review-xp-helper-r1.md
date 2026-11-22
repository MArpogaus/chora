# review xp-helper r1 (rc 1: shared bar "the helper (mouse and keys)"): seeded review of claude/xp-helper before its merge

Model: sonnet, effort high (plugin diff, 127 product lines plus a 141-line gate script; no core change)
Lane: cloud, read-only on code: claude/xp-helper head eda60a6b, base 3bec0cbf (8 files, +257 -11). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it: 54 PASS, watchdog-starved foreign).
Read briefs/xp-helper-r1.md (done criteria) and reports/xp-helper.md "## r1" on claude/session-log; `git diff 3bec0cbf..eda60a6b`.
Checks, each with its command and files:
1) Red first: ae9bbe5e before 4fec75c3 (xp-30), 66c5a23b before 09ff8ff9 (xp-28); `git log --oneline 3bec0cbf..eda60a6b`.
2) xp-30 greeting (plugins/chora-assistant/src/voice.rs, assistant.rs, main.rs; crates/chora-kit/src/words.rs): greets once,
   only in the xp form, honest "no helper" text; the "Click me to see how" promise holds (a click on the paperclip with no
   helper shows how to set one up; if it does not, that is a finding); the remembered state file path is created with -m 700
   rules (mkdir mode), survives no panic on a read-only or missing XDG_STATE_HOME; the 1.5 s timer cannot greet after the
   look changed away from xp; de words exist and fit. `cargo test -p chora-assistant -p chora-kit`.
3) xp-28 placement (plugins/chora-xp/src/window.rs, desktop_icons.rs): the cascade starts right of the icon column only
   where icons show (output with no icons, a second output, a small output 800x600 where 116+700 does not fit: the window
   must stay on screen); the cascade wraps as before. `cargo test -p chora-xp`.
4) gate-xp-helper.sh: no process picked by name alone, no bare sleeps as the only wait for a state the cell asserts, cleans
   its state dir; the xp-29 cell really clicks (person path), so it guards the finding.
5) POLS and KISS: no new eprintln (lint ratchet), no second greeting path, words in words.rs only.
Known, not findings: xp-29 absent at the head (cell kept as guard); watchdog-starved red (FLK, core, not this lane).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rx-N, severity,
file:line, text, fix in one sentence). Append to reports/xp-helper.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
