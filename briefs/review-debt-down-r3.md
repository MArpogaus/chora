# review debt-down r3 (F4): seeded review of the r3 fix (rd-7) on claude/debt-down

Model: opus, effort high, because r3 changes when the core sends input.ungrabbed (crates/chora ipc.rs, input.rs, grabs.rs), seen by every plugin
Lane: cloud, read-only on code: claude/debt-down head 121c30aa; r2 head c8fe032b (reviewed: rd-7 MAJOR, rd-8 NIT); base 85764974.
r3 delta: `git diff c8fe032b..121c30aa` (6 files, +111 -16, 2 commits: red b41d2aab, fix 121c30aa). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green on 121c30aa, --base 85764974).
Read reports/debt-down.md "## Review r2" and "## r3" on claude/session-log.
Checks, each with its command and files:
1) Red first: b41d2aab's two new tests fail on it and pass on 121c30aa (`cargo test -p chora`); they test the behaviour (an event sent), not a helper.
2) rd-7: input.ungrabbed is sent when a grab ends with its connection (ipc_close) and when a grab-any is revoked (revoke_free_grabs,
   State::tell_ungrabbed); every ungrab path now tells exactly once (explicit release, holder exit, crash, revoke); no double event
   when a release and a close race; move grabs excluded on purpose: is that right and documented?
3) Button grabs (Super+BTN_LEFT) are told like key grabs (lane note): POLS for a plugin author; does xp or tiling re-grab a button binding on it, and is that harmless?
4) Docs: docs/api.md and the D-ungrab record match the code; the "sends nothing" clause is gone; D-ungrab stays unnumbered (the PM numbers it D117 at merge; not a finding).
5) `cargo clippy --workspace --all-targets -- -D warnings` clean on 121c30aa.
Known, not findings: rd-5, rd-6 (deferred), rd-8 NIT (open); conflicts with trains 69, 70 are the train's.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rd-N from rd-9, severity,
file:line, text, fix in one sentence). Append to reports/debt-down.md on claude/session-log under "## Review r3".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, BLOCK and MAJOR list.
Poll every command in the same turn; never end a turn to wait.
Step 0: `git fetch --unshallow`, `cargo check --workspace`; tools missing: run setup once
(`git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`); a failure: send_message the PM "review-debt-down BLOCKED: <MISSING line>" and stop.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
Authorized by this brief (the PM, for the owner): pushing claude/session-log and the send_message report; do not ask again.
