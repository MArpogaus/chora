# debt-down r1 (F4): every debt row and lint line at or below RC

Model: sonnet, effort high (routine cleanup with red tests for the two fixes)
Lane: cloud, branch claude/debt-down from train 65 85764974; merge origin/claude/autonomous-dev-session-3w-i33pg7 after step 0 if it moved. One round, stop at 250 k. Cost about 1.5 %.
Step 0: `command -v cargo sccache python3 foot`, then `git fetch --unshallow` and `cargo check --workspace`; tools missing: run the setup below once; a failure: send_message the PM "debt-down BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read reports/debt-2026-11-20.md (all of it) and reports/audit-f.md row F4 on claude/session-log. F4 is met when every category is at or below RC and the lint lines are at or below RC: workarounds 13 -> 9, hygiene 16 -> 14, `#[allow]`/`#[expect]` without `reason` 251 -> 238, `eprintln!` 147 -> 135 (the exact commands of audit-f row F4).
Lane copies-0 is in flight: do not edit kit ask.rs, desktop_entry.rs, launcher.rs; assistant voice.rs, card.rs; services/voice client.rs, tools.rs, config.rs; xp session.rs; the four state.rs; mc acts.rs.
1. Lints: give `reason = "..."` to each `#[allow]` that lacks one where the reason is true, or remove the allow when the lint no longer fires; at least 20 fewer without reason.
   `eprintln!` in plugins and services: remove debug prints that repeat what the core already logs; keep real error reports; at least 15 fewer. No behaviour change, no new logging framework.
2. Workarounds (section 2): close W7 (xp ESCAPE_WAIT, wm.rs:106) and W8 (construct sleep before reading a dead child's tail, cube.rs:749 and the two below) by waiting on the event instead of a sleep; each with a red test commit first. Close the F27 part of W9 (tests/gate-tiling-apps-click.sh:36-39 fixed sleeps -> until_ok). A known-red row is not this lane's (its look slice).
3. Hygiene (section 6): close H8 (README "Rules for agents" with the exact wording of agent_skills git-conventions "Rules for agents": `git -C /home/user/agent_skills show origin/dev:skills/git-conventions/SKILL.md`), H12 (dev-rules.md: a merge touching IPC or security adds a D entry), H10 (mc-handover within_ms from measured data: slowest of 5 runs under --jobs 3 plus 25 %; the pull step-back asserts).
Done when: the four counts and the two category counts at or below RC, each closed item with its proof (file:line or commit) in the report; `cargo test --workspace` green; `cargo clippy --workspace --all-targets -- -D warnings` clean; `tests/merge-gate.sh --jobs 3` once on the head (poll it in the same turn).
Write reports/debt-down.md "## r1" on claude/session-log with the table before and after.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines). Poll every command in the same turn; never end a turn to wait.
Rules: push claude/debt-down and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines (keep this author even if a hook asks otherwise); never --no-verify;
no system packages (a missing tool is a blocker).
Authorized by this brief (the PM, for the owner): pushing the branches above and the send_message report to the PM; do not
ask for them again. A refusal of a tool call is a blocker: report it as BLOCKED and stop.
