# review co-r4 r1 (RC 2 construct, stability: co-30 core grab rule, co-28, rv-1..3): seeded review of claude/co-r4 before its merge

Model: opus, effort high because co-30 changes the core rule for button grabs (input.rs, ipc.rs button_grab_for) that every look depends on (rules.md 7: the builder never reviews itself)
Lane: cloud, read-only on code: claude/co-r4 head 42c9304b, base 68790912 (co-majors-r3, MERGE-OK, in train 87); 7 files +170 -34. One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No full merge gate (the lane ran 17 cells green).
Step 0 (no setup script): `git fetch --unshallow || true`; `git fetch origin claude/session-log claude/co-r4`; `cargo check --workspace`; a failure: send_message the PM "review co-r4 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/co-r4.md (done criteria), reports/co-majors.md "## r4", findings/construct.md rows co-28, co-30; then `git diff 68790912..42c9304b`.
Checks, each with its command and files:
1) Red first: e758dcfd fails on its parent and passes at 5a782e58 (`cargo test -p chora --lib` the named test, per sha in a scratch worktree).
2) co-30, input.rs and ipc.rs: the grabber that owns the node under the pointer wins; the smallest claim breaks a tie only over no grabber node. Walk each look: xp (taskbar, start menu), tiling (bar, launcher), mc (page, remote), construct (hud, panes), the chat and notify overlays. Can a press now go to a look that does not own the node (stacked overlays, a node removed between press and release, a release delivered to a different grabber than the press)? Is the press/release pair kept on one grabber? Do the old pins keep their cases (name them)?
3) rv-1, rv-2 (c98adb18, 51175f9a): pins of code that is already right; the lane showed red with a mutant. Is each pin able to fail (not a tautology)?
4) co-28 (b11cda2a, 42c9304b): the lane closed it as stale checks, not faults (label now a glass pill; walk step 800 per tap). Does the changed 3d gate still catch the original symptom (a wrong pane after the walk), or did it widen its window until anything passes?
5) merge-gate.sh change: construct-extra gains windows and 3d; time cost against the matrix cap (MX-CAP, 315 s).
6) `cargo test -p chora --lib`, `cargo test -p chora-construct`, `cargo clippy --workspace --all-targets -- -D warnings`; no file over 1,500 lines.
Known, not findings: gate-tiling-apps-click / launcher-click red in lane containers (ti-1); FJ-R2 construct bare Escape (open by design).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv-N, severity, file:line, text, fix in one sentence). Append to reports/co-majors.md on claude/session-log under "## Review r4".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
