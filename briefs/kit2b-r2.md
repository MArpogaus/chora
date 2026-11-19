# kit2b r2 (F2): fix round after review r1 (FIX, rk-1 MAJOR)

Model: sonnet, effort medium (small fix on settled code)
Lane: cloud, branch claude/kit2b at ce0a564d (continue on it). One round, stop at 250 k. Cost about 0.8 %.
Read reports/kit2b.md "## Review r1" on claude/session-log.
PM decision (KISS, no new feature before the rc, rules.md 1): construct does not act on Run or Give.
1. rk-1: red cell first (a construct config with `{run: ..}` and `{give: ..}` bindings: today both
   act), then refuse them in acts_in_construct with one warning per refused binding naming the key; the
   rest of the file loads. Remove the construct claim in docs/configuration.md:939.
2. rk-2: with both `keys` and `bindings`, `bindings` wins and one warning names the dropped `keys`; a
   cell. rk-3: settings.set with `keys` writes `bindings` (or the renamed() doc says it is refused; pick
   the simpler, a cell either way).
3. NITs only if one line each.
Done when: cells red then green; merge gate once on the head with --jobs 1 for `cube` if it flakes
under load (poll it in the same turn); report the cell names.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines)
and append "## r2" to reports/kit2b.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
