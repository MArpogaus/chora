# flakes-f r1 (F5, F17): causes of the gate flakes that cost trains

Model: Fable, effort high, because timing races span core, gate scripts and load; each red train costs about
55 min of the one serial merge gate, which caps merged work this week (owner 06:12: Fable where it pays)
Lane: cloud, branch claude/flakes-f from dc4bc46d (train 56 head). One round, stop at 250 k. Cost about 3 %.
Run `git fetch --unshallow` first.
Seeds (read these, not the whole repo): backlog.md items F17 (line 120), F21 (145), F26 (164),
F28 (173), F29 (177) and the pull row (217) on claude/session-log; reports/kit2b.md (cube failed once
under --jobs 3, passed alone); reports/plan-critique-1119.md (trains ran about 1 per 3 h, mostly reruns).
1. For each flake, name the cause with file:line and a reproducer that fails reliably under load (for
   example `stress-ng` or a busy loop at --jobs 3, or a forced delay); commit the reproducers first (red).
2. Fix the cause, not the wait: no longer sleeps, no retries that hide a real bug; `until_ok`-style polling
   on the observed state is fine where the gate only measures. The core escape hatch stays above plugins.
3. Prove it: each fixed step 10 runs under --jobs 3 with the reproducer load, all pass; name the counts.
4. Order by trains saved: cube and pull (F21), tiling-audit (F29), doors (F26), chat-keys (F28), F17 last.
   Stop after the ones you can prove; list the rest with what you found.
Done when: reproducers red, then green 10 of 10; merge gate once on the head (poll it in the same turn).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/flakes-f.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
