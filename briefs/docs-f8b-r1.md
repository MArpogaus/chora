# docs-f8b r1 (F8): decision records and where shared code lives

Model: sonnet, effort medium (docs from settled decisions)
Lane: cloud, branch claude/docs-f8b from a17e7858 (train 57 head). First `git fetch --unshallow`; merge
origin/claude/autonomous-dev-session-3w-i33pg7 when it moves. One round, stop at 200 k. Cost about 1.5 %.
Read design/foundation.md F8, reports/arch-plan-fable.md section 2, backlog.md cut2 and kit6 entries,
reports/kit6.md (Review r1, D104 wording) on claude/session-log.
1. docs/decisions.md: the D65 amendment line (look.theme, look.view; core stamps data.look; cut2 review);
   D104 in the reviewer's wording (shell wrapper, status file, exit 0 waits 30 s, a sh beside each app).
   Records for the merged structure (kit::wm::book, focus, launch, cut gate) only where none exists; each names its merge.
2. CONTRIBUTING.md and the dev rules: where shared code lives (chora-kit), the copies.toml check, the cut gate.
3. Only what is merged on the session branch; claims name file:line. No code change.
Done when: `tests/merge-gate.sh` steps pre-commit and gate-lib green; docs link check if the repo has one.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/docs-f8b.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job.
Commits by the owner identity, no attribution lines, never --no-verify.
