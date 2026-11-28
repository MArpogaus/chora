# review co-reach r8 (rc 1, rc 2 construct; co-79 MAJOR): seeded review of claude/co-reach-r8 before its merge

Model: opus, effort high, because it changes the construct key rule with xp running and moves the put wait into a thread on its own connection (a look and design review; procedures: opus high)
Red on head: none (a review; each fix of r8 has its red commit first: ddaac07c, a33bdffc, e9b64b4a, d8621fe9, 98fa47a9, b3fe5216)
Lane: cloud, read-only on code: claude/co-reach-r8 at 830ab836, review 008714ba..830ab836 (15 files, +721 -71: chora-construct pull.rs, put.rs (new), names.rs, overlay.rs, input.rs, hints.rs, main.rs, cube.rs, cube/start.rs, cube/construct.rs, scene.rs, reach_tests.rs, tests_construct.rs; tests/gate-construct-cube.sh; docs/cube-modes.md). One round. Stop at 120 k context; cost at most 0.4 %; at most 8 findings.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-co-reach-r8 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/co-reach-r8.md (done criteria), reports/co-reach.md "## Review r7" and "## r8" on claude/session-log.
Checks, each with its command and files (no merge gate; `cargo test -p chora-construct`, `cargo clippy -p chora-construct`, and `tests/gate-construct-cube.sh` once on head):
1) Red first: each red commit's tests fail on its parent and pass on 830ab836 (run the 5 unit reds on 008714ba; cube gate step 11 logs named in the report); a fix without its red before it is MAJOR.
2) rcrr7-1: with xp running (manager pane or client) Super+Alt+2 from a pane gives to it and the bar tells `To XP`; is `face_manager_elsewhere()` one predicate, or are client names (clients.list plus connected/disconnected) able to go stale (xp restarts, a client named xp that is not the face manager)?
3) The wait (put.rs, `construct-ask-N` thread): thread or connection leak on every end (20 s, close_cube, window gone, step in, move on), a 2 s re-ask that piles up questions, an answer read after the put ended; the frame loop never blocks on xp.
4) Honest ends (rcrr7-4): every unput end tells a notice once, De words beside; no double notice on close_cube.
5) Open points the builder named: after the give to the manager-pane xp the camera stays on the floor, the pane may be out of view, and the window keeps the keyboard; the ask connection shows in clients.list while waiting. Rate each by POLS (MINOR, MAJOR or not a finding), with a fix in one sentence.
6) Gate: cube gate step 11 (a) and (b) assert told reasons, not loosened; hints at most six in every set; docs/cube-modes.md says what the code does.
Known, not findings: rpt2-2, sh-37..sh-47, TF-1, the doors helper-none tiling flake (foreign), rcrr6-1 docs/configuration.md hints.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rcrr8-N, severity, file:line, text, fix in one sentence). Append to reports/co-reach.md under "## Review r8".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
