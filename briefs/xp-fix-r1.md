# xp-fix r1 (RC 2, shared bar): xp-11, xp-35 and the r2 fix check, plus the persona harness gaps

Model: sonnet, effort high (two product faults in one look plus four harness gaps; each needs a red test)
Lane: cloud, branch claude/xp-fix from 79c8d2b5 (session branch, train 81). One round, stop at 250 k. Cost about 1.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace` (then `cargo build --workspace`); a failure or a missing tool: send_message the PM "xp-fix r1 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/xp.md xp-11, xp-26, xp-27, xp-33, xp-35; reports/p2-r2-xp.md "## r2"; feedback/2026-11-20-p2r2-{erika,murat,pixel}.md; findings/README.md; vision-card.md.
Product items (red act or cell first, own commit; then the fix, own commit; xp plugin only unless the fault is in kit or core, say so; fallback art only, never commit Microsoft files):
1. xp-11 MAJOR: Help window: Esc closes it; the right pane is filled on the first topic; the key table lists every kit keysheet row (All keys, Lock the screen, Power menu, Take a screenshot, Volume up/down, Mute, Start a program or run a command) and scrolls or fits at 1280x800. Read the capture first.
2. xp-35 MAJOR: the "Starting ..." balloon never overprints a menu or popup (I5) and goes when the started window maps. Red: an act opening the volume menu while the balloon shows, I5 check.
Fix check (r2 left them unreached): xp-26 (tooltip and tick follow the level), xp-27 (question card), xp-33 (Super+F1 opens the key list): one act each that drives the exact path; pass, or a red act and a fix as above.
Harness items (tests/persona, tests/acts; own commits): a. the wpctl stand-in keeps the last set level (not a constant 0.50); b. the Notepad stand-in shows typed text; c. the persona checker closes or ignores a Help window left open before contracts T2, X2, T4 (no red from it); d. the probe window gets a title bar or is excluded from X2 drag targets; e. the swaylock stand-in draws a visible lock.
Poll each run in the same turn; never end a turn to wait.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,acts,xp,xp-faces,xp-paperclip,xp-question,xp-fields,xp-notepad,doors,keys`; all PASS. A red cell you did not cause: rerun it alone once and name it.
Done when: each item names red commit (or guard), fix commit and passing cell; gate green; findings/xp.md rows `fixed <fix sha> (red <sha>) act:<name>`; reports/xp-fix.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/xp-fix and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
