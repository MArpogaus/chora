# xp-readable r1 (F7, rc 11; xp MINOR, rule 6: xp has no open BLOCK or MAJOR): contrast and small text of chora-xp

Model: sonnet, effort high (pixel and colour work against Luna references with asserts; no new feature)
Lane: cloud, branch claude/xp-readable from 5b34d4b1 (session head, t114 green). Stop at 250 k context; cost at most 1.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true; cargo check --workspace; python3 -c "import PIL"`; a red check: send_message the PM "xp-readable BLOCKED: <line>" and stop (no apt).
A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/xp.md rows xp-9, xp-10, xp-13, xp-19, xp-39, xp-40, xp-41; reports/record-v9.md "## r4"; demo/tour/known-red.toml (the tray exemption); design/looks/xp*.md; vision-card.md (chora-xp: pixel-perfect Luna, fallback art acceptable).
Scope (one look, these rows only; xp-28, xp-29 are not this round):
1) xp-39 + xp-19: workspace buttons 1-4 are our own element, not Luna. Dim them to the tray tone and reach >= 4.5:1 for every digit (I6 cell of `python3 demo/tour/run.py --check` or a pixel assert in tests/gate-xp-*.sh). Red first.
2) xp-40, xp-41: for each pair decide in the report: authentic Luna colour (measure our pair against the Luna reference values in design/looks; a match within 2 per channel) -> exempt in demo/tour/known-red.toml like the tray, floor 3.0, one line per pair with the reference; our own colour or under 3.0 (inactive title 2.16, Recycle Bin label 1.67) -> fix (e.g. label shadow as XP draws it) with a red assert first, or argue in the report why the replica must keep it (then it stays open).
3) xp-9, xp-10, xp-13: Turn Off Cancel >= 24 px high target, Date and Time text and buttons at XP metrics, Recycle Bin not under the chat balloon; each with a pixel assert red first, then the fix. A row that cannot be measured headless: say why, it stays open.
4) Fallback art only; no Microsoft files. No change to other looks.
Gate: the full `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` as a background task (no --only; product code), poll it in the same turn; green; merge origin/claude/autonomous-dev-session-3w-i33pg7 first if it moved; push the branch.
Report: reports/xp-readable.md "## r1" on claude/session-log: per row red sha, fix sha or exemption line, cell, contrast before and after, captures (paths), what stays open.
Send the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 6 lines with send_message. Poll every command in the same turn; never end a turn to wait.
Rules: commits as Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify;
red test commit before each fix; push only claude/xp-readable and claude/session-log (fetch and merge on reject, no force); no system packages.
