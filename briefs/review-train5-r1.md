# review train 5 r1: doors and xtc (rc 1, rc 6)

Lane: opus, cloud, read-only on code. One round. Fable is off this week.
Branches: claude/doors at ea47e27b (brief briefs/doors-r1.md) and claude/xtc-fix at
91fb7d96 (brief briefs/xtc-r1.md), each against the session branch
claude/autonomous-dev-session-3w-i33pg7. Review only `git diff <session branch>...<branch>`.
Setup: clone MArpogaus/chora to /home/user/chora, fetch all branches, add the worktree
/home/user/wt/log on claude/session-log, run `bash /home/user/wt/log/setup.sh`.
Check: does each commit do what its brief and its finding (findings/, reports/) ask; red
test before each fix (tests/merge-gate.sh --red-first-check --base <session branch>);
correctness, regressions in other looks, simplicity (KISS, POLS), docs in step; no
Microsoft files. Run `tests/merge-gate.sh --worker ''` on each branch in its own worktree
(/home/user/wt/<name>, git worktree add; never check out another branch in an existing
worktree). xtc reports two tiling gates failing locally: say whether they fail on the
session branch too.
Write reports/review-doors-r1.md and reports/review-xtc-r1.md on claude/session-log: one
line per finding (id, BLOCK/MAJOR/MINOR/COSMETIC, file:line, text, evidence). Verdict per
branch: an open MAJOR or BLOCK is FIX, else MERGE-OK.
Report (to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN, at most 15 lines): verdict per branch,
the MAJOR and BLOCK list.
Rules: push only claude/session-log (fetch and rebase on reject, no force), only those
reports; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI
attribution lines; never --no-verify. Stop at 250 k context and hand over.
