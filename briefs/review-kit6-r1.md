# review kit6 r1 (F2): seeded review of claude/kit6 before its merge

Model: opus, effort high, because the diff carries a design deviation (D104 unmet: the exit-status file stays)
and a watcher thread on a pidfd, where a leak or race loses a launch state; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/kit6 head ad88ebaf, base dc4bc46d (22 files, +1430 -1016). One round.
Stop at 120 k context; cost at most 0.6 %; at most 8 findings. No merge gate run (the lane ran it green once).
Read briefs/kit6-r1.md (done criteria), reports/kit6.md on claude/session-log (Deviations), docs/decisions.md D87, D104;
`git diff dc4bc46d..ad88ebaf`.
Checks, each with its command and files:
1) Red first: 56990810 holds the cells before 34ca4aa4 and ad88ebaf (`git log --oneline dc4bc46d..ad88ebaf`).
2) crates/chora-kit/src/launch.rs: the watcher thread ends with its program (no thread or fd per launch left
   behind; read its join or drop path); pidfd open fails cleanly (old kernel, unit gone); "Exited at once" =
   gone AND status non-zero AND silent is sound and has no window race. `cargo test -p chora-kit --features testing`.
3) Deviation: is keeping the status-file wrapper in the kit the simplest correct design (KISS), or is there a small
   way to get the exit status (unit ExecMainStatus over D-Bus or `systemctl show`)? Say which, and what D104 should say.
4) Ports: xp launch.rs own tracking deleted, tiling uses launch::run only, mc and construct path changes only;
   visible behaviour unchanged (start menu, launcher; xp-notepad exit-0 hand-off). `cargo test -p chora-xp -p chora-tiling`.
5) tools/copies.toml rows true (`tools/cut-check`); docs/configuration.md and architecture.md match the code.
Known, not findings: the unit path is not exercised (no user manager in the container); D104 status is the PM's.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/kit6.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list, the answer to check 3. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
