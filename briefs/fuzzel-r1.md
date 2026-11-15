# fuzzel r1 (rc 1): build fuzzel >= 1.11 under $HOME, then check ti-1

Model: sonnet, effort low (a build recipe and one click check)
Lane: cloud. Two places: setup.sh on claude/session-log (owner 11-15 09:27 "Yes, build under
$HOME"), and, only if the click fails, a branch claude/fuzzel from the session branch
claude/autonomous-dev-session-3w-i33pg7 (1014dddf or newer). One round. Stop at 250 k
context and hand over. Read dev-rules.md and conventions.md first, then findings/tiling.md
ti-1 and answers.md 11-14 22:16, 11-15 09:27. Setup as briefs/flakes-r2.md.

1. setup.sh: build fuzzel 1.11 or newer (a tagged release, pinned by version and sha256 of
   the tarball) under $HOME/.local with its build deps not in the distro at the right version
   (meson via pip, tllist, fcft) also under $HOME; no sudo, no apt for these. Idempotent:
   skip when `fuzzel --version` already says >= 1.11. Put $HOME/.local/bin first on PATH for
   the gates. Keep the apt line as is (fallback). Commit to claude/session-log (stage by
   name, pull --rebase first); it must pass the log hooks.
2. Check ti-1 headless: tiling look, open the Apps menu, move the pointer to a row and click
   (wlrctl or the gate's pointer helper); the program on that row starts. Use the existing
   doors or tiling gate cell if one covers it (tests/doors/tiling.sh); say the cell and the
   capture. Run it 3 times.
3. If the click still fails with 1.11: stop and report the cause (no fix in this round).
Done when: setup.sh builds fuzzel >= 1.11 in a fresh container (say the time it adds), and
the ti-1 cell passes 3/3, or a cause is named.
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 12 lines) and as
reports/fuzzel-r1.md on claude/session-log. Rules as briefs/flakes-r2.md; push only
claude/session-log (setup.sh, the report) and, if needed, claude/fuzzel.
