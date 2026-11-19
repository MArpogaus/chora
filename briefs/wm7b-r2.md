# wm7b r2 (F3): finish the hosted cube (owner 10:52 "Hosted (Recommended)")

Model: opus, effort high, because the open items are the focus owner across two looks (I3) and crash recovery
of the cube host, where a wrong rule loses windows (vision card: no lost window in any handover)
Lane: cloud, branch claude/wm7b from 1ae3f243 (r1 handed over at 250 k: reports/wm7b.md on claude/session-log).
Cost at most 4 %. Stop at 250 k context: push, report, hand back.
Start: `git fetch origin claude/autonomous-dev-session-3w-i33pg7 && git merge` it (train 62 brings wm7a and kit2d);
merge it again before the gate if it moved. Resolve conflicts on the session side's types (kit::wm::state).
Items, each with a red cell first (rules.md 8), then the fix:
1) a1: I3 in the act "the key of the step back stands back": xp drops its claim on focus.changed {surface: null}
   (and when the core focuses another client's node); row wm7a-a1 out of tests/acts/known-red.toml.
2) Flat fallback act "without construct the pager of xp switches the desk": click on button 2 gives desk 2
   (diagnose desk 4: index base or face order); the act passes.
3) m4: MANAGER no longer the constant "xp": the cube host is found by its role; unit test.
4) tests/gate-construct-cube.sh section 6 states the new behaviour (a dying construct leaves a flat xp with every
   window; a restarted construct reopens the cube with the windows on faces); no assertion of the old crash.
5) Re-run the act that failed I1 before 1ae3f243 (construct answers state.get); green.
6) docs/api.md (input.key: a button grab of one client ends another's lone-modifier tap; xp hosted and flat),
   docs/configuration.md (cube_manager outlives the construct; pager on faces), one line in docs/decisions.md for
   hosted. xp's own screen cube stays (owner call; proposal goes to backlog via the report, not built).
Done when: items 1-6 closed with red and green shas; `tests/merge-gate.sh --only build,acts,cube,pull,construct-hud,xp,xp-faces,keys`
green (poll in the same turn); `cargo test -p chora-construct -p chora-xp -p chora-kit --features chora-kit/testing`;
clippy --workspace clean. No full gate (the train runs it). No visible change beyond the named behaviour.
Report: append "## r2" to reports/wm7b.md on claude/session-log (items, red/green shas, gate lines, open).
Then send_message to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines). Poll every command in the
same turn; never end a turn to wait.
Rules: push claude/wm7b and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
no system packages (a missing tool is a blocker).
