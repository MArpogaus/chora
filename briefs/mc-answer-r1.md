# mc-answer r1 (RC, vision bar): long voice answer and alarm card at sofa distance, mc-column gate

Model: sonnet, effort high (two layout faults in the host hud and notify slot; each needs a red cell)
Lane: cloud, branch claude/mc-answer from 92ecd30a (mc-majors r1 head). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace` (then `cargo build --workspace`); a failure or a missing tool: send_message the PM "mc-answer r1 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/mc.md mc-39, mc-40, reports/mc-majors.md "## r1" (seen on the way 1-3), findings/README.md (severity), vision-card.md (mc: sofa and stove distance).
1. mc-39: red cell first in tests/gate-mc-sofa.sh (`answer-long`: a 150-character `voice.say`; the answer rect must not overlap the date, the column (x >= 860) or the page title row; at most 2 lines at the mc-34 size), its own commit; then the fix in the hud layout (max width or a place of its own), smallest change, own commit.
2. mc-40: red cell (`alarm` extended or `card-size`: card text at least the page body size, ink 7:1 by tests/png-contrast.py), then the fix in plugins/chora-mc/notify (slot config or assets) only.
3. tests/gate-mc-column.sh: `ctl wait owner:notify` times out (no node until a notice shows); wait by `ctl call notify state.get` as gate-mc-sofa does. Then run it green, and add it to tests/merge-gate.sh as cell `mc-column` (header comment too), its own commit.
4. findings/mc.md rows `fixed <fix sha> (red <sha>) act:<cell>` on claude/session-log.
Poll each run in the same turn; never end a turn to wait.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,acts,mc-paths,pixels,mc-sofa,mc-column`; all PASS.
Done when: mc-39 and mc-40 each name red commit, fix commit and passing cell; mc-column green in the merge gate; reports/mc-answer.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/mc-answer and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
