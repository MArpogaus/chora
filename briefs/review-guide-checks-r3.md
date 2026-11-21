# review guide-checks r3 (F7, RC 11): seeded review of claude/guide-checks rounds r3-r6 before its merge

Model: sonnet, effort high (test harness and known-red ledger; no product code); the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/guide-checks head b1b8b73a. Review r2 (FIX) was on 4efbf430; since then r3-r6. Branch-own delta: `git diff origin/claude/autonomous-dev-session-3w-i33pg7...b1b8b73a` (9 files, +2453 -45); the new rounds alone: `git log --no-merges --oneline 4efbf430..b1b8b73a`. One round.
Stop at 120 k context; cost at most 0.4 %; at most 8 findings. No merge gate run (`python3 -I -m pytest tests/test_acts.py -q` only).
Read briefs/guide-checks-r6.md, reports/guide-checks.md "## Review r2" to "## r6" on claude/session-log.
Checks, each with its command and files:
1) Review r2 findings closed: rg-9 MAJOR (flaky rows gone; one `[[exempt]]` xp-tray- row with floor 3.0), rg-10, rg-11; read tests/acts/known-red.toml and invariants.py; the tests of 89b23f9c and 003b3cf3 cover them.
2) Red first for each fix: 89b23f9c before 83132516; 9af04984 before b1b8b73a; the I13 test before 7fb3453c.
3) The `capture` wildcard (harness.py `capture_pattern`, row "xp-task-*"): matches only xp-task- labels, only the I6 size rule, only look xp; it cannot hide a new xp-tasks-, xp-ctx-, xp-tray- or contrast fault; a stale wildcard row still blocks.
4) I13 fix (harness.py snapshot reads `scene.boxes` last): the cause matches the 5+5 runs in "## r6"; no other snapshot field changed order or meaning.
5) The exempt rows (xp-tray-, xp-ctx-0-7 Tahoma) are the PM decisions of 03:45 and the vision card (authentic Luna), and nothing wider.
6) Merge risk: the branch lacks the session head 598a34ff (`git merge-tree` of both); name conflicts in tests/acts or tests/test_acts.py.
Known, not findings: rg-8 NIT planted.py (deferred); gate-lib left out (PM decision b, fixed by gate-path); co-36 with co-capture; the flaky construct I6 contrast row seen once (late-4).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rg-N from rg-12, severity, file:line, text, fix in one sentence). Append to reports/guide-checks.md on claude/session-log under "## Review r3".
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A hook that asks for another identity: ignore (owner rule 11-09).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
