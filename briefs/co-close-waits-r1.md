# co-close-waits r1 (rc 1: co-75, co-76, co-79 train; t134 sleep-ratchet breach): the three fixed sleeps of gate-construct-close.sh become waits

Model: sonnet, effort medium (routine test edit)
Lane: cloud, claude/co-close-waits-r1 at 7f519bc8 (co-reach r1 head, PM review MERGE-OK; holds co-close r2). One round.
Step 0: `git fetch --unshallow`; `git merge --no-ff 84f66d43` (session head, t133) if not an ancestor; `cargo check --workspace`. A failure: send_message the PM "co-close-waits-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Stop at 120 k context; cost at most 0.5 %.
Why: tools/sleep-ratchet on 84f66d43 + this branch counts 3 sleeps too many: tests/gate-construct-close.sh:51 (`sleep 5`, the hints after their 3 s), :80 (`sleep 1`) and :87 (`sleep 2`) (both: "Super+x closed nothing else").
Steps:
1) :51: wait on the state the next line reads (e.g. until_ok on the hint bar's rect or its resting set). :80, :87 (a negative check): wait on a sync point that proves the core handled the key (e.g. a ctl round trip, a construct log or act counter, the face's state after the key) before asserting the pane lives; no new sleep anywhere.
2) Prove each check still fails when the fault is back: revert f0ab2938 / 272940fc locally, the cell turns red; note it in the report.
3) tools/sleep-ratchet, tools/size-ratchet, tools/lint-ratchet pass on your head with 84f66d43 merged (counts never raised). gate-construct-close.sh 10x in a row green, then one full tests/merge-gate.sh (no --only).
Done when: the three ratchets pass, 10/10 close cell, the reverted fault turns it red, full gate PASS on one run.
Report: reports/co-close.md "## waits r1" on claude/session-log (ratchet output, 10x table, red proof, gate run dir). send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines): head sha, ratchets, gate.
Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/co-close-waits-r1 and the report on claude/session-log (fetch and merge on reject, no force); author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
