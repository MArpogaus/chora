# focus-port r1 (WM2b; F2, F3, F7): the four looks hand focus through kit::wm::focus

Model: opus, effort high, because the port crosses all four looks and the core focus path, and it must prove kit7-a1/a2 regression or old on the v8 build
Lane: cloud, branch claude/focus-port from claude/integ 3619ccb3. One round, stop at 250 k. Cost about 3.5 %.
Step 0: setup (below), then `git fetch --unshallow` and `cargo check -p chora`; a failure: send_message the PM "focus-port r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read on claude/session-log: reports/audit-f.md (section 2 rank 3, refactor proposals), reports/kit7.md (kit7-a1, kit7-a2), design/foundation.md F2.
Known: integ 3619ccb3 is red on two xp tests (`wm::state` a_give_shuts_the_start_menu, a_give_with_the_drag_shuts_the_start_menu);
lane integ fixes them. Skip those two locally; before the gate merge origin/claude/integ (it will hold the fix).
1. kit7-a1 (mc I3) and kit7-a2 (construct Tab act, I4): reproduce each on the v8 build 733967aa with the same choractl
   steps; the report says regression or old, with the command and output.
2. Red first: the cut-check copies rows "Event::FocusChanged outside kit" and "Command::Focus outside kit::wm" set to max 0
   for plugins (allow the crates that define the types: chora-scene, core ipc, kit testing, each with a one-line reason).
3. Port mc (main.rs, state.rs), construct (plugin.rs and the files that match), tiling (main.rs, windows.rs), xp (wm/windows.rs,
   wm/dispatch.rs) and the assistant (main.rs) to `kit::wm::focus`; tests that build FocusChanged by hand use a kit testing helper.
   Behaviour stays; the kit7-a1/a2 rows in tests/acts/known-red.toml go green (remove them in the merge that turns them green).
Done when: both rows at 0, `tools/cut-check` no new REPORT line; kit7-a1/a2 gone from known-red.toml or a written root cause;
`tests/merge-gate.sh --jobs 3` once, green (poll it in the same turn).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines) and write reports/focus-port.md
"## r1" on claude/session-log. Poll every command in the same turn; never end a turn to wait.
Rules: push claude/focus-port and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
no system packages (a missing tool is a blocker).
