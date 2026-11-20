# co-pane r4 (RC 2, construct; F4 lint ratchet): make the reviewed co-pane branch pass lint-ratchet after lint-down

Model: sonnet, effort medium (a mechanical lint fix and a test wait on a reviewed branch)
Lane: cloud, claude/co-pane at 18417f06 (review r3 MERGE-OK, holds co-fix). Cost at most 0.4 %; stop at 150 k context.
Why: train 92 (20:02 UTC) failed on ci-check lint-ratchet: lint-down (78ec62e1) set tools/lint-ratchet.list (eprintln only falls); co-pane adds `eprintln!("chora-construct: the tap of the modifier: {err}")` (plugins/chora-construct/src/camera.rs).
Step 0 (no setup script): `git fetch --unshallow`, `cargo check --workspace`, then `cargo build --workspace`; a failure: send_message the PM "co-pane r4 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
1) `git fetch origin claude/lint-down && git merge --no-ff 78ec62e1` (message "Merge branch 'claude/lint-down' into claude/co-pane"); `tools/lint-ratchet` shows the breach (that is the red).
2) Fix: the grab error must stay visible once (POLS: honest failure); keep eprintln within the list the cleanest way (an existing log path of chora-construct, or remove an equal number of redundant eprintln in chora-construct without losing a message). No other code change.
3) rx-2 NIT (reports/co-pane.md "## Review r3"): tests/gate-construct-pull.sh:557 bare `sleep 1` before the no-Start-menu check becomes a wait on a condition or a settle helper of tests/gate-lib.sh.
4) Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,cube,pull,placed,construct-hud,construct-extra,wm-pane,pane-keys,walk,keys,xp`. Poll in the same turn; never end a turn to wait.
Report: append "## r4" to reports/co-pane.md on claude/session-log (shas, gate cells, what replaced the eprintln); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/co-pane and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
