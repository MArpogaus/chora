# review debt-down r2 (F4): seeded review of the r2 fixes on claude/debt-down

Model: opus, effort high, because r2 adds a core event (input.ungrabbed) seen by every plugin, incl. confined ones
Lane: cloud, read-only on code: claude/debt-down head c8fe032b; r1 head c79a9d66 (reviewed, findings rd-1..rd-6); base 85764974.
r2 delta: `git diff c79a9d66..c8fe032b` (12 files, +162 -55, 4 commits). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green on c8fe032b, --base 85764974).
Read reports/debt-down.md "## Review r1" and "## r2" on claude/session-log.
Checks, each with its command and files:
1) rd-1: crates/chora/src/ipc.rs, ipc/grabs.rs: `input.ungrabbed {binding}` is sent on every ungrab (explicit release, holder exit,
   holder crash); xp grabs Alt+Escape for any holder and with no holder (plugins/chora-xp/src/wm/keys.rs, dispatch.rs);
   `grep -rn escape_free` empty. Can a confined subscriber learn anything it should not (binding string only)? Is there a race
   where two plugins re-grab on the same event, and does the loser get a clean refusal (no log spam, rd-2)?
2) Red first: 4f90c968 fails on its parent, passes on c8fe032b (`cargo test -p chora -p chora-xp`); 07fe4d03 fails on c79a9d66,
   passes on 9346e83d (`cargo test -p chora-construct`). The lane says the rd-1 red test failed at subscribe: is it a real red for the behaviour?
3) rd-3: plugins/chora-construct/src/cube/start.rs: settle counts only passes that began after the ask; no new fixed sleep; a hang still fails.
4) Docs: docs/api.md, configuration.md, decisions.md entry `D-ungrab` (number it D116+ in your text only; the PM numbers at merge)
   match the code; POLS for a plugin author.
5) rd-4 report figure (192, 75 wrapped) re-run with the r1 count command.
6) `cargo clippy --workspace --all-targets -- -D warnings` clean on c8fe032b.
Known, not findings: rd-5, rd-6 (deferred); conflicts with copies-0 and moved-tests mk in xp wm.rs and tests_moved.rs are the train's.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rd-N from rd-7, severity,
file:line, text, fix in one sentence). Append to reports/debt-down.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, BLOCK and MAJOR list.
Poll every command in the same turn; never end a turn to wait.
Step 0: `git fetch --unshallow`, `cargo check --workspace`; tools missing: run setup once
(`git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`); a failure: send_message the PM "review-debt-down BLOCKED: <MISSING line>" and stop.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
Authorized by this brief (the PM, for the owner): pushing claude/session-log and the send_message report; do not ask again.
