# review co-reach r6 (rc 1, rc 2 construct; co-79 MAJOR): seeded review of claude/co-reach-r6 before its merge

Model: opus, effort high, because it changes the construct mode rule after Alt+Escape (a look and design review; procedures: opus high)
Red on head: none (a review; the lane's red 5f8f0c84 precedes its fix 1916f517)
Lane: cloud, read-only on code: claude/co-reach-r6 at bcecb671fefce844b1cf25759c934dc6a5dc8347, review 4d8bbe36..bcecb671 (8 files, +190 -28: chora-construct hints.rs, input.rs, modes.rs, overlay.rs, reach_tests.rs; tests/gate-construct-cube.sh, gate-construct-layout.sh; docs/cube-modes.md). One round. Stop at 120 k context; cost at most 0.5 %; at most 8 findings.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-co-reach-r6 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/co-reach-r6.md (done criteria), reports/co-reach.md "## r6" (c5e6a217) and findings/construct.md co-79 on claude/session-log.
Checks, each with its command and files (no merge gate; `cargo test -p chora-construct`, `cargo clippy -p chora-construct`, and `tests/gate-construct-cube.sh` once on head):
1) Red first: gate-construct-cube.sh step 8 of 5f8f0c84 fails on 4d8bbe36 (run it there or name its log) and passes on bcecb671; else MAJOR.
2) co-79: one rule for every pane path (Terminal row, All programs, launcher, Super+t in space and in a pane, Programs, pull, face path): after Alt+Escape the bar tells a key that works and leads to a window in a face. Read step_out, cube_window, cube_key: one rule, or a special case per path (KISS)?
3) POLS: Alt+Escape from a pane pulled out of a face now stands back instead of landing in use; does any earlier path change behaviour a person relies on (r5 step 7, face path cells)? Super+C noting the pane window: can it put a wrong window into a face (two panes, focus elsewhere)?
4) Hints: the walk holds at most six; the layout gate's three pinned strings were changed outside the area: is the change honest (the hint is shown), not a loosened assert?
5) Launcher stood back 2/5 (open point 1, also on base): is it a separate finding of its own (name severity), not caused by this diff?
6) Docs: docs/cube-modes.md says what the code does.
Known, not findings: rpt2-2, sh-37..sh-45, TF-1, launcher-mc mc dark flake.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rcrr6-N, severity, file:line, text, fix in one sentence). Append to reports/co-reach.md under "## Review r6".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
