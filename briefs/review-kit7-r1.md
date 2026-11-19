# review kit7 r1 (F2): seeded review of claude/kit7 before its merge

Model: opus, effort high, because the diff replaces every look's helper presenter with one kit state machine
(design across four looks, -2568 lines), where a wrong state leaves a look without an honest helper state
Lane: cloud, read-only on code: claude/kit7 head c1dbc47b, base daf83391 (56 files, +930 -2568). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran its steps green).
Read briefs/kit7-r1.md (done criteria) and reports/kit7.md on claude/session-log; `git diff daf83391..c1dbc47b`.
Checks, each with its command and files:
1) chora_kit::helper::Helper (06b9666c): states no_helper|idle|listening|working|answered and door words are
   complete and honest (vision card: honest states; no "working" without a helper); transitions tested;
   `cargo test -p chora-kit --features testing helper`.
2) Red first: c0414145 before 06b9666c (`git log --oneline daf83391..c1dbc47b`); the 3 kit cells fail there.
3) chora-assistant is the only presenter (state.get helper/door/shown); looks give costume only; old presenters
   gone (grep: kit Helpers, xp Helper, chat door_word, VoiceHud::Own, construct-hud ask, mc-answers);
   services/voice/chat/src/opener.rs: two helpers, the door names the last reporter: correct or a finding.
4) No lost behaviour: xp Start/assistant, mc voice HUD, construct HUD each still show the helper (acts I8 in
   four looks; tests/acts/looks/common.py); gate-construct-hud.sh lost 190 lines: no check dropped without
   an equal one elsewhere (name each).
5) Test changes are not weakened: known-red.toml rows kit7-a1 (mc I3 after chat.show, WM2) and the construct
   Tab "prior flight" in c1dbc47b: a gate workaround hiding a bug (also on daf83391)? Say which, and the item.
6) tools/copies.toml and size-ratchet counts true (`tools/cut-check`); docs/api.md matches state.get words.
Known, not findings: kit7-a1, kit7-a2 as reported (judge only whether the gate hides them).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/kit7.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
