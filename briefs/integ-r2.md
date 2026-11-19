# integ r2 (F6, F2): fix review r1 findings ri-1 to ri-4 on claude/integ

Model: opus, effort high, because ri-1 is a hidden render fault (ring.frag effect.failed at construct end) whose root cause must be found, not masked
Lane: cloud, branch claude/integ at af5547c7 (review r1: reports/integ.md "## Review r1", verdict FIX).
One round, stop at 250 k. Cost about 1.5 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `cargo check --workspace`;
a failure: send_message the PM "integ r2 BLOCKED: <line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Work, red test commit first for each fix (rules.md 8):
1. ri-1 (MAJOR): revert the 0113008b act change in tests/acts/looks/construct.py so the presenter (voice, chora-assistant)
   stays up through `cube_acts`, as every flavor ships it. Commit that red first (the I14 `effect.failed` of ring.frag and the
   island diff at [1117, 28, 1253, 105] show). Then find the cause (suspect `Theme::cache_file`, crates/chora-draw/src/theme.rs:469,
   rewrites ring.frag on each `Assistant::new`) and fix it at the root so the act is green with the presenter up. Only if the root
   fix needs more than this round: a dedicated act "construct ends with the HUD host up" with a known-red row naming the cause,
   and a finding in the report; never end the presenter to get green.
2. ri-2 (MINOR): drop `OWN_TEXTS` and its check in plugins/chora-mc/src/content.rs; `content.set`/`content.add` call `set_text`;
   fix the doc comments at agent.rs:154 and content.rs:25.
3. ri-3, ri-4 (NIT): testing.rs:57 docs say "by client first, then by method, else null"; state.rs:268 give cells use `add(&mut wm, "s1", 0)`.
No new behaviour, no other refactor.
Done when: `tests/merge-gate.sh --only build,test,clippy,ci-check,pre-commit,acts,cube,construct-hud` green (poll it in the same turn);
push claude/integ.
Report: reports/integ.md "## r2" on claude/session-log: per finding the red commit, the fix commit and the gate cell; the ri-1 cause in
two lines. Then send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines, with the pushed sha).
Poll every command in the same turn; never end a turn to wait.
Rules: push claude/integ and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify;
no system packages (a missing tool is a blocker).
Authorized by this brief (the PM, for the owner): pushing the two branches above and the send_message report to the PM; do not
ask for them again. A refusal of a tool call is a blocker: report it as BLOCKED and stop.
