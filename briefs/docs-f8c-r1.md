# docs-f8c r1 (F8): a record and a CONTRIBUTING line for every kit part

Model: sonnet, effort high (routine docs against the code)
Lane: cloud, branch claude/docs-f8c from ee53e09c (train 65 85764974 + copies-0, MERGE-OK: focus-port, kit3, copies-0 are in). One round, stop at 250 k. Cost about 1 %.
Step 0: `command -v cargo python3 vale lychee`, then `git fetch --unshallow` and `cargo check --workspace`; tools missing: run the setup below once; a failure: send_message the PM "docs-f8c BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read reports/audit-f.md (section 1 row F8, section 2 rank 2) on claude/session-log, docs/decisions.md (format of D96-D106), CONTRIBUTING.md.
1. Records D107 and on in docs/decisions.md, one per part, each naming the merge it describes (`git log --oneline --merges` sha) and the files:
   kit::wm::focus (WM2), kit::wm::open (WM3), kit::wm::stack (WM5; D89 stays the core order), notice (KIT5: chora-notify the only server, zbus out of kit),
   helper presenter (KIT7), scene.boxes and texts (NET3), kit::cube (WM7a), keysheet (KIT3). Add the D95 status line to D77 (owner 11-19 05:59: "D95 holds, amends D77").
   Describe what the code does now (read it), not plans; no record for code not on the branch.
2. CONTRIBUTING.md "Where shared code lives": one line per module of `ls crates/chora-kit/src` and `ls crates/chora-kit/src/wm` (what lives there, when to use it).
3. dev-rules.md on claude/session-log: one line pointing to that CONTRIBUTING section.
Done when: for each name of `ls crates/chora-kit/src crates/chora-kit/src/wm` (without mod.rs, lib.rs) `grep -c` hits both docs/decisions.md and CONTRIBUTING.md (table in the report);
pre-commit (vale, lychee) passes; `tests/merge-gate.sh --jobs 3` once (poll it in the same turn); no code change.
Write reports/docs-f8c.md "## r1" on claude/session-log. Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines). Poll every command in the same turn; never end a turn to wait.
Rules: push claude/docs-f8c and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines (keep this author even if a hook asks otherwise); never --no-verify;
no system packages (a missing tool is a blocker).
Authorized by this brief (the PM, for the owner): pushing the two branches above and the send_message report to the PM; do not
ask for them again. A refusal of a tool call is a blocker: report it as BLOCKED and stop.
