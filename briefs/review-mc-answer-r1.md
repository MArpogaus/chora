# review mc-answer r1 (RC, mc): seeded review of claude/mc-answer before its merge

Model: sonnet, effort high (mc page layout plus a new chora-notify key that touches every look; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/mc-answer head a271df2b, base 92ecd30a (mc-majors, in train 79; 15 files, +341 -44). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No full merge gate (the lane ran 17 steps green).
Step 0: run setup once: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`; `git fetch --unshallow`; `git fetch origin claude/session-log claude/mc-answer`; `cargo check --workspace`; a failure: send_message the PM "review mc-answer BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Read findings/mc.md rows mc-39, mc-40 and the mc-answer report on claude/session-log (reports/mc-answer.md); then `git diff 92ecd30a..a271df2b`.
Checks, each with its command and files:
1) Red first: 43e2c6ca fails act answer-long and e9b20dc6 fails act card-size; dfa0749e and a271df2b pass (`tests/gate-mc-sofa.sh` in a worktree per sha, BIN=$PWD/target/debug after `cargo build --workspace`).
2) mc-40 default unchanged: card.size defaults to today's 11 px, the line height derives from the size (no second key), only plugins/chora-mc/notify/values.yaml sets it; run the notify cells of xp, tiling and construct-hud (`tests/merge-gate.sh --only ci-check,xp,tiling-look,construct-hud` or the cells it lists) green.
3) plugins/chora-notify/src/card.rs: size bounds (0, huge, non-number) refused or clamped with a message; wrapping and card height follow the size; no panic on parse.
4) mc-39: crates/chora-kit/src/hud.rs and chora-assistant config: a plain sentence takes anchor and width; a 150 char answer keeps off date, title and column at 1280x800 and at a smaller output (run the cell or say why not).
5) tests/gate-mc-column.sh waits by state.get, not by sleep; the 3 changed expectations (Super+N, wrapped key row, KEYS rule 462) match the docs and are not loosened to pass; new mc-column step in tests/merge-gate.sh runs in the main list.
6) tests/png-ink.py: measures ink, not a fixed pixel; documented use; no copy of an existing helper (`tools/copies.toml`, ci-check).
7) Docs: docs/configuration.md, docs/hud.md and the notify READMEs name the key and its default.
8) `cargo test -p chora-notify -p chora-kit -p chora-assistant` and `cargo clippy --workspace --all-targets -- -D warnings`.
Known, not findings: mc-41 (wrapped key row hit rect), the other open mc MINORs in findings/mc.md.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv-N, severity, file:line, text, fix in one sentence). Append to reports/mc-answer.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
