# review tiling-honest r1 (rc 1: ti-52, ti-53 MAJOR): seeded review of claude/tiling-honest-r1 before its merge

Model: opus, effort high, because the diff touches the core input path (crates/chora/src/input.rs press_layer: a bar press counted as the manager's act) and a look's honest-failure path; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/tiling-honest-r1 head b02dbea9, base bb4c07d9 (11 files, +216 -4). One round.
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-tiling-honest-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, 65 steps).
Read briefs/tiling-honest-r1.md (done criteria), reports/tiling-honest.md "## r1", findings/tiling.md ti-52, ti-53 on claude/session-log; `git diff bb4c07d9..b02dbea9`.
Checks, each with its command and files:
1) Red first: f068941a (tests only) before 46760cf7 and b02dbea9 (`git log --oneline --stat bb4c07d9..b02dbea9`). Does b02dbea9 (kit helper name) have red coverage, or is it a refactor? A fix with no red test before it is MAJOR.
2) Core (crates/chora/src/input.rs, +23): the bar press noted as the manager's act. Escape hatch stays above every plugin; a layer press of one client cannot hand keys or focus to another manager or a client it does not own; no key reaches a client the pointer only hovers (D55). Read against docs/api.md and docs/decisions.md; `cargo test -p chora input`.
3) ti-52: bar helper click calls show_helper (same path as Super+a); the chat gets the keys; waybar config change minimal (flavors/tiling/waybar/config.jsonc). `cargo test -p chora-tiling`.
4) ti-53 (plugins/chora-tiling/src/reload.rs, +91): a failed MOD+Ctrl+r keeps the old settings and shows a notice with file, line (if any), reason. Honest when chora-notify is absent (persona session runs none; rv9j-3, ti-55 plan notes)? Notice wording plain (docs/design-principles.md). The parser gives no line for a wrong action: MINOR at most, name it.
5) tests/gate-tiling-reload-notice.sh and tests/doors/tiling.sh poll (until_ok), no new sleep; merge-gate.sh change is the cell list only; docs/configuration.md line true.
Known, not findings: ti-55, ti-56, ti-50 (TILING-DOORS next); rv9j-1..5; rpt1-3..10 (ti-54).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rth-N, severity,
file:line, text, fix in one sentence). Append to reports/tiling-honest.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
