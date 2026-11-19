# review kit2c r1 (F2): seeded review of claude/kit2c before its merge

Model: sonnet, effort high (a port onto fixed kit types plus two launch fixes; the builder never reviews itself)
Lane: cloud, read-only on code: claude/kit2c head c29a3e9c, base 1d1aa5fd. kit6 (ad88ebaf) is already
MERGE-OK (reports/kit6.md "## Review r1"): review only the merge 854981fe (`git show --remerge-diff 854981fe`)
and the four commits `git log --no-merges c29a3e9c ^1d1aa5fd ^ad88ebaf`. One round. Stop at 120 k context;
cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green).
Read briefs/kit2c-r1.md (done criteria) and reports/kit2c.md, reports/kit6.md on claude/session-log.
Checks, each with its command and files:
1) Red first: f58da09f before 2e12231c, e5eea8a4 before c29a3e9c; the cells fail at their red commit
   (run only if cheap: `cargo test -p chora-tiling`, `-p chora-kit --features testing launch`).
2) Tiling on kit::actions: settings::Common as Host, no own action table or parser left
   (grep plugins/chora-tiling); configs of 29e2cd77 (default and the user file) load to the same table;
   an unfit binding is dropped with a warning, never a silent change. `cargo test -p chora-tiling`.
3) Cross-look test of kit2b gains tiling; door acts and tiling gate steps unchanged; known-red only shrinks.
4) rw-1: the wrapper runs the command in a `( )` subshell; `exit N` and a failed `exec` write the status;
   red cells use `exit 1`, not `false`. rw-2: a start that cannot be followed (pidfd_open fails, EINTR,
   unit active with pid null) is polled each second, not counted as running; no busy loop, no leaked fd
   (crates/chora-kit/src/launch*). rw-3 prefix chora-launch-exit-; D104 open point closed in docs/decisions.md.
5) tools/copies.toml tiling action rows at 0 and true (`tools/cut-check`); size-ratchet not grown.
Known, not findings: rw-2 real pidfd failure tested only through the seam; xp `bindings` is KIT2d.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/kit2c.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
