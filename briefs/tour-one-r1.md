# tour-one r1 (F7, RC 11): one tour code set, design/tour-v9.md section 8 lane 1

Model: sonnet, effort medium (folding copies into one set by a written plan; routine)
Lane: cloud, branch claude/tour-one at 9dc384b3 (session head). Stop at 250 k. Cost about 1.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "tour-one BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: design/tour-v9.md (all; section 0 and section 8 row 1 are the scope), reports/tour-plan.md "## PM review r1" (decisions on the open points).
Scope (section 8 row 1 only): one set `demo/tour/{tour,human,launch,slides,assemble,run}.py` and `demo/tour/chapters/{xp,tiling,mc,construct,handover}.py`; fold ch1-8, ch7b, v4-v8_*, tour4-8, assemble4-8 into it and delete the copies; keep the v8 fixes (PATH without writers, pane-centre aim, keep-bus, probe bus); act rows with expect, contract, cells, capture fields; `run.py --check` runs the rows without recording; a tools/copies.toml row so no `v*_` copy returns; diagnose F-1 (no notice in `choractl record`) in the report, fix only if under 20 lines.
Chapters: rows may be stubs marked `todo` for acts that need xp-give (not yet merged); CHAPTERS-A/B fill them later.
Red test first: a test that an act whose expect fails shows as a red timeline row (and the copies.toml check red on a `v9_` copy), then the code.
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,gate-lib,build,test,acts` plus any cell that imports demo/tour. Poll the gate in the same turn; never end a turn to wait.
Done when: line count before and after; copies 0; `run.py --check` output on all five chapters (green or `todo`); gate PASS lines; reports/tour-one.md "## r1" with shas.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/tour-one and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
