# mc-majors r1 (RC, vision bar): the five open mc MAJORs from journey-1

Model: sonnet, effort high (five product faults in one look; each needs a reproduction and a red act)
Lane: cloud, branch claude/mc-majors from a61e8eaa (train 76). One round, stop at 250 k. Cost about 1 %.
Step 0: run setup once: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "mc-majors r1 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/mc.md mc-33 to mc-37, findings/README.md (severity), vision-card.md (mc: sofa and stove distance, honest helper states).
Each of mc-33 (chat door by mouse opens the chat), mc-34 (voice HUD text readable at distance: at least the body size of the page, ink not grey), mc-35 (a critical alarm is a card, the column keeps its order), mc-36 (helper pill honest while answering, no overlap with the status word), mc-37 (Super+n hides the column and the page widens):
1. Reproduce on the current head first (tests/acts/looks/mc.py or a pixel cell). The finding is from tour v4: if it no longer reproduces, write a green act that guards it, mark it `fixed <sha of the guard> (gone before)` and move on.
2. Else a red act or cell first (its own commit), then the fix (its own commit), smallest change, mc plugin only unless the fault is in kit (say so).
3. findings/mc.md: each row `fixed <fix sha> act:<name>` on claude/session-log.
Poll each run in the same turn; never end a turn to wait.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,acts,mc-paths,pixels`; all PASS.
Done when: each of the five names its red commit (or guard), fix commit and passing cell; gate green; reports/mc-majors.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/mc-majors and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
