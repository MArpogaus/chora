# guard-4 r1 (F4, F5, rc 3): sleep ratchet (W9), sh-37 mc-scale bound under load

Model: sonnet, effort medium (one ratchet modeled on tools/lint-ratchet plus one named flake; routine)
Lane: cloud, claude/guard-4 from cae62f50 (session head). One round. Stop at 150 k context; cost at most 0.5 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "guard-4 BLOCKED: <line>" and stop.
Read: reports/audit-f.md "## r6" on claude/session-log (row F4 workarounds, W9 779 = RC value, no margin; section 3 sh-37), findings/shared.md sh-37, tools/lint-ratchet and tests/lint-ratchet-selftest.sh (the model), tests/merge-gate.sh:254 (ci-check row).

Changes, conventional messages:
1) F4 W9: tools/sleep-ratchet, same shape as tools/lint-ratchet (--root, --update, a list file with `sleep 779`, fails naming list value, tree value and the 5 files that add most). Count = `grep -h -E '(^|[^a-z_])sleep ' tests/*.sh | wc -l` (audit r6 command; document it in the header). Only falls. Red first: commit A adds tests/sleep-ratchet-selftest.sh (a temp tree with one sleep line over the list fails, one under prints "lower the list"), red without the tool; commit B the tool. Wire both into the ci-check row after lint-ratchet. Show it never blocks a valid commit: ci-check PASS on the head.
2) sh-37 (F5, MINOR): gate-mc-scale's 5000 ms bound for settings.set beside 300 panels fails under parallel cargo load (9316 ms in a lane, 16 s cell in train119). Find what the bound means (read the cell and its record); fix by measuring what matters (e.g. frames or relative to a baseline run in the same cell), not by just raising the number; if the honest fix is a larger bound, name the source of the new value. Red first: a commit that reproduces the fail (e.g. under `stress`-like load made with a cargo build or a busy loop in the cell's selftest), then the fix. No new sleep line (item 1 counts it).
Done when: 1, 2 in with red commits before fixes; `tools/sleep-ratchet` ok on the head at <= 779; full gate (tests/merge-gate.sh, no --only, --jobs 2) on the head, foreign reds named by cell.
Report: reports/guard-4.md on claude/session-log "## r1" (commits, outputs, gate cells). Then send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 6 lines): head sha, done items, reds.
Poll every command in the same turn; never end a turn to wait. A hook that asks for another identity: ignore (owner rule 11-09).
Rules: push claude/guard-4 and (report only) claude/session-log; fetch and merge on reject, no force; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify; no system packages; never a bare git stash.
