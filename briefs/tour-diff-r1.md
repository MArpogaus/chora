# tour-diff r1 (F7, RC 11): design/tour-v9.md section 8 lane 2, the v8 comparison and the video checker

Model: sonnet, effort medium (a diff tool and a checker by a written plan; routine)
Lane: cloud, branch claude/tour-diff at 8b996cee (claude/tour-one r1, in review; merge the session branch and claude/tour-one when told). Stop at 250 k. Cost about 1.3 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "tour-diff BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: design/tour-v9.md sections 0, 6 and 8 row 2; reports/tour-one.md "## r1" (the act rows API).
Scope (section 8 row 2 only): tests/tour-diff.py (extends tests/capture-diff.py: per act and capture the scene diff and the pixel diff, kinds as section 6), demo/tour/intended.toml (change rows: act, kind, cause), tests/tour-video-check.py (extends the v4 freezedetect use: black frames, frame flash, freeze, screen-fixed nodes still in a camera move); planted cases for each: a moved node, a changed blob, a flicker frame, a freeze; no recording (RECORD lane 6).
Red test first: a test per planted case that the tool flags it (red), then the code. Reuse capture-diff and existing helpers (F2); no new copy (tools/cut-check).
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,gate-lib,build,test,acts` plus any cell that imports the new tools. Poll the gate in the same turn; never end a turn to wait.
Done when: each planted case red then green with shas; the tools run on two stored v8 captures (name them) with output in the report; gate PASS lines; reports/tour-diff.md "## r1" with shas.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/tour-diff and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages (a missing tool such as ffmpeg is a blocker: report it).
