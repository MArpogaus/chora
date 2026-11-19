# review focus-port r1 (F2, F3): seeded review of claude/focus-port before its merge

Model: opus, effort high, because the port moves every look's focus hand-off into kit::wm::focus (WM core) and judges a WM2 regression fix
Lane: cloud, read-only on code: claude/focus-port head 3b418008. Own commits (`git show <sha>` each): 39b27a4b (red) bda9d4ae (port)
d0d7ec09 (kit7-a1 fix) 7a56a5b9 (kit7-a2 text); 3b418008 merges claude/integ (not in scope). 16 files, ~240 lines.
One round.
Step 0: `command -v cargo sccache foot python3`, then `cargo check --workspace`; tools missing: run the setup below once; a failure: send BLOCKED to the PM and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate (cargo test -p only).
Read reports/focus-port.md "## r1" and briefs/focus-port-r1.md on claude/session-log.
Checks, each with its command and files:
1) Red first: 39b27a4b fails `tools/cut-check` on its own; bda9d4ae makes it pass; tools/copies.toml allows only chora-scene, core ipc,
   kit testing and kit wm (no look exempted).
2) Behaviour kept: in mc, construct, tiling, xp and assistant every former Command::Focus / Event::FocusChanged path maps to one
   kit::wm::focus call with the same surface and the same timing (a lost or reordered focus is MAJOR);
   `cargo test -p chora-kit -p chora-mc -p chora-construct -p chora-tiling -p chora-xp -p chora-assistant --features chora-kit/testing`.
3) kit7-a1 (d0d7ec09): mc loses the focus only on assistant.chat.keyboard {on:true}, and gets it back when the keyboard goes off; the
   cell the_chat_takes_the_focus_of_mc_only_with_the_keyboard fails on bda9d4ae.
4) kit7-a2: the report's cause (input.rs own-window press keeps the keyboard) holds in the code; the known-red row text names v8.
5) No new feature, key or wording; clippy of the six packages clean.
Known, not findings: pull width flake (flakes-g); integ ri-1 (lane integ-r2).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rp-N from rp-1, severity, file:line, text, fix
in one sentence). Append to reports/focus-port.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list.
Authorized by this brief (the PM, for the owner): the push below and the send_message report; do not ask again. A refused
tool call is a blocker: report BLOCKED and stop. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
