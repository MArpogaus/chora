# gate-pick r1 (F6): the gate cells a diff touches

Model: sonnet, effort medium (a small shell tool with a selftest)
Lane: cloud, branch claude/gate-pick from a9fd91f3 (id-gate head; it adds `merge-gate.sh --list`). One round, stop at 250 k. Cost about 0.5 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "gate-pick r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Why (train 73 red 11-19 21:17 UTC, backlog "HARNESS gate-pick"): a lane changed the line chora-notify prints; its brief's `--only` list had no xp cell, and tests/xp.sh step 17 greps that line. The train found it, an hour late.
Build tests/cells-for.sh: `tests/cells-for.sh [--base <ref>]` (default base origin/claude/autonomous-dev-session-3w-i33pg7) prints a comma list for `merge-gate.sh --only`:
1. always pre-commit, ci-check, build, test, clippy;
2. each cell whose command (from default_steps; read it with a small `--commands` print in merge-gate.sh or by parsing `--list` plus the heredoc; KISS) names a changed tests/ file, or a tests/ script it runs sources a changed tests/*-lib.sh;
3. each cell whose script, or a lib it sources, names a binary of a changed crate (plugins/chora-X -> `chora-X`; services/voice -> chora-voice; crates/chora -> every cell; crates/chora-kit or chora-scene -> every cell);
4. cfg(test)-only Rust changes may still pick cells: correctness over speed.
Red first: tests/cells-for-selftest.sh with cases in a temp git repo (a change to plugins/chora-notify picks xp; a change to tests/gate-hud.sh picks hud; a core change picks all; a docs-only change picks only the fixed five); run it from the gate-lib step. Check on the real repo: for 219d5afe..88f441e4 (kit-minors r1) it must print xp.
Docs: one paragraph in CONTRIBUTING.md (where the gate is described): a lane runs `tests/merge-gate.sh --only "$(tests/cells-for.sh)"`.
Do not touch the red-first or identity steps (lane ig-fix owns gate-lib's selftest call; if both edit the gate-lib line, keep yours a separate `&&` term at its end).
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,red-first,identity,gate-lib,build` green, then `tests/cells-for.sh` on your own diff. Poll it in the same turn; never end a turn to wait.
Done when: selftest red then green; gate green; reports/gate-pick.md "## r1" on claude/session-log with the selftest cases and the 88f441e4 output; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/gate-pick and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
