# mc-minor r1 (vision: stability, KISS; mc has no open BLOCK or MAJOR): one fade tracker in mc, and the mc-fix review MINORs

Model: sonnet, effort medium
Lane: cloud, branch claude/mc-minor at 929de144 (mc-fix r6, MERGE-OK). Step 1: `git merge --no-ff origin/claude/flakes-l` (cc1282db, MERGE-OK) and `git merge --no-ff origin/claude/autonomous-dev-session-3w-i33pg7`; a conflict you cannot resolve in a few lines: send_message the PM and stop. Stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "mc-minor BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/mc-fix.md (Review r1, rv-1..rv-5), reports/flakes-l.md (PM review r2, pr2-1).
Items, each a red test commit first, then the fix:
1. pr2-1 + rv-2: one fade tracker. The region cross-fade (region.rs ~720) and the 600 ms theme fade and build nodes (build.rs:221, apply_scenes ~43) go through `try_apply`, so lifecycle.rs `fading`/`writing` never see them; flakes-l added `fading_until` (busy "fade") as a second tracker. Route these commands through the same note (`note_writes` or one helper both paths call), drop `fading_until` and FADE_SLACK unless the core really needs the slack (then keep it as the grace of the one tracker, with a comment why). Red: a unit test that a theme change makes `busy` hold a `fade:` entry; keep mc_is_busy_while_its_regions_cross_fade green (adapt its matcher to `fade:`).
2. rv-1: lifecycle.rs:130 Remove clears `<id>-*` siblings (`foot` vs `foot-server`). Clear only the node and its real children (by the scene tree or an exact child list), not a string prefix. Red: unit test with `foot` removed while `foot-server` fades.
3. rv-3: tests/persona/check.py:364 T1/T4 accept the key list only when the persona opened it (`ctx.opened(sheet)` or its own chord). Red: a check.py selftest case with an unasked list.
4. rv-4, rv-5 NITs only if items 1-3 are done under 150 k.
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,mc-paths,mc-sofa,mc-column,mc-handover,acts` plus persona-selftest if check.py changed. Poll the gate in the same turn; never end a turn to wait.
Done when: each item red then green with shas; gate PASS lines; reports/mc-minor.md "## r1" with per item sha, test, cell; findings rows updated.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/mc-minor and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
