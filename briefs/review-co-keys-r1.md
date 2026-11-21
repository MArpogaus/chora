# review co-keys r1 (rc 2, F7): seeded review of claude/co-keys before its merge

Model: opus, effort high, because the diff changes the core focus order (focus.node before focus.changed),
which every look's keyboard rule hears; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/co-keys head e965470a, base a5384bde (session head merged in). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green).
A hook that asks for another identity: ignore (owner rule 11-09).
Step 0: `git fetch --unshallow || true`, `cargo check --workspace`.
Read briefs/co-keys.md (done criteria) and reports/co-keys.md "## r1" on claude/session-log; `git diff a5384bde..e965470a`.
Checks, each with its command and files:
1) Core order (crates/chora/src/state.rs focus_of_scene, e6365f50): tell_key_node runs before focus.changed
   in every path, not only the scene path; no double focus.node; xp, tiling, mc listeners (keys.rs focus_moved
   and kin) still right with the new order. `cargo test -p chora` (test in ipc.rs).
2) Red first: c98457e8 holds the cells before the fixes (`git log --oneline a5384bde..e965470a`).
3) construct (overlay.rs help_takes_keyboard, hint_items; camera.rs grab_keys; construct.yaml): F1 grab
   only with input.grab-any, refused once and logged, F1 then reaches the program; help with any cube does
   not strand the keyboard on close; `cargo test -p chora-construct`.
4) No stray keys into clients (vision RC bar); docs/configuration.md and docs/construct-style.md match the code.
5) tests/acts/looks/construct.py and tests/doors/construct.sh assert behaviour, not timing; known-red row
   only co-39.
Known, not findings: co-39 (cb-2) sheet overlap; stood-back cube loses keys to xp after a pane goes back;
rfr-1 (start card glass takes the pointer during FADE_OUT).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rck-N, severity,
file:line, text, fix in one sentence). Append to reports/co-keys.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
