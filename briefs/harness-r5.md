# harness r5 (rc 3, owner 11-15 02:10): close review-harness-r4 (FIX: one MAJOR)

Lane: new opus worker, branch claude/harness (continue it at 7ef5fe1f; worktree
/home/user/wt/harness5). One round, small and fixed. Read dev-rules.md, conventions.md, then
reports/review-harness-r4.md. Start: merge origin/claude/autonomous-dev-session-3w-i33pg7 (newest).
Stop at 250 k context and hand over.
Items, a red case first for each (separate test commit before the fix):
1. MAJOR: tests/gate-0.8-plugin-runtime.sh:21 - an allow window opened by
   `#[allow(clippy::x)] // reason` (trailing comment) never closes, so string calls after it are
   hidden (20 spans, 1217 lines in plugins/ today). Close the window on `)]` followed by optional
   whitespace and an optional `//` comment. Selftest: a `core.call("events.subscribe")` after such a
   line is refused; the three real allowed sites still pass. Then confirm on today's tree that no
   string call was hidden (report the count of spans before and after).
2. MINOR (blocks a valid commit): no-foreign-kill refuses `need pgrep pkill jq` and
   `command -v pgrep pkill jq`; any word order of tool names after need / command -v passes.
Not in scope (backlog R13): escaped quotes, here-doc substitutions, eval / bash -c, red-first on
tracking branches with another name, rt/b.
Output: the full tests/merge-gate.sh once (serial), green; tests/ci-local.sh once; reports/harness-r5.md
on claude/session-log. Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines).
Rules: push only claude/harness and claude/session-log (report); author Marcel Arpogaus
<38564291+MArpogaus@users.noreply.github.com>; no AI attribution lines; never --no-verify; never end a
turn to wait for a long background job; CARGO_TARGET_DIR inside your worktree only.
