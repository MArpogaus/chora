# wm7b r4 (F3): merge the session head (train 62: wm7a state::Open, kit2d bindings) into a MERGE-OK branch

Model: sonnet, effort high (conflict resolution onto session types; no new behaviour)
Lane: cloud, branch claude/wm7b at 3e193a91 (MERGE-OK, PM review). One round, stop at 200 k. Cost about 1 %.
Step 0, first 3 minutes: `cargo check -p chora` and `command -v foot fuzzel waybar`; if either fails, send_message
the PM at once "wm7b r4 BLOCKED: no setup" and stop.
1. `git fetch --unshallow`, then `git merge origin/claude/autonomous-dev-session-3w-i33pg7` (dd2a0aa3 or newer).
   Resolve every conflict to the session side's types and names: kit::wm::state (state::Open of wm7a), kit2d's
   common.bindings and KeyAction::System(...). Conflicts seen in train 63: xp help.rs, wm.rs, wm/startup.rs (grab_keys takes the bindings; give_keys a closure).
   Keep this branch's behaviour; drop nothing of either side. One merge commit, no rebase, no force push.
2. Merge the session branch again right before the gate if it moved (train 63 adds flakes-f).
Done when: `tests/merge-gate.sh --only build,ci-check,acts,cube,construct-hud,xp,xp-faces,keys` green (poll in the same turn; never end a turn to wait);
`cargo clippy --workspace --all-targets` clean; `python3 tools/cut-check` exit 0. No full gate (the train runs it).
Report: append "## r4 (merge)" to reports/wm7b.md on claude/session-log: merge sha, each conflict file with the side kept
and why, gate lines. Then send_message to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 5 lines).
Rules: push claude/wm7b and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
no system packages (a missing tool is a blocker).
