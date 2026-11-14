# core r2 (rc 6): open security MEDIUMs of the core review

Lane: opus, cloud, new branch claude/core-r2 from claude/review-core at eff84702 (MERGE-OK,
merges into the session branch soon; merge the session branch in when it moves). One round.
Stop at 250 k context and hand over. Read dev-rules.md and conventions.md (with the skills it
names) first. Setup: clone MArpogaus/chora to /home/user/chora, fetch all branches, worktree
/home/user/wt/log on claude/session-log, `bash /home/user/wt/log/setup.sh`.
Items (reports/review-core-r1.md, reports/review-core-fixes-r1.md); PM decisions of 11-14
17:27 UTC (owner delegated 17:17), secure defaults:
1. C05: gate output.configure and output.reserve like input.configure (methods.privileged);
   give the voice agent's MCP tools configure_output and reserve_space the grant they need,
   so the product behaviour stays for the agent and ends for an unprivileged client.
2. C14: injected input no longer counts as activity for idle-notify (a client with
   input.inject cannot keep the session from locking). Document it in docs/api.md.
3. V02: utterances, reply sentences and coder tasks log at debug, not info; adapt the live
   gate and docs/voice.md section 11 so they read the debug level.
4. C13: trace the injection mark; draw it after the overlay roots so no client node covers it.
5. CR-1: no URL with userinfo or query in any log line of services/voice/src/llm.rs.
6. CR-6: keep *_api_key_env through the harness (no key loss).
7. CR-4, CR-5: amend D23 for the lock; add 256 grabs, 64 KiB exports and code 6 to docs/api.md.
Method: red test first for each item, then the fix; push after every commit.
Done when: tests/merge-gate.sh passes (the cube cell F14 may fail until claude/flakes merges;
say so); voice-*.sh run where the container allows, else say which did not run.
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 20 lines) and as reports/core-r2.md
on claude/session-log: per item fixed / not, commits, gate.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution
lines; never --no-verify; stage by name; no force push; push only claude/core-r2 and that
report; never check out another branch in an existing worktree.
