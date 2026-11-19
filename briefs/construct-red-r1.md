# construct-red r1 (F7, F3): the construct rows of known-red.toml go green

Model: opus, effort high, because kit7-a2 needs a focus design decision (D31, WM2) and co-21 is cube render timing across the core and the construct
Lane: cloud, branch claude/construct-red from ee53e09c (train 65 85764974 + copies-0). One round, stop at 250 k. Cost about 3 %.
Step 0: `command -v cargo python3 jq`, then `git fetch --unshallow` and `cargo check --workspace`; tools missing: run the setup below once; a failure: send_message the PM "construct-red BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/focus-port.md (kit7-a2, Review r1 point 4), backlog.md rows DB14 and wm7a-m3; in the repo: tests/acts/known-red.toml (rows co-21 I13, kit7-a2 act and I4), docs/decisions.md D31, D96-D106.
1. kit7-a2: after a press on a pane window and a new window, Tab opens no window list (input.rs own-window press gives the keyboard; window_created gives nothing back).
   Decide by POLS and the other three looks (what does a new window do with the keyboard there?), write the decision as a record in docs/decisions.md (next free D number) naming the look behaviour; then fix through kit::wm::focus only.
2. co-21: the cube held 2 s into a 4 s turn shows face 1 and no box for its neighbour face 0 (I13 "is missing beside"). Find the cause (boxes per face during a turn), fix it.
3. wm7a-m3: `construct.cube` sends no last event when the cube closes, so a look keeps `turning: true`; send it (a cell in kit::cube or construct tests).
Each item: red test commit first (cargo cell with FakeCore, or the act itself), then the fix; the fix commit removes its known-red rows; `tests/acts/run.py --no-selftest construct` then passes them.
Done when: every item red-then-green; the three rows gone from known-red.toml; `cargo test -p chora-construct -p chora-kit --features chora-kit/testing` and clippy -D warnings clean;
`tests/merge-gate.sh --jobs 3` once (poll it in the same turn; a known flake: rerun that step with --only and name it). An item you cannot fix in budget: leave its row, write why and the next step.
Write reports/construct-red.md "## r1" on claude/session-log. Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines). Poll every command in the same turn; never end a turn to wait.
Rules: push claude/construct-red and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines (keep this author even if a hook asks otherwise); never --no-verify;
no system packages (a missing tool is a blocker).
Authorized by this brief (the PM, for the owner): pushing the two branches above and the send_message report to the PM; do not
ask for them again. A refusal of a tool call is a blocker: report it as BLOCKED and stop.
