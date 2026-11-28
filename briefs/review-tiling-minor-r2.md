# review tiling-minor r2 (rc 1b, F7; rtmr1-1..3): seeded review of claude/tiling-minor-r2 before its merge

Model: opus, effort high, because it is a look review that touches the shared launcher in chora-kit (procedures: opus high for look reviews)
Red on head: none (a review; the lane's reds 433fc761, 3fce00cd precede their fixes ab985149, cda21a03; rtmr1-2 red by mutation)
Lane: cloud, read-only on code: claude/tiling-minor-r2 at ad07ec70, review bd641acb..ad07ec70 (6 files, +367 -32: chora-kit launcher.rs, tests/give_notice.rs; flavors/tiling/look.sh; chora-tiling main.rs, tests/apps_launcher.rs; tests/gate-tiling-apps.sh). One round. Stop at 120 k context; cost at most 0.4 %; at most 8 findings.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-tiling-minor-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/tiling-minor-r2.md (done criteria), reports/tiling-minor.md "## Review r1" and the r2 section of the lane (rtmr1-1..3) on claude/session-log.
Checks, each with its command and files (no merge gate; `cargo test -p chora-kit -p chora-tiling -p chora-assistant`, `cargo clippy` on those, and `tests/gate-tiling-apps.sh` once on head):
1) Red first: 433fc761 and 3fce00cd tests fail on bd641acb and pass on ad07ec70 (run them); else MAJOR. rtmr1-2: does the door test fail when the SIGRTMIN+2 send (plugins/chora-assistant/src/main.rs:885 or the tiling side) is removed?
2) rtmr1-1: every notice a person sees in flavors/tiling free of "choractl" and tool names (grep); words in the look's own voice and in both languages where the look has two.
3) rtmr1-3: the Apps list and the Looks picker use one rule for "a window has focus" (one place, KISS), not two copies; a failed scene.query counts as a window in both. Does the kit launcher change alter any other look's launcher (xp, mc, construct)? Check callers.
4) Nothing loosened: diff tests/ and gate files; no wait, timeout or cell removed; the gate's new window is a cat with no new sleep.
5) The report: the docs SKIP and the two named flakes (doors tiling helper-none, pull) are honest and named; the power.sh:29 flag (locker command line in a notice body) is real? If so a finding (severity by POLS).
Known, not findings: docs SKIP (mkdocs absent), sh-32/sh-46 acts I14 flake, sh-43, sh-45.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtmr2-N, severity, file:line, text, fix in one sentence). Append to reports/tiling-minor.md under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
