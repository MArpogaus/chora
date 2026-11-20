# review kit-sec r2 (F5, security): seeded review of the r2 fixes on claude/kit-sec before its merge

Model: opus, effort high, because the diff changes the private dir and runtime dir rules (mkdir modes, canonical paths, the check before listen), where a wrong rule lets another user plant files or stops chora on a safe system; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/kit-sec head 6de357b9, r1 head cb2976c1, base 30727c82 (19 files, +902 -47 in all). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green).
Step 0: `git fetch origin claude/session-log claude/kit-sec claude/autonomous-dev-session-3w-i33pg7`; `cargo check --workspace` only if a check below runs tests; a failure: send_message the PM "review kit-sec r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Read briefs/kit-sec-r2.md (items 1-6) and reports/kit-sec.md "## Review r1" and "## r2" on claude/session-log; `git diff cb2976c1..6de357b9` first, then the whole `git diff origin/claude/autonomous-dev-session-3w-i33pg7...origin/claude/kit-sec`.
Checks, each with its command and files:
1) rw-1 (was MAJOR): flavors/tiling/waybar/waybar.sh and wallpaper.sh create with `mkdir -p -m 700`; note `-m` applies only to the last component: are the parents made safe or already private? tests/tiling-scripts-selftest.sh runs under umask 022 and fails on the old code (`bash tests/tiling-scripts-selftest.sh`; `git stash`-free: check out 1e6ba81a in a worktree if cheap).
2) rw-2: own dirs 0700, not 0755; CHANGELOG line true.
3) rw-3: the runtime dir check runs before `state.listen()` in crates/chora/src/main.rs; crates/chora/tests/runtime_dir.rs covers a group-writable and a foreign-owned dir (`cargo test -p chora --test runtime_dir`).
4) rw-4: runtime_subdir_in canonicalizes; a symlink owned by another user is refused or followed to a checked target; no TOCTOU window beyond the r1 known one.
5) rw-5: 0500 refused with path and mode in the error; message says what to do.
6) rw-6/rw-7: Display with a remedy; temp suffix random (not pid or time), created exclusively.
7) 6de357b9 (copies row): the XDG_RUNTIME_DIR row change in tools/copies.toml is a true count, not a loosened guard.
8) Red first: each red sha before its fix (`git log --oneline cb2976c1..6de357b9`); `cargo test -p chora-scene -p chora-kit -p chora-draw`.
Known, not findings: r1 "Not done" fallbacks (xp launch.rs:167, construct cube start, voice config.rs:1306); fd-based refactor (backlog); H-mkdir pre-commit check (backlog); the images signal-mask test that failed once in a full `cargo test -p chora` (flake backlog, not this diff).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N continuing after r1's ids, severity, file:line, text, fix in one sentence). Append to reports/kit-sec.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
