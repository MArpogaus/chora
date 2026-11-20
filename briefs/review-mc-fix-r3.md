# review mc-fix r3 (RC 2 persona round 4, stability: mc-42, mc-46, I3, I5): seeded review of claude/mc-fix before its merge

Model: opus, effort high, because 18247e0d adds Plugin::refresh to the kit runtime (state.get of every look) and b0997b71 asks the core during an RPC answer (blocking or reentrancy risk); dab46f6b loosens checker I5 (rules.md 7: the builder never reviews itself)
Lane: cloud, read-only on code: claude/mc-fix head b0997b71, base cb2b80f1 (11 files, +185 -18). One round.
Stop at 120 k context; cost at most 0.6 %; at most 8 findings. No full merge gate (the lane ran 13 gate steps green at --jobs 3).
Step 0 (no setup script): `git fetch --unshallow || true`; `git fetch origin claude/session-log claude/mc-fix`; `cargo check --workspace`; a failure: send_message the PM "review mc-fix BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/mc-fix-r3.md (done criteria), reports/mc-fix.md "## r3", findings/mc.md rows mc-42, mc-46, mc-50 to mc-52; then `git diff cb2b80f1..b0997b71`.
Checks, each with its command and files:
1) Red first: 34270632 fails on its parent and passes at 03cff6d2; 9eff365f before 45fcc3f1; 51642134 before dab46f6b; 08ae4d56 before b0997b71 (the named cell or test per sha in a scratch worktree; gate cells: `tests/merge-gate.sh --only mc-paths` / `mc-sofa` if the cell runs headless, else say so).
2) Kit, runtime.rs `refresh` and wm/focus.rs `drop_node`: refresh runs inside answer_call. Can it block the plugin's event loop (a synchronous call to the core while the core waits on this plugin's answer), recurse into state.get, or fail silently? Does the default no-op leave xp and tiling exactly as before?
3) I3, plugins/chora-mc/src/main.rs b0997b71: when the core reports another node has the keyboard, mc drops its node. Can mc drop its node when it still holds the keyboard (race with a focus it just sent), and does the next key go to the right place?
4) mc-42, main.rs and windows.rs 03cff6d2: a page started by a plain Right keeps the keys. Does Escape and Right still work on a page reached by page turn AND from the launcher (mc-fix r3 item 1)? Any path where a newly adopted window now never gets the keyboard when it should?
5) mc-46, show.rs and region/items.rs: the key list follows the holder. One XF86Back press: is the first press handled, not only shown (the persona finding)?
6) I5 checker, tests/acts/invariants.py dab46f6b: "a look under another look's wallpaper is not shown". Can a real I5 breach (mc visibly shown and broken) now pass? Each new case in test_acts.py fails without the change.
7) `cargo test -p chora-kit -p chora-mc`, `python3 -m pytest tests/test_acts.py -q`, `cargo clippy --workspace --all-targets -- -D warnings`; no file over 1,500 lines (mc main.rs).
Known, not findings: launcher-click red in lane containers (fuzzel 1.9.2; the PM runs fuzzel 1.11.1 in the train); mc-50 yellow square, T1 end-time gap (open, not in scope).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv-N, severity, file:line, text, fix in one sentence). Append to reports/mc-fix.md on claude/session-log under "## Review r3".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
