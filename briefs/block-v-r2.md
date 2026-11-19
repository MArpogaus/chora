# block-v r2 (RC 1, mc-38): a remote's Home goes to the first page in mc

Model: sonnet, effort medium (small product fix with a red act; plan given)
Lane: cloud, branch claude/block-v, head cbdbc34f (r1). One round, stop at 250 k. Cost about 0.5 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "block-v r2 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Read on claude/session-log: reports/block-v.md "## PM review r1" (rw-1, rw-2), findings/mc.md mc-32, mc-38, mc-1.
1. Red act first (own commit): in tests/acts/looks/mc.py remote_acts, after the Right presses, `remote("Home")`
   goes to agenda, and `remote("XF86HomePage")` does too. Both red on cbdbc34f.
2. Fix in plugins/chora-mc where plain Right, Left and Escape act on a page with no program (mc-1, 35f0c227):
   plain Home and XF86HomePage go to the first page there too; not a binding; with --output beside other
   window managers a page still takes no keyboard. Update the mc.yaml comment ("Right and Left alone ...") and
   any hint or key-list text that names the page keys, so a person reads Home there.
3. rw-2: remote_acts sets the theme back to what it was at its end.
4. mc-38 row in findings/mc.md: `fixed <sha> act:<name>`; mc-32: `closed` with both acts.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,acts,mc-paths` green. Before
the final gate merge origin/claude/autonomous-dev-session-3w-i33pg7. Poll it in the same turn; never end a turn to wait.
Done when: 1-4 hold with the commands that show them; gate green; reports/block-v.md "## r2" on
claude/session-log (each claim with its command or cell); head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/block-v and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
