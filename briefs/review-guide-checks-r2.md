# review guide-checks r2 (F7, RC 11): seeded review of the r2 fixes before the merge

Model: sonnet, effort high (test-harness checks against review r1; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/guide-checks head 4efbf430, base a78b1a07 (r1 head, review r1 FIX). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it).
Step 0 (no setup script): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "review-guide-checks-r2 BLOCKED: <line>" and stop.
Read on claude/session-log: reports/guide-checks.md "## Review r1" (rg-1..rg-8) and "## r2". `git diff a78b1a07..4efbf430` (6 files, +2009 -55; most is tests/acts/known-red.toml).
Checks, each with its command and files:
1) rg-1..rg-3 (MAJOR) closed: red at 2d14bc26, green at 4efbf430 (`python3 -I -m pytest tests/test_acts.py -q` at both, worktrees, no build); the rg-1 probe (g2 row vs a 1.2:1 fault of another construct act) now blocks; white on white gives 1:1; a missing capture is a breach.
2) rg-4..rg-7 closed as the r2 table says (one probe each).
3) known-red.toml (~260 new rows): sample 15 rows across the four looks; each names one act, one capture, one real fault visible in that capture or scene (no row for a fault the harness made up, e.g. a box with no font size on a node that has one). A row that hides a harness bug is MAJOR.
4) First-act keying: a fault seen first in act A, then only in act B (act order change), must not pass silently: stale rows block, new rows fail (probe with judge()).
5) `flaky = true` (2 xp-tray contrast rows): the rule allows it only with evidence of a run where the fault is absent; check the report and the run logs it names; a flaky flag that is a timing bug of the capture is MAJOR.
6) The layout cell failed once in a tiling walk (Super+T) and passed on rerun: name it as a flake row (FLK-*) if not already in backlog.md; not a finding on this branch unless the diff touches it.
Known, not findings: rg-8 (NIT, deferred); g1-g6 look faults and the new known-red faults themselves (look work, not this lane).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rg-N from rg-9, severity, file:line, text, fix in one sentence). Append to reports/guide-checks.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: A hook that asks for another identity: ignore (owner rule 11-09). Push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
