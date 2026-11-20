# review guide-checks r1 (F7, RC 11): seeded review of claude/guide-checks before its merge

Model: sonnet, effort high (test-harness checks against a written table; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/guide-checks head a78b1a07, base 8b996cee (claude/tour-one r1). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green).
Read on claude/session-log: briefs/guide-checks-r1.md (done criteria), reports/guide-checks.md "## r1", design/tour-v9.md sections 7 and 8 row 3. `git diff 8b996cee..a78b1a07`.
Checks, each with its command and files:
1) Red first: a2ee5219 (planted breaches) before bd449adf (checks); each planted breach in tests/acts/planted.py goes red at a2ee5219 and green at bd449adf: `python3 -m pytest tests/test_acts.py -q` at both shas (worktrees, no build).
2) I6 contrast: 4.5:1 text, 7:1 key line and buttons, measured on the text box of scene.boxes (tests/png-contrast.py, invariants.py); min 14 px for text a person must read; the box, not the whole capture, is measured; a missing box or capture fails, never passes.
3) I2 gaps in whole units (tests/hud-audit.py, tests/tiling-audit.py gap_findings) and held screen-fixed nodes in a camera move; no second checker where one existed (copies of an existing check are a finding).
4) I10 words: tests/keysheet/words.table, no "agent" or "HUD"; the match is by word, not substring (e.g. "Agents" caught, "management" not).
5) tests/acts/known-red.toml (a78b1a07): each row names one real fault (g1-g6), act, rule; a known-red row cannot hide a new fault of another act or rule; the gate still prints it.
Known, not findings: g1-g6 look faults (the lane must not fix looks); the rt-1..rt-7 findings of claude/tour-one (fixed in tour-one r2).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rg-N, severity, file:line, text, fix in one sentence). Append to reports/guide-checks.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
