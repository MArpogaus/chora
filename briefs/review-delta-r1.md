# review-delta r1 (rc 6): seeded per-area code review of adf71956..6b01c976

Model: opus, effort high, because it is a per-area review of core input, IPC and the plugins' key paths (stray keys, lost windows, security); the builders never review themselves (rules.md 7)
Lane: cloud, read-only on code: claude/review-delta at 6b01c976, base adf71956 (code part: crates, plugins, services: 87 files, +6243 -1394; 61 first-parent merges). One round.
Stop at 200 k context; cost at most 1.5 %; at most 10 findings. No merge gate run (each merge ran it green). Why: plans/week-3.md row 19 and rc 6 (claude/session-log): the 11-20 08:59 review covers code to adf71956 only. This round takes the delta now; a short round on the Thursday head follows.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review-delta BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read: `git diff adf71956 6b01c976 -- crates plugins services`; `git log --oneline --first-parent adf71956..6b01c976`.
Checks, each with its command and files:
1) Core (crates/chora/src/input.rs, ipc.rs, testing.rs): the escape hatch stays above every plugin; no key reaches a client the pointer only hovers; IPC requests validated (sizes, ids, unknown fields) and no panic on bad input; testing.rs compiled only for tests or a feature. `cargo test -p chora input:: ipc::`. Skip safety.rs and logs.rs: reports/core-rv.md r1 reviewed and fixed them (rv-1, rv-2, rv-5).
2) Kit (crates/chora-kit/src/launcher.rs, wm/focus.rs; tests/wm_focus.rs): launcher spawns without a shell from user strings (no injection), honest failure when the launcher is missing; focus holder per look. `cargo test -p chora-kit --features testing`.
3) xp (plugins/chora-xp/src/help.rs, context.rs, dialog.rs, display.rs, wm/state.rs, wm/press.rs, wm/startup.rs): no lost window on handover or close; a dialog asks before a risky act; no blocking call on the event loop. `cargo test -p chora-xp`.
4) construct (plugins/chora-construct/src/lifecycle.rs, plugin.rs, camera.rs, state.rs, overlay.rs, command.rs, input.rs): no window lost when a face leaves; no stray keys while the camera moves. `cargo test -p chora-construct`.
5) mc (plugins/chora-mc/src/main.rs, +397): split main.rs if it holds more than one concern (name the cut as a finding, do not cut); voice and mouse paths equal. `cargo test -p chora-mc`.
6) Logging (rc 9a recheck): new eprintln! or log lines with user data (paths, typed text) at info or above; `tools/lint-ratchet`.
Known, not findings: the tiling launcher-chat flake (FLK-DOORS-CHAT, fixed in ec8de7b7); co-41/co-42 (CO-KEYS, CO-HINT lanes); mc-47 (fuzzel 1.9.2 ignores clicks: known issue).
Severity: BLOCK, MAJOR (HIGH), MEDIUM, MINOR. Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rd-N, severity, file:line, text, fix in one sentence), grouped by area.
Report: reports/review-delta.md "## r1" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 6 lines: verdict, counts per severity, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify; no system packages.
