# wm7b r3 (F3): close review r2 rw-1 (cut-check) and rw-5 (stale doc)

Model: sonnet, effort high (routine: move paths and names behind existing kit helpers, one doc line)
Lane: cloud, branch claude/wm7b from 31f04c07. Cost at most 1.5 %. Stop at 250 k context: push, report, hand back.
Step 0, first 3 minutes: `cargo check -p chora` and `command -v foot fuzzel waybar`; if either fails, send_message
the PM "wm7b r3 BLOCKED: no setup" and stop.
Start: `git fetch origin claude/autonomous-dev-session-3w-i33pg7 && git merge` it; merge again before the gate if it
moved. Resolve conflicts on the session side's types (kit::wm::state).
Seed: reports/wm7b.md "## Review r2" on claude/session-log (rw-1, rw-5). Red first is the failing
`python3 tools/cut-check` on 31f04c07 (record its output in the report).
1) rw-1 MAJOR: cut-check clean again. plugins/chora-construct/src/cube/start.rs:34 log dir via
   chora_scene::paths::runtime_home() (KIT1: no XDG_RUNTIME_DIR outside kit); neutral program names in tests and doc
   comments of cube/role.rs:128,134 and names.rs:91,158,161 (construct names chora-xp back to 5, chora-mc to 17);
   Command::Focus outside kit::wm back to 18 (modes.rs:749 through a kit::wm helper, or a reasoned table row).
2) rw-5 MINOR (wrong doc): docs/api.md:1619 says "no desk switch" on virtual outputs without a host; drop the dead
   flat_desks text in help.rs:139 and panels.rs:113.
Not this round (backlog): rw-2, rw-3, rw-4, rw-6.
Done when: `python3 tools/cut-check` clean; `tests/merge-gate.sh --only build,ci-check,acts,cube,construct-hud,xp,xp-faces,keys`
green (poll in the same turn); `cargo test -p chora-construct -p chora-xp -p chora-kit --features chora-kit/testing`;
clippy --workspace clean. No visible change.
Report: append "## r3" to reports/wm7b.md on claude/session-log (items, shas, gate lines, open).
Then send_message to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines). Poll every command in the
same turn; never end a turn to wait.
Rules: push claude/wm7b and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
no system packages (a missing tool is a blocker).
