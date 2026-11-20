# review xp-p2r3 r1 (RC 2, shared bar "no stray keys"): seeded review of claude/xp-p2r3 before its merge

Model: sonnet, effort high (keyboard grabs in xp, kit and construct; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/xp-p2r3 head f89f785c, base ab4cbb4f (co-p2-fix; 18 files, +570 -60). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No full merge gate (the lane ran 12 cells green).
Step 0 (no setup script): `git fetch --unshallow || true`; `git fetch origin claude/session-log claude/xp-p2r3`; `cargo check --workspace`; a failure: send_message the PM "review xp-p2r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/xp-p2r3-r1.md (done criteria), reports/xp-p2r3.md "## r1", reports/co-p2-fix.md (rw-1, rw-2), reports/xp-fix.md (XPF-R1 rv-1, rv-2), findings/construct.md rows co-25, co-26; then `git diff ab4cbb4f..f89f785c`.
Checks, each with its command and files:
1) Red first: 06c900b6, f41199a4, 8357b193 each fail on their parent's code and pass at the fix (`cargo test -p <crate> <test>` per sha in a scratch worktree; the cube cell case by reading tests/gate-construct-cube.sh).
2) The gate change: tests/gate-construct-cube.sh had three `known()` uses (co-25 twice, co-26); all are plain asserts now. Is co-26 really fixed on this head, or did the assert pass for another reason? Name the code path.
3) plugins/chora-xp/src/wm.rs grab_keys: grab_swallowed runs only on Flat or Hosted cube keys and after xp's own grabs; a chord xp binds (Start, key list, helper) still reaches xp; Super alone still opens Start (POLS); no grab leaks after the face is left (end_grabs).
4) New `Look::Swallow` (serde skip, does nothing): is it a hidden setting or dead variant? KISS: say whether a plain function would do; MINOR at most.
5) Kit Launches `bare` counter (5449aa27): two starts at once, a start that exits without a window, a window with a stranger token: the Starting message ends right in each; no leak of entries.
6) plugins/chora-construct regrab `in_pane() && cube.is_none()`: a pane inside an open cube and a flat pane both behave as the brief says; test in cube/tests_moved.rs.
7) `cargo test -p chora-xp -p chora-kit -p chora-construct` and `cargo clippy --workspace --all-targets -- -D warnings`; no file over 1,500 lines.
8) The pre-existing "Start opens on release of Super after a swallowed chord": is it a stray key into the user's path (POLS)? Severity with reason; it is known (backlog CO-P2-FIX r1), so a finding only if this branch makes it worse.
Known, not findings: the open xp MINORs in findings/xp.md, xp-24 (fix lane XP-24).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv-N, severity, file:line, text, fix in one sentence). Append to reports/xp-p2r3.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
