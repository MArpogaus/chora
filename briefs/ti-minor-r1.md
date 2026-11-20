# ti-minor-r1 (RC 2 known issues, shared bar "notify never covers work"; tiling MINORs): ti-5, ti-6/ti-19, ti-7

Model: sonnet, effort medium (routine: three small tiling fixes with gate cells).
Lane: cloud, branch claude/ti-minor from fee9c695 (session head). One round, stop at 250 k. Cost about 1.5 %.
PM note on rules.md 6: ti-1 (BLOCK) is open only for the fuzzel >= 1.11 pin in the environment, not chora code; RC 1 is waived (owner 10:47). So MINOR work on tiling is allowed this round.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`; `cargo check --workspace`; a failure or missing tool: send_message the PM "ti-minor BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/tiling.md rows ti-5, ti-6, ti-7, ti-19 and their sources. On the branch: plugins/chora-tiling (bar, notify placement, fullscreen), tests/gate-tiling-*.sh, docs/configuration.md tiling part.
Items (red gate test commit first, then the fix; no sleep, no retry; smallest change; KISS, POLS):
1. ti-5: a toast that starts while the chat is open is placed clear of the chat panel (read the chat state at start); red cell: toast rect overlaps chat rect.
2. ti-6 and ti-19 (same cause): the tooltips of Apps, Keys and power do not stay after a click or Escape; red cell: tooltip surface present 1 s after the click.
3. ti-7: with a fullscreen window the bar does not cover the window's first line (bar below the fullscreen window, or hidden while fullscreen, as tiling designs say); red cell: bar rect intersects the fullscreen window's top 20 px while the window is fullscreen.
Not yours: ti-1 (fuzzel pin), ti-11, ti-12, ti-14, ti-25 (later), other looks.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 if it moved; `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib,layout,tiling-audit,tiling-look,tiling-empty,chat-keys,doors`; poll in the same turn; never end a turn to wait. A red cell you did not cause: name it with its log line (FLK-* in backlog.md), do not rerun to green.
Done when: each item red sha, fix sha, cell, capture; findings/tiling.md rows `fixed <sha> (red <sha>) cell:<name>`; reports/ti-minor.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/ti-minor and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
