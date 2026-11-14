# review train 5 r2: mc and core (rc 1, rc 6)

Lane: opus, cloud, read-only on code. One round. Read dev-rules.md and conventions.md (with
the skills it names) first. Setup: clone MArpogaus/chora to /home/user/chora, fetch all
branches, worktree /home/user/wt/log on claude/session-log, `bash /home/user/wt/log/setup.sh`.
Branches against the session branch claude/autonomous-dev-session-3w-i33pg7 (498d9dea):
- claude/mc-fix at 4feb4dc0 (brief briefs/mc-r1.md; worker report in state.md history: 22 of
  23 gate steps, cube fails on F14).
- claude/review-core at eff84702 (brief briefs/review-core-r1.md; report
  reports/review-core-r1.md: 37 commits, fixes of C01-C03, V01 and 18 MEDIUM).
Review only `git diff <session branch>...<branch>`. Check: each fix matches its finding;
red test before each fix (`tests/merge-gate.sh --red-first-check`); regressions in the other
looks and in automation (core: the behaviour changes listed at the end of the core report:
safety chords off while locked, confirm key needs 1 s, 256 grabs, 64 KiB exports, code 6,
confined headless outputs; say whether each is right and documented); simplicity (KISS,
POLS); docs in step. Run `tests/merge-gate.sh --worker ''` on each branch in its own worktree
(git worktree add /home/user/wt/<name>; never check out another branch in an existing
worktree). The cube cell (F14) fails on the base too: do not count it.
Write reports/review-mc-r1.md and reports/review-core-fixes-r1.md on claude/session-log, one
line per finding (id, BLOCK/MAJOR/MINOR/COSMETIC, file:line, text, evidence); verdict per
branch: an open BLOCK or MAJOR is FIX, else MERGE-OK.
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines).
Rules: push only claude/session-log (fetch and rebase on reject, no force), only those
reports; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI
attribution lines; never --no-verify. Stop at 250 k context and hand over.
