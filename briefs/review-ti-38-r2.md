# review ti-38 r2 (F7, rc 1 and 2: ti-38 MAJOR): seeded review of claude/ti-38 before its merge

Model: opus, effort high, because the diff changes the core rule that picks the window manager of a new window (crates/chora/src/state.rs, ipc.rs) and the keyboard holder at a join; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/ti-38 head a1f3c8d7, base 3b14c825 (15 files, +227 -16). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran 49/49 green at --jobs 1).
Read briefs/ti-38-r2.md, findings/tiling.md ti-38 and reports/ti-38.md "## r1" and "## r2" on claude/session-log; `git diff 3b14c825..a1f3c8d7`.
Checks, each with its command and files:
1) `State::first_manager` (state.rs ~935) and `Ipc::conns_of` (ipc.rs): "connected first" is true (connection ids grow with connect order, a reconnect counts as last, an owner with two connections uses its oldest); only owners with a claim count; no cost per new window that grows badly. `cargo test -p chora`.
2) Rule order in `manager_of_new`: a click choice (`chosen_manager`, keys-route `choose_manager_of`) and the focus still win over `first`; `first` applies only with no focus and the pointer in no claim, as D119 says. Read docs/api.md "Window managers" against the code.
3) Merge resolution: plugins/chora-xp/src/wm/keys.rs `focus_moved` keeps `!own_node()` (rx-1) and `!joined()`; tests/gate-handover-shared.sh keeps the rx-1 check unweakened; tests/merge-gate.sh keeps gate-join-keys (matrix) and gate-keys-route (virtual). `cargo test -p chora-xp`.
4) Red first: 27b5476d (test) before ac366c40 and 4441f1d4 (fixes); gate-join-keys asserts the holder after the join and after a click, not only that a window opened.
5) D119 matches the code (startup.rs `adopt_all`, `press_wallpaper`) and the persona result in r2 (first_look_takes_the_keyboard changes nothing).
6) The construct I6 step-in-text cell: 2/10 on the merged head vs 0/10 on the session head (r2 table). The diff touches no construct file; confirm, and say whether anything in the diff (first_manager, focus at join) could reach the construct hint chip. Not a finding if not.
Known, not findings: the construct hint-click act flake (lane flake-hint r1); D119 "Accepted by the PM" is filled at merge.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity, file:line, text, fix in one sentence). Append to reports/ti-38.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
