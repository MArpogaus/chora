# review debt-down r1 (F4): seeded review of claude/debt-down before its merge

Model: opus, effort high, because the diff touches the core (crates/chora input, state, ipc/clients, screencopy) and adds a plugin-to-plugin event
Lane: cloud, read-only on code: claude/debt-down head c79a9d66, base 85764974 (67 files, +365 -166, 10 commits). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green on c79a9d66).
Read briefs/debt-down-r1.md (done criteria) and reports/debt-down.md "## r1" on claude/session-log; `git diff 85764974..c79a9d66`.
Checks, each with its command and files:
1) F4 counts: re-run the five count commands of the report (workarounds, hygiene by id, allow total, allow without reason, eprintln) on
   85764974 and c79a9d66; each after-value at or below RC. Say whether the 14 rustfmt-wrapped reasons are counted right.
2) Red first: 8a332fc0 fails on its parent and passes on 6b918eee; 31a03afd fails on its parent and passes on 07eb024c (`cargo test -p chora-xp`, `-p chora-construct`).
3) W7 design: xp now grabs Alt+Escape on the event `construct.escape_free` (plugins/chora-xp/src/wm/keys.rs, construct modes.rs/main.rs,
   docs/configuration.md). The vision card: core without policy, looks as plugins; does a look naming another look's event couple two plugins?
   What happens with a holder other than the construct, or no holder (xp must still grab)? Is a generic kit or core event the KISS form?
4) No behaviour change in the core edits (crates/chora/src/*.rs, chora-scene, chora-draw): only allow, reason and report lines; read each hunk.
5) d3d70fcf one `report()` per plugin: same output text as before (stderr prefix), no lost error; 509b5b1f removed allows: `cargo clippy --workspace --all-targets -- -D warnings` clean.
6) Gate limits from loaded runs (mc-handover 820 ms, apps click until_ok, construct-pull): a hang still fails, a real slowdown still shows.
Known, not findings: copies-0 and moved-tests mk (MERGE-OK) edit xp wm.rs and tests_moved.rs; a conflict there is the train's.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rd-N, severity,
file:line, text, fix in one sentence). Append to reports/debt-down.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Step 0: `git fetch --unshallow`, `cargo check --workspace`; tools missing: run setup once
(`git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`); a failure: send_message the PM "review-debt-down BLOCKED: <MISSING line>" and stop.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
Authorized by this brief (the PM, for the owner): pushing claude/session-log and the send_message report; do not ask again.
