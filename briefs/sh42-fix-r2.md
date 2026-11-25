# sh42-fix r2 (rc 1: sh-42 MAJOR; train134 sleep-ratchet breach): replace the two fixed sleeps of r1 with waits

Model: sonnet, effort medium (routine test edit)
Lane: cloud, claude/sh42-fix-r2 at 84f66d43 (session head, t133), merge b92de049 (claude/sh42-fix-r1) first. One round.
Step 0: `git fetch --unshallow`; `git merge --no-ff b92de049`; `cargo check --workspace`. A failure: send_message the PM "sh42-fix-r2 BLOCKED: <line>" and stop.
Stop at 120 k context; cost at most 0.6 %.
Why: train134 (84f66d43 + b92de049 + fbdcd6e2) fails tools/sleep-ratchet (sleep 783 vs list 779). r1 adds two fixed `sleep 2` in tests/gate-handover-shared.sh (review rsf1-1 join_tiling, rsf1-2 join-mc before Return).
Steps:
1) Replace both sleeps with a wait on the state the next step needs (ctl wait / a gate-lib wait helper, e.g. the focus holder or the window mapped), no new sleep anywhere. rsf1-3: add the unit test for the mc lost_to_other reclaim from a foreign node (plugins/chora-mc manage.rs:218).
2) tools/sleep-ratchet, tools/size-ratchet, tools/lint-ratchet pass on your head (counts never raised).
3) handover cells 10x in a row green (`tests/gate-handover-shared.sh` loop), then one full tests/merge-gate.sh (no --only).
Done when: the three ratchets pass, 10/10 handover, full gate PASS on one run.
Report: reports/sh42-fix.md "## r2" on claude/session-log (ratchet output, 10x table, gate run dir). send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines): head sha, ratchets, gate.
Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/sh42-fix-r2 and the report on claude/session-log (fetch and merge on reject, no force); author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
