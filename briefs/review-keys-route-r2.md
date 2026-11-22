# review keys-route r2 (rc 2, RC bar: no stray keys): seeded review of claude/keys-route r2 before its merge

Model: opus, effort high, because the diff changes the core input path (choose_manager_of, grab_action, focus_owner) where a wrong rule sends stray keys; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/keys-route head e16d558e, base 541b92bb (r1 head; 7 files, +191 -46); whole branch base 892d035c. One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green on merge 4bdb2c23, 49/49).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review-keys-route BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/keys-route-r2.md (done criteria), reports/keys-route.md "## Review r1" and "## r2" on claude/session-log; `git diff 541b92bb..e16d558e`.
Checks, each with its command and files:
1) rw-1 (crates/chora/src/state.rs choose_manager_of, chosen_manager; ac327bc0): the M4 choice (pointer or focus on a manager's claim) holds for a face or pane window with the keyboard; a key-made choice ends on a grab of the focus owner, a click-made one stays; no held chord survives a manager change. `cargo test -p chora ipc::` and `input::`.
2) grab_action and focus_owner (crates/chora/src/input/place.rs): the focus owner's grabs never wait; prove no stray key: a key goes to exactly one client or to none by a named binding; the escape hatch (press_action: VT switch, Quit, Reset, RevokeAll) stays first and untouched. key_place and grab_action share focus_owner with the same meaning.
3) Red first: 18436b85 before ac327bc0; the unit fails at 18436b85 (run `cargo test -p chora a_grabbed_key_of_the_screen_chooses_it_over_the_window_of_a_face` there only if cheap).
4) rw-2 gate step tests/gate-keys-route.sh (in step `virtual`): it reaches the key-made choice; the brief's deviation (Super+Ctrl+r, Alt+F4, then Super+Return instead of Super+e) is justified by the code (tiling grabs Super+e); waits by until_ok with bounds, no bare sleep; the report's red line at 18436b85 is plausible from the script.
5) docs/api.md input.key matches the new rule (same commit as the code); rw-3 and rw-4 fixed.
Known, not findings: co-43 I3 belongs to lane co-i3 r3.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N from rw-6 on, severity, file:line, text, fix in one sentence). Append to reports/keys-route.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify; no system packages.
