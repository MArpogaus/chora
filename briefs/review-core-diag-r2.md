# review core-diag r2 (rc 9: sl-3, sl-1): seeded review of claude/core-diag before its merge

Model: opus, effort high, because the diff changes the scene core (root band order, unlink) and the allocator setup of the core process; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/core-diag head 788605f4, base fc779a76 (r1 repro scripts on 3b14c825), 4 files +108 -12. One round. Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run.
Step 0: `git fetch --unshallow || true`, `cargo check --workspace`; a failure: send_message the PM "review-core-diag BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/core-diag-r2.md (done criteria) and reports/core-diag.md "## r1" and "## r2" on claude/session-log; `git diff fc779a76..788605f4`.
Checks, each with its command and files:
1) Scene::band (crates/chora-scene/src/scene.rs): the partition_point searches give the same band as the old take_while walks for every root order that link and set_order can make; name any other writer of the root children (grep `children` writes in scene.rs and crates/chora). `cargo test -p chora-scene`.
2) Scene::unlink: position + remove keeps sibling order; a node not in the list is handled as before.
3) mallopt in main (crates/chora/src/main.rs): first call, before any thread or allocation that matters; SAFETY line true; unsafe_code allow not widened beyond main. `cargo test -p chora`.
4) Red first: 722d5baa holds the three tests before d845a7f5 (`git log --oneline fc779a76..788605f4`); bounds not flaky on a loaded debug host (read the margins in the report).
5) The lane gate's acts red (construct face-hint "the hint keys names no key"; I6 head-6 11 px under co-42): could this diff cause it (band order of construct face nodes, unlink order)? Read tests/acts/looks/construct.py around the hint act and the construct plugin's use of root order; answer yes, no or unknown with the reason. Do not run the gate.
6) Ratchets: tools/lint-ratchet stays 360; no file over 1,500 production lines.
Known, not findings: the construct hint act is a suspected flake (ldm-2, lm-2), lane flake-hint r1 works on it.
Verdict computed: an open BLOCK or MAJOR is FIX; a failed Done-when line of briefs/core-diag-r2.md (the gate is not green) is INCOMPLETE unless check 5 shows the red is foreign; else MERGE-OK. Findings one line each (cd-N, severity, file:line, text, fix in one sentence). Write reports/review-core-diag-r2.md on claude/session-log.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, check 5 answer. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash.
