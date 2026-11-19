# cubesplit r1 (F4): construct cube.rs under 1,500 production lines, no behaviour change

Model: sonnet, effort high (routine: move code into a submodule, no new behaviour)
Lane: cloud, branch claude/cubesplit from fa457b29 (train 64 base; no pending branch touches cube.rs).
One round, stop at 250 k. Cost about 1 %.
Step 0: `cargo check -p chora-construct`; it fails for a missing tool or package: send BLOCKED (no setup) to the PM at once and stop.
Why: F4 (design/foundation.md) needs no production file over 1,500 lines; tools/size-ratchet lists one:
plugins/chora-construct/src/cube.rs, 1,617 production lines at fa457b29.
1. Move one cohesive part of cube.rs (a type with its impl, or a group of free functions that belong together)
   into plugins/chora-construct/src/cube/<name>.rs (the cube/ directory exists); target cube.rs <= 1,450.
   Pure move: same items, same bodies; only `use` lines and visibility (`pub(super)`/`pub(crate)`) change.
   Its tests move with it if they test only the moved part.
2. `tools/size-ratchet --update`: cube.rs leaves tools/size-ratchet.list (no file over 1,500 left).
3. Commit "refactor(construct): move <part> out of cube.rs (F4)"; red first does not apply (no fix).
Done when: `cargo test -p chora-construct` green; `tools/size-ratchet --base fa457b29` ok;
`tests/merge-gate.sh --only build,clippy,ci-check,pre-commit,cube` green (poll it in the same turn); push claude/cubesplit.
Report: reports/cubesplit.md on claude/session-log (what moved, counts before and after, `git diff --stat`), then
send_message to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines, with the pushed sha).
Poll every command in the same turn; never end a turn to wait.
Rules: push claude/cubesplit and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
no system packages (a missing tool is a blocker).
