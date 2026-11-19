# review kit-sec r1 (F5, security): seeded review of claude/kit-sec before its merge

Model: opus, effort high, because the diff changes the security checks of the runtime, cache and data dirs (socket, shaders, icons), where a wrong rule lets another user plant files or stops chora on a safe system; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/kit-sec head cb2976c1, base a61e8eaa (11 files, +500 -37). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green).
Step 0: `git fetch origin claude/session-log claude/kit-sec claude/autonomous-dev-session-3w-i33pg7`; `cargo check --workspace` only if a check below runs tests; a failure: send_message the PM "review kit-sec BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Read briefs/kit-sec-r1.md (done criteria 1-6) and reports/kit-sec.md "## r1" on claude/session-log; `git diff origin/claude/autonomous-dev-session-3w-i33pg7...origin/claude/kit-sec`.
Checks, each with its command and files:
1) Runtime dir rule (crates/chora-scene/src/paths.rs): owner is the euid; no group or other write bits; a symlinked runtime dir is checked at its target (is following it right, or should a symlink owned by others be refused?); refusal logged with path and mode. `cargo test -p chora-scene paths`.
2) private_dir mode rule: no 077 bits accepted (0700, 02700, 0500, 01700). Is 0500 (not writable by the owner) a dir chora can then use, or does a later create fail without a clear message? Is creation still 0700 regardless of umask?
3) One helper runtime_subdir_in used by listen (crates/chora/src/ipc.rs), kit runtime_file (crates/chora-kit/src/lib.rs), icons.rs, chora-draw theme.rs cache_dir, tiling theme.rs; no caller left that makes a dir in the runtime dir by itself (`git grep -n "create_dir\|XDG_RUNTIME_DIR\|runtime_home" -- crates plugins services`).
4) Races: path-based check then use (the lane chose no fd). Name any window where another user can swap the dir between check and use, and whether the owner-only parent closes it.
5) POLS: a group-writable or foreign runtime dir now stops chora at start. Message clear (path, mode, what to do)? Icons fall back to the temp dir only when the runtime dir is unset; is that temp dir path safe (private, not predictable)?
6) voice (services/voice/src/coder.rs, main.rs): no relative path; refusal message clear.
7) Red first: each red commit before its fix (`git log --oneline a61e8eaa..cb2976c1`); tests named in the report exist and pass (`cargo test -p chora-scene -p chora-kit -p chora-draw`, `cargo test -p chora ipc`).
8) CHANGELOG lines true to the behaviour.
Known, not findings: xp launch.rs:167, construct cube start, voice config.rs:1306 fallbacks (report "Not done"); fd-based Refactor row and copies rows (backlog).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity, file:line, text, fix in one sentence). Append to reports/kit-sec.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
