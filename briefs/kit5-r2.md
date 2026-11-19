# kit5 r2 (F2): kit5 on the session branch without the dev merge, gate proven

Model: sonnet, effort high (a named cherry-pick of finished work and a gate run; no new design)
Lane: cloud, new branch claude/kit5b from claude/autonomous-dev-session-3w-i33pg7 (head at start). One round, stop at 200 k. Cost about 1.5 %.
Run `git fetch --unshallow` first. claude/kit5 af0dc379 has d060dc03 (dev rc.4 merge) as an ancestor; the session
branch must not take dev history (owner 11-09: never touch main or dev). Read reports/kit5.md on claude/session-log.
1. Cherry-pick, in order, the kit5 commits only: `git log --reverse --no-merges origin/claude/autonomous-dev-session-3w-i33pg7..origin/claude/kit5 ^origin/dev`
   (fetch dev first),
   skipping 6aad3457 (release-file restore) and any commit that only touches release files. Resolve conflicts to
   both sides; name each in the report. `git log origin/claude/autonomous-dev-session-3w-i33pg7..HEAD` must show no
   commit of dev and no merge.
2. No other change. A cell that breaks: fix in its own commit, red test first, name it.
Done when: `cargo test -p chora-kit --features testing -p chora-notify -p chora-xp -p chora-mc -p chora-tiling
-p chora-construct`; kit notice.rs coverage >= 80 %; `tests/merge-gate.sh` full run green on the head (it is the
open item of r1; poll in the same turn).
Push claude/kit5b. Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines:
head sha, picks, skips, gate) and append "## r2" to reports/kit5.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
