# logging r2 (rc 9a): fix round after review-logging-r1 (FIX)

Lane: opus, cloud, same branch claude/logging-r1 (150db8f0). First merge the session branch
claude/autonomous-dev-session-3w-i33pg7 (be205f89 or newer). One round. Stop at 250 k context and
hand over. Read dev-rules.md and conventions.md first, then reports/review-logging-r1.md (all
findings with file:line and repro). flakes r5 works in tests/ and the xp theme cache; harness r3 in
tests/merge-gate.sh, tests/red-first.sh and hooks: keep away, except tests/battle/w16.sh (item 4).
Items, each fix with its red test commit first (a separate commit, red on its own):
1. M1: a second chora or any chora-voice CLI call must not rotate a live log. Rotate only when the
   process holds an exclusive flock on the log; without the lock write nothing to the file (stderr
   only). chora-voice opens its log only when the agent runs, not for --version or subcommands.
   tests/headless-session.sh gets a private XDG_STATE_HOME. Test: two writers, the first keeps its
   file.
2. M2: chora-report scrubs block-YAML secrets (key on one line, indented values below), hyphenated
   and case-varied header names (X-Api-Key, Api-Key, Authorization), repeated user or host names
   (MINOR 4, ERE escaping too), follows symlinks (MINOR 5, cp -RL), umask 077 (MINOR 6). Extend the
   fixture with each shape.
3. MINOR 1 (cap rmcp at warn in the filter), MINOR 2 (O_NOFOLLOW, fix modes of an existing dir and
   file), MINOR 7 (the tests build the subscriber through the same function init() uses, so removing
   the filter from init() turns a test red).
4. MINOR 8: tests/battle/w16.sh lines 147 and 153 use a signal that still exists (the confirm event,
   a tool event with the program name only); line 153 must be able to fire again. Run w16 once.
5. MINOR 12: docs/logs.md says exactly what a log may hold (argv at info, URLs in errors,
   CHORA_VOICE_LLM_DUMP) and that every chora-voice CLI call no longer rotates.
MINOR 3, 9, 10, 11 go to backlog L10-L13 (PM); do not fix them.
Output: the full tests/merge-gate.sh once, green; reports/logging-r2.md on claude/session-log (item,
red commit, fix commit, cell). Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines).
Rules: push only claude/logging-r1 and claude/session-log (reports); author Marcel Arpogaus
<38564291+MArpogaus@users.noreply.github.com>; no AI attribution lines; never --no-verify; never end a
turn to wait for a long background job.
