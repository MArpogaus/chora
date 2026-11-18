# net2-adapters r2 (F3): re-merge round after harness r2 (train 49)

Model: sonnet, effort medium
Lane: cloud, branch claude/net2-adapters (head 5da2712c, review MERGE-OK 02:34), one round.
Stop at 250 k. Cost about 1 %. Run `git fetch --unshallow` first.
1. Merge the session branch (claude/autonomous-dev-session-3w-i33pg7, with harness r2 from
   train 49) into claude/net2-adapters; wait for its head to carry "Merge branch
   'claude/net2-harness'". Missing keys are now breaches "missing key <k>"; rows take `match`.
2. planted.py selftest "sound I4" reads mc's real state, which no longer has `open`: let its
   sound baseline inject `open` through `filtered` (do not put `"open": []` back in mc). Red first.
3. Each adapter answers a real `helper` key (the assistant's state, or "none" when no helper
   runs; never a fixed "idle"), so I8 gets no "missing key helper" once an act emits voice.state.
   One cell per look, red first.
4. Give each known-red row a `match` naming its breach text (mc I4: "missing key open"); add the
   `match` key to the known-red.toml header. No new rows except one a merge forces, with its
   finding and slice.
5. No look logic changes beyond 2-3; an adapter only reads.
Done when: the new cells red then green, merge gate once on the head (poll it in the same turn).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/net2-adapters.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
