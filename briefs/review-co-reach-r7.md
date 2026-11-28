# review co-reach r7 (rc 1, rc 2 construct; co-79 MAJOR): seeded review of claude/co-reach-r7 before its merge

Model: opus, effort high, because it changes the construct key rule in and after a pane and adds a timed wait on xp (a look and design review; procedures: opus high)
Red on head: none (a review; the lane's red 61741663 precedes its fix 2e521bbc)
Lane: cloud, read-only on code: claude/co-reach-r7 at 008714ba, review dc532d60..008714ba (11 files, +322 -61: chora-construct pull.rs, overlay.rs, hints.rs, input.rs, cube.rs, construct.rs, reach_tests.rs; tests/gate-construct-cube.sh, gate-construct-layout.sh, gate-construct-keys.sh; docs/cube-modes.md). One round. Stop at 120 k context; cost at most 0.5 %; at most 8 findings.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-co-reach-r7 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/co-reach-r7.md (done criteria), reports/co-reach.md "## r7" and findings/construct.md co-79 on claude/session-log.
Checks, each with its command and files (no merge gate; `cargo test -p chora-construct`, `cargo clippy -p chora-construct`, and `tests/gate-construct-cube.sh` once on head):
1) Red first: the unit test of 61741663 and gate-construct-cube.sh step 10 fail on dc532d60 (run or name its log) and pass on 008714ba; else MAJOR.
2) co-79: one predicate (`pane_to_face`) for every pane path (report table section 4); read pane_to_face, put_pane, put_in_face: one rule or a special case per path (KISS)?
3) The wait (`Host::put_later`, 150 ms poll, 250 ms state.get answer, 20 s end): can it put a window the person no longer means (focus moved, pane closed, second key press, step in elsewhere), leak a timer, or block the frame loop? Does an unput end tell the person anything (honest state), or fail silently?
4) Gate changes outside the area: gate-construct-layout.sh (expected hint strings, refused-give cell now presses on the floor first) and gate-construct-keys.sh (Super+Alt+1 instead of +2): does each keep what the cell tested (a refused give says why), or is it a loosened assert? Is the new put path itself covered where those cells used to press Super+Alt+2?
5) POLS: Super+Alt+2 in a pane now opens a cube and puts the window instead of giving it to xp; is a give to xp still reachable and told where a person expects it (vision card: construct holds the face turn, flat fallback)? Hints: at most six in every set.
6) Docs: docs/cube-modes.md says what the code does, wait included.
Known, not findings: rpt2-2, sh-37..sh-47, TF-1, the doors helper-none flake (foreign, run 2).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rcrr7-N, severity, file:line, text, fix in one sentence). Append to reports/co-reach.md under "## Review r7".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
