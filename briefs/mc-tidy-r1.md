# mc-tidy r1 (F7, rc 11; mc MINOR, rule 6: mc has no open BLOCK or MAJOR): four open MINOR rows of chora-mc

Model: sonnet, effort medium (routine fixes with red asserts in existing mc cells; no new feature)
Lane: cloud, branch claude/mc-tidy from 5b34d4b1 (session head, t114 green). Stop at 250 k context; cost at most 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true; cargo check --workspace; fuzzel --version`; a red check: send_message the PM "mc-tidy BLOCKED: <line>" and stop (no apt).
A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/mc.md rows mc-50, mc-55, mc-56, mc-57; reports/record-v9.md "## r4"; reports/gate-alive.md "## r1"; design/looks/mc*.md; design/tour-v9.md section 7 (word table); vision-card.md (chora-mc).
Scope (these rows only):
1) mc-56: with the chat open the column's doors (Programs, Lock, Power, Volume) stay in the scene with live hit areas under the chat panel (contrast 1.00:1). Decide KISS/POLS: hide them and drop their hit areas while the chat is open, or keep them visible beside the chat. A click where a hidden door sits must not act. Red assert first (tests/gate-mc-sofa.sh or gate-mc-paths.sh), then the fix.
2) mc-55: the helpers chapter says 'AGENTS'; use the word of the word table (helper). Red first (the I10 cell or a unit test on the label), then the fix.
3) mc-50: yellow square after the first Return stays; XF86Back on a window page needs two presses. Red assert per symptom first; a symptom that cannot be reproduced headless: say why, it stays open.
4) mc-57: tests/gate-mc-host.sh:185 (known-red-mc-host, 3 of 3 red): find the cause (injected key or no helper registered by the fake agent); fix the product or the fake, then remove the known-red entry. A test may not be loosened.
No change to other looks.
Gate: the full `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` as a background task (no --only; product code), poll it in the same turn; green; merge origin/claude/autonomous-dev-session-3w-i33pg7 first if it moved; push the branch.
Report: reports/mc-tidy.md "## r1" on claude/session-log: per row red sha, fix sha, cell, captures (paths), what stays open.
Send the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 6 lines with send_message. Poll every command in the same turn; never end a turn to wait.
Rules: commits as Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify;
red test commit before each fix; push only claude/mc-tidy and claude/session-log (fetch and merge on reject, no force); no system packages.
