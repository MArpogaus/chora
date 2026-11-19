# pw r1 (P2 prerequisites): persona checker, stand-ins, round script (reports/p2-plan.md items 1, 2, 4)

Model: sonnet, effort high (harness work across act harness and persona session, routine but wide)
Lane: cloud, branch claude/pw from the train 74 merge a9e7084d. One round, stop at 250 k. Cost about 1.5 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "pw r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Read on claude/session-log: reports/p2-plan.md (all; sections 1, 2, 4 are your spec), reports/swarm-2/swarm.sh. Do not touch the cells of the running lanes flake-h, flake-k (hud), flake-l (xp 16c), temp-rest (test temp folders), nits-3. Product code is out of scope: tests/ and docs only.
1. session.sh stand-ins first (safety, red test first): wpctl, systemctl, grim as in tests/doors/lib.sh doors_fakes, logged to PERSONA_CALLS; a selftest proves that a persona session's `systemctl suspend` reaches the stand-in, never the host. Plus `FLAVOR+FLAVOR` for a second look.
2. tests/persona/check.py (about 180 lines + selftest): `Session.attach(env_file)` split out of harness.Session.setup without changing the acts gate; client `persona-check` granted by session.sh; after each act.py action with PERSONA_CHECK=1 it runs tests/acts/invariants.py (skip I4, I14 as in the plan) and writes $PERSONA_DIR/breaches.jsonl only; `check.py end TASK...` checks the T1-T7, X, L, M, C contracts of the plan as Expect objects.
3. tests/persona/round.sh (about 60 lines): swarm.sh without scratchpad paths: binary snapshot, 2 slots, PERSONA_KEEP, PERSONA_CHECK=1; a smoke mode that starts each of the 4 looks, runs one scripted act.py action and `check.py end` on a trivial task.
4. A planted breach (planted.py) shows in breaches.jsonl; the contract selftest goes into tests/test_acts.py; tests/persona/README.md says how a driver uses all three.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,gate-lib,acts,doors` green, plus the round.sh smoke on 4 looks. Before the final gate merge origin/claude/autonomous-dev-session-3w-i33pg7. Poll it in the same turn; never end a turn to wait.
Done when: items 1-4 hold with the commands that show them; gate green; reports/pw.md "## r1" on claude/session-log (each claim with its command or cell); head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/pw and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
