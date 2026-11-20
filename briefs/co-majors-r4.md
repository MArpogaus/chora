# co-majors r4 (shared RC bar "no stray keys"; rule 6, construct MAJOR): co-25, co-26

Model: sonnet, effort high (one key-routing fault with a probe, one verify; each needs a red cell, then the fix)
Lane: cloud, branch claude/co-majors-r4 from d81c8293 (session head, train 87e). One round, stop at 250 k. Cost about 1.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow || true`; `cargo check --workspace`; a failure or missing tool: send_message the PM "co-majors r4 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/construct.md co-25, co-26; reports/p2-r1-construct.md "## r1b" (T2, T5, probe); reports/verify-maj.md (co-26 part); docs/configuration.md:324-330 (tiling swallows unbound Super chords). On the branch: plugins/chora-construct (input, keys, hud), tests/gate-construct-keys.sh, tests/gate-keys.sh.
Items (red gate test commit first, then the fix):
1. co-25 MAJOR: in a construct face (walk mode and with pane focus) Super+D, Super+Q, Super+Tab and any unbound Super chord reach the client (probe saw keys 125, 32, 16, 15). Red: a cell `construct-keys stray-super` with a probe client that records keys; assert no Super chord arrives. Fix: construct swallows unbound Super chords as tiling does (POLS); bound ones act. Docs: the construct section names the rule.
2. co-26 MAJOR verify: Super+F1 with pane focus in a face. Measure in gate-construct-keys first; red if the key list does not draw, then fix; if it draws, close as not-a-bug with the capture.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 if it moved; `tests/merge-gate.sh --jobs 2 --only pre-commit,ci-check,build,test,clippy,gate-lib,keys,construct-hud,construct-extra,cube,walk,pane-keys,doors`; poll in the same turn; never end a turn to wait. A red cell you did not cause: name it with its log line (FLK-* in backlog.md), do not rerun to green.
Done when: each item red sha, fix sha, cell, capture; findings rows `fixed <sha> (red <sha>) cell:<name>`; reports/co-majors.md "## r4" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/co-majors-r4 and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
