# harness r1 (rc 3, rc 4): shorten the loop to the rc

Lane: opus, cloud, new branch claude/harness from the session branch (8bcbdc73 or newer). One round.
Stop at 250 k context and hand over. Read dev-rules.md and conventions.md (with the skills it names)
first. Setup as briefs/ci-r1.md. Owner 11-15 push-back: sharpen the harness now where it speeds up
convergence. Rule: no change that makes existing code red (no churn in plugins); every new check must
pass on the current tree.
Context: tonight's rework came from checks that ran late or lied: CI-only scripts (fonts, plugin-runtime,
e2e), skips counted as passes (friction 47), a fix commit without a test caught only by the gate, five
review rounds on mc focus. Lanes ci e2e r4 and ci check already mirror CI in the gate and stop
counting skips; do not duplicate them.
Items:
1. Architecture rules at compile time: the rules that tests/gate-0.8-plugin-runtime.sh and similar
   grep gates check become clippy `disallowed-methods`/`disallowed-types` in clippy.toml (or the kit's
   raw connection not public to plugins), so the editor and clippy flag them at once; keep the grep
   gate as a backstop. Only rules the tree already meets.
2. Red-first at commit time: the commit-msg or pre-commit hook rejects a fix/feat commit with no test
   change in it or before it on the branch and no "No test:" line (the gate's red-first rule, run
   early); ci/docs/chore/test types are free.
3. Gate time: measure each step of tests/merge-gate.sh on this container; propose and build the cheapest
   cut that keeps every check (e.g. one shared build for all steps, running independent steps in
   parallel where they use separate sockets, a --since mode that skips the steps a diff cannot touch for
   lane runs only, never for the train). Report minutes before and after.
4. Builder checklist: add to dev-rules.md the review findings that recurred tonight (state that is set
   but never cleared, skips, commit types, hand-written CHANGELOG, docs in step, CI-only checks) as a
   short self-check before a lane reports.
Done when: tests/merge-gate.sh passes; each new check shown red on a planted violation and green on the
tree. Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines) and reports/harness-r1.md on
claude/session-log.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution lines;
never --no-verify; stage by name; no force push; push only claude/harness and that report; never end a
turn to wait for a long background job.
