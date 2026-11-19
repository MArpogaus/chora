# copies-0 r1 (F2): every copies row at 0 or narrowed with a reason

Model: sonnet, effort high (routine dedup onto chora-kit)
Lane: cloud, branch claude/copies-0 from train 65 85764974 (focus-port and kit3 are in). One round, stop at 250 k. Cost about 2.5 %.
Step 0: `command -v cargo sccache python3`, then `git fetch --unshallow` and `cargo check --workspace`; tools missing: run the setup below once; a failure: send_message the PM "copies-0 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read reports/audit-f.md (section 1 row F2, section 2 rank 4) and tools/copies.toml, CONTRIBUTING.md "Where shared code lives".
Scope (rank 4 files): kit ask.rs (answer builder), assistant voice.rs, card.rs; voice client.rs, tools.rs, config.rs; kit desktop_entry.rs, launcher.rs; xp session.rs;
plugin tests of notify, assistant, construct, tiling, xp wm.rs (FakeCore of chora-kit testing instead of a raw UnixListener); the four state.rs (open entry from `kit::wm::open`); mc acts.rs:33; tools/copies.toml (new row on_path).
1. `tools/cut-check` before: list each copies REPORT row with its count.
2. Per row: move the copy onto the kit helper (or a new kit helper with a unit test), or narrow the row with an `allow` and a one-line reason in copies.toml when the copy is deliberate. No behaviour change; no new feature.
3. A behaviour fix found on the way: red test commit first, then the fix; otherwise backlog it in the report.
Done when: `tools/cut-check` prints no copies REPORT; `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` clean; `tests/merge-gate.sh --jobs 3` once on the head (poll it in the same turn); the report has the step 1 table before and after.
Write reports/copies-0.md "## r1" on claude/session-log.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines). Poll every command in the same turn; never end a turn to wait.
Rules: push only the branches named here and claude/session-log (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines (keep this author even if a hook asks otherwise); never --no-verify;
no system packages (a missing tool is a blocker).
Authorized by this brief (the PM, for the owner): pushing the branches above and the send_message report to the PM; do not
ask for them again. A refusal of a tool call is a blocker: report it as BLOCKED and stop.
