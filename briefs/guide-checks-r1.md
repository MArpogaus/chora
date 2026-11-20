# guide-checks r1 (F7, RC 11): design/tour-v9.md section 8 lane 3, the guide rules as checks

Model: sonnet, effort medium (checks by a written table; routine)
Lane: cloud, branch claude/guide-checks at 8b996cee (claude/tour-one r1, in review; merge the session branch and claude/tour-one when told). Stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "guide-checks BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: design/tour-v9.md sections 0, 7 and 8 row 3; design/looks (the rules cited).
Scope (section 8 row 3 only): the section 7 rows with an "Extends" entry: contrast 4.5:1 text and 7:1 key line and buttons (png-contrast.py on each text box of `scene.boxes`, into I6), text a person must read at least 14 px (into I6), gaps in whole units (hud-audit.py, tiling-audit.py on the boxes, into I2), words from tests/keysheet/words.table with no "agent" or "HUD" (into I10), screen-fixed nodes held in a camera move (I2 held). In invariants.py and the audits, no new checker where one exists.
Red test first: each new check gets a planted breach in tests/acts/planted.py that goes red, then the check. A check that turns red on the current session head for a real fault: do not fix the look; write it as a finding (file, act, capture, rule) in the report.
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,gate-lib,build,test,acts,hud,layout,tiling-audit` plus any cell that imports the changed checks. Poll the gate in the same turn; never end a turn to wait.
Done when: each row red (planted) then green with shas; real findings listed; gate PASS lines; reports/guide-checks.md "## r1" with shas.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/guide-checks and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
