# flake-k r1 (F5): hud flake of train 73b (construct-host-hud, chat-long-task missing)

Model: sonnet, effort high (routine flake fix, reproduce then fix)
Lane: cloud, branch claude/flake-k from the train 72 merge 219d5afe. One round, stop at 250 k. Cost about 0.7 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "flake-k r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Read on claude/session-log: backlog.md row "F5 hud flake 73b". Do not touch the cells of the running lanes flake-h (layout HUD audit, doors helper, construct pull pane), flake-j (tiling-audit fullscreen) and temp-dirs (test temp folders).
The failure (train 73b, `--jobs 3`, the full gate beside other steps): the hud step printed
`construct-host-hud: a done of another id closed the question` then `construct-host-hud missing 4s`, and
`timeout 5s: header_capped atlas · working · Read this first.` / `chat-long-task meta: A question is waiting.` then `chat-long-task missing 6s`. Alone the hud step passed (71 s).
1. Reproduce: the hud step 30 times under load (`--jobs 3` with a parallel `cargo build --workspace` of a second target dir beside it, or stress); name the count per cell.
2. Find the cause per cell: construct-host-hud: why does a done of another id close the question (a shared id or a stale done from an earlier act?); chat-long-task: why is the header still "working" after 5 s (a fixed timeout too short under load, or a lost event?). A real product fault (a done closes the wrong question) is a finding: red test first, then the fix in the plugin.
3. Fix: bounded waits on the state (until_ok of the gate lib), no retry of the whole cell, no longer plain sleep.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,gate-lib,hud` green, then hud twice more under load. Poll it in the same turn; never end a turn to wait.
Done when: both cells 0 of 30 under load after (counts before and after), or a named product finding with its red test and fix; gate green; reports/flake-k.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/flake-k and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
