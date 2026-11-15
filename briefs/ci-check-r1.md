# ci check r1 (rc 4): the scripts of the check job

Lane: sonnet, cloud, new branch claude/ci-check from the session branch (5dd34fcd). One round. Stop at
250 k context and hand over. Read dev-rules.md and conventions.md (with the skills it names) first.
Setup as briefs/ci-r1.md. Never end a turn to wait for a long job.
On PR #106 (run 43659214313, job 132644545000) the check job now passes its tests and fails
tests/gate-0.8-plugin-runtime.sh: two plugins call the core themselves instead of through the
chora-kit runtime:
  plugins/chora-construct/src/cube.rs:1592 conn.call("output.list", ...)
  plugins/chora-assistant/src/chat/host.rs:674 call("events.subscribe", ...)
Items:
1. Route both through the chora-kit runtime (its output list and its subscription), the rule of the
   gate (no plugin connects, exports or subscribes by itself); no change to the gate pattern.
2. Run every script the check job of .github/workflows/ci.yaml runs (agent-sandbox.sh, gate-0.7-9,
   gate-0.8-5, -8, plugin-runtime, and any others in that job) locally; fix each failure at its cause.
3. Add these check-job scripts to the CI-mirror step that ci e2e r3 adds to tests/merge-gate.sh (if it
   is not merged yet, add a step "ci-check" that runs them; the two lanes merge in one train).
Done when: every check-job script passes locally; tests/merge-gate.sh passes. Report to the PM
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 10 lines).
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution lines;
never --no-verify; stage by name; no force push; push only claude/ci-check; never check out another
branch in an existing worktree.
