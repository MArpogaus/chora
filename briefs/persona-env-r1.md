# persona-env r1 (RC2): the persona session has what a real xp session has

Model: sonnet, effort medium (harness work in shell and Python, no product change)
Lane: cloud, branch claude/persona-env from a2f50532 (train 78b head: pw r2 in). One round, stop at 250 k. Cost about 0.6 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace` (then `cargo build --workspace`); a failure or a missing tool: send_message the PM "persona-env r1 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/p2-r1-xp.md "## r1" and "## PM review r1", feedback/2026-11-20-p2r1-*.md, backlog.md H-persona-env; tests/persona/README.md and round.sh, session.sh, check.py; tests/doors/lib.sh doors_fakes (the pattern for stand-ins).
In tests/persona (session.sh or round.sh), smallest change:
1. Shared scripts: the session PATH holds the flavor's shared scripts (lock, screenshot, volume) as installed by a package, so T4 lock and screenshot measure chora, not the harness; their system calls (loginctl, swaylock, grim, wpctl) are recorded stand-ins as in doors_fakes, and check.py reads those marks.
2. Editor: a GUI editor stand-in (a small Wayland client from the workspace, e.g. chora-probe with a title "Untitled - Notepad", installed as `mousepad`) so T1 measures the Start menu path; with no stand-in, xp's nano fallback stays (xp-25 is a product item, not this lane).
3. Probe window for X2: a window with a title bar the xp look decorates (server-side frame), so a drag can move it; check.py compares the rect.
4. Helper: the fake voice agent answers "hello" (one entry in voice/entries.json is not enough: T3 waits for a working then idle state with an answer).
5. tests/persona-selftest.sh: a cell per item (stand-in present on PATH, editor starts, probe draggable, helper answers); red first (own commit), then the change.
Poll each run in the same turn; never end a turn to wait.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib` and `bash tests/persona-selftest.sh`; all PASS. Then one smoke: `tests/persona/smoke.sh` on xp, T1, T3, T4 lock, X2 contracts pass with a scripted driver (no haiku persona).
Done when: items 1-5 each name red commit, change commit and selftest cell; smoke result in reports/persona-env.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/persona-env and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
