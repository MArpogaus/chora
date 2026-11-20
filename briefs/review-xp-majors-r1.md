# review xp-majors r1 (RC, xp): seeded review of claude/xp-majors before its merge

Model: sonnet, effort high (five "gone before" claims and one xp fix; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/xp-majors head 81a29fe8, base 30727c82 (train 77; 8 files, +341 -9). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No full merge gate (the lane ran 13 steps green).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`; `git fetch origin claude/session-log claude/xp-majors`; `cargo build --workspace`; a failure: send_message the PM "review xp-majors BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Read on claude/session-log: briefs/xp-majors-r1.md (the done criteria), findings/xp.md rows xp-23, xp-26, xp-27, xp-31, xp-32, xp-33, reports/xp-majors.md "## r1", reports/p2-r1-xp.md "## r1" and feedback/2026-11-20-p2r1-{murat,pixel}.md (what the personas saw); then `git diff 30727c82..81a29fe8`.
Checks, each with its command and files:
1) Red first: 24640558 fails the speaker act and 81a29fe8 passes it (`tests/acts/run.py` per its usage, xp look, in a worktree per sha after `cargo build --workspace`).
2) "Gone before" is real, not a loosened test: for xp-23, xp-27, xp-31, xp-33, xp-26 name the guard (act or cell) that asserts what the persona missed (paperclip at its place; question card in the chat font and frame, off the transcript, Yes/No buttons; Super+D toggles back and a task button restores; Super+F1 key list; level in tooltip and menu tick). The guard must fail if the behaviour is removed: show it by a one-line revert in a scratch worktree, or say why not.
3) Persona misses blamed on the harness (wpctl stand-in fixed at 0.50, Help left open, one Super+D press): check tests/acts/harness.py STAND_INS and tests/acts/looks/xp.py; the stand-in must not hide a product fault (a real wpctl level change shows in the tray).
4) xp-32: plugins/chora-xp/src/context.rs, wm/panels.rs, wm/press.rs: the popup is left of the helper and on screen at 1024x768 and 1280x800, Mute toggles and shows, level 100/75/50/25 ticks the current level, a click outside closes it, keys do not leak to the client. The brief asked for "the XP level slider with Mute": the builder made a stepped list; judge against vision-card (pixel-faithful XP, POLS) and give MAJOR only if Erika or Murat would not find or use it; else MINOR to the known issues.
5) tests/doors/xp.sh volume-door clicks Mute in the popup and still proves the volume door.
6) plugins/chora-xp/src/wm/panels/tests_moved.rs: unit test for the popup; no file over 1,500 lines (`tools/` line check in ci-check).
7) `cargo test -p chora-xp` and `cargo clippy --workspace --all-targets -- -D warnings`; acts xp look all green.
8) No Microsoft files committed (`git diff --stat` binaries; fallback art only).
Known, not findings: the other open xp MINORs in findings/xp.md.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv-N, severity, file:line, text, fix in one sentence). Append to reports/xp-majors.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
