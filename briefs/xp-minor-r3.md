# xp-minor r3 (rc 1 known issues, F7): review r2 FIX (rxm2-1 MAJOR) on claude/xp-minor-r2; fix it red-first, then four small items

Model: sonnet, effort medium (small test and plugin fixes on a reviewed branch; routine)
Lane: cloud, branch claude/xp-minor-r3 at 8109e08b (xp-minor r2). First merge origin/claude/autonomous-dev-session-3w-i33pg7 (6b89b56f, gate-ratchets; no rebase). Areas: plugins/chora-assistant, plugins/chora-xp/src/wm/panels.rs, tests/gate-xp-helper.sh. Stop at 200 k. Cost at most 1.0 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "xp-minor-r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read reports/xp-minor.md "## Review r2" and "## PM decisions r2" on claude/session-log.
Each item: red test commit first (tests only, message names the id), then the fix commit. In this order (rules.md 6: the MAJOR first):
1) rxm2-1 MAJOR: no test of chora-assistant writes the real $XDG_STATE_HOME/chora/assistant-greeted. Red: a test that runs the greeting units against a temp state home and fails when the real path is touched, or a unit that fails when `greet_with` in test mode reaches `remember_greeting` (your choice; no env var race between parallel tests). Fix: the state home is a field the tests set to a temp dir.
2) rxm2-6: xp30-click waits on the condition: `ctl wait --gone --timeout 4` on the visible balloon (no fixed wait, no tight loop).
3) rxm2-5: `pause` in tests/gate-xp-helper.sh is a fixed wait; replace each use with a wait on the condition (window rect for `pause 2`; for the negative 5 s case a `ctl wait --timeout 5` that fails when the balloon shows). No new sleep (tools/sleep-ratchet).
4) rxm2-2: `greeting_gone` hushes only the greeting text, not a later voice or state sentence.
5) rxm2-4: a click on the tray clock (open and raise paths) takes the focus from the window behind and syncs tasks, as `focus_clock` does.
Out of scope: rxm2-3, rxm2-7, rxm2-8 (backlog), Esc and 11 px (PM decisions r2), other looks, ratchet counts.
Run: the full gate, no step filter: `setsid nohup bash -c '(CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll in the same turn; paste the last 20 lines. A red cell you did not cause: name it with its log line. Before ending: `pgrep -fa 'chora --headless'` empty. Never end a turn to wait.
Done when: five red-then-fix sha pairs with pasted red lines (a gate-cell item may show its red as the cell failing on the old wait); full gate EXIT 0 (or foreign reds named).
Report: reports/xp-minor.md "## r3" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/xp-minor-r3 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
