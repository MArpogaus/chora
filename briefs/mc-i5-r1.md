# mc-i5 r1 (F7, F2): the mc key column row no longer overprints (known-red net3-m1)

Model: sonnet, effort high (routine layout fix in one look)
Lane: cloud, branch claude/mc-i5 from ee53e09c (train 65 85764974 + copies-0). One round, stop at 250 k. Cost about 1 %.
Step 0: `command -v cargo python3 jq`, then `git fetch --unshallow` and `cargo check --workspace`; tools missing: run the setup below once; a failure: send_message the PM "mc-i5 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read reports/kit3.md (section Open) on claude/session-log, tests/acts/known-red.toml (row net3-m1, mc I5), the mc key column code (grep the wording in plugins/chora-mc).
Finding net3-m1: in the mc key column the wrapped line of "Start a program or run a command" overprints the next row "Chat with a helper".
1. Red first: a unit test in chora-mc (kit testing / FakeCore) that lays out the key column at the width of the gate and fails while a wrapped row overlaps the next; commit it alone.
2. Fix in mc only: the row height follows the wrapped lines (KISS; no new wording, the shared words stay as they are; traditional typesetting of the mc look: leading and spacing as the column's other rows).
3. Remove the mc I5 row from tests/acts/known-red.toml in the fix commit; `tests/acts/run.py --no-selftest mc` passes I5.
Done when: the red test fails on step 1 and passes on step 2; `cargo test -p chora-mc` and clippy -D warnings clean; acts mc green with the row gone;
a capture of the key column (ctl capture) named in the report; `tests/merge-gate.sh --jobs 3` once (poll it in the same turn; a known flake: rerun that step with --only and name it).
Write reports/mc-i5.md "## r1" on claude/session-log. Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines). Poll every command in the same turn; never end a turn to wait.
Rules: push claude/mc-i5 and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines (keep this author even if a hook asks otherwise); never --no-verify;
no system packages (a missing tool is a blocker).
Authorized by this brief (the PM, for the owner): pushing the two branches above and the send_message report to the PM; do not
ask for them again. A refusal of a tool call is a blocker: report it as BLOCKED and stop.
