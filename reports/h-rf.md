# h-rf: red-first per finding id

## r1

Branch `claude/h-rf` from bc153929. Head `fff3f1ab`.

### Commits

- `73525d9c` test(rf): red-first cases per finding id. 8 of the 42 new cases red against the old rule
  (the other new cases are green before and after: the rule they guard was already true).
  Red output before the change:
  `FAIL: per id: a fix naming sh-2 after a test naming only sh-1 is red: got '0', want '1'`,
  `... the red names the id and the sha`, `... a fix naming sh-1 and sh-2 with a test for one is red`,
  `... that red names the id without a test`, `the hook refuses a fix naming an id that has no test commit`,
  `the hook names that id`, `the hook reads the id in the body`,
  `the hook refuses a fix naming an id when the test commit names another`.
- `fff3f1ab` fix(rf): red first per finding id (H-RF). tests/red-first.sh (shared rule), the red-first
  step of tests/merge-gate.sh, CONTRIBUTING.md (one list item). Cases live in tests/merge-gate-selftest.sh
  (the selftest that already proves red-first; a new file would need wiring in the gate-lib step).

### Rule

- id: a word of subject or body matching `[a-z]{1,6}-[0-9]+[a-z]?` (words split on anything but A-Za-z0-9-;
  `SH-1` and `tiling-audit` are not ids).
- Only `fix` commits. A fix naming ids: each id needs an earlier non-merge commit in base..HEAD that names
  it (subject or body) and changes a test, or a test change in the fix itself (staged for the hook).
  feat and fixes without an id keep the any-test rule. `No test:` passes. Merge and MERGE_HEAD pass.
- Hook: refuses only what it can prove (no-base, base-branch, unborn HEAD notes kept).
- Gate: prints `RED-FIRST MISSING: <sha> <subject>` and one `red-first: <sha> misses a test commit before it
  that names <id>` line per id.

### Selftest

tests/merge-gate-selftest.sh: three runs, all `ok: merge-gate-selftest`, rc 0, no FAIL line
(run 1 on the working tree before the commit, runs 2 and 3 on `fff3f1ab`; ~2 min each).
New cases (42 `per id` and `the hook` lines): fix sh-1 with test sh-1 green; fix sh-2 with test sh-1 red;
sh-1+sh-2 with both green, with one red (only the missing id named); id in body of fix or test commit;
fix without id green after any test, red without; `No test:` green; test in the fix commit green; test commit
after the fix red; feat naming an id green; SH-2 / tiling-audit not ids; merge commit naming an id green;
hook: same cases staged.

### Item 3: last 30 non-merge commits (`git log --no-merges -30 bc153929`)

Run: the new gate step over `61f3b19e^..bc153929` (75 non-merge commits, so each fix sees its pair).
Two commits of the 30 would now fail:

1. `0483bbeb fix(doors): the helper word of the bar is waited for` names `ti-46` (body). Its red is
   `ded5d30d test(doors): ... arrives late (red)`, which names no id. VALID red-first pair.
2. `3a625261 fix: a shut menu takes no keys; Start on the front face` names `rcf-7`, `rcf-9` (body). Its red
   is `35fa4b8d test: ... (red)`, which names no id. VALID red-first pair.

Item 3 is NOT met as the brief words it ("none may be a valid red-first pair"): both are valid pairs whose
test commit omits the id. The rule as specified fails them on purpose (it asks the test commit to name the
id); it is not a bug of the script. The other 28 pass. PM decision needed: keep the rule (lane briefs then
say: the red commit names the same ids as the fix) or relax it (for example: an id also counts when the
immediately preceding commit is a test commit whose subject ends in `(red)`). I did not relax it.

### Gate (`tests/merge-gate.sh --jobs 2`, no --only, on `fff3f1ab`)

PASS: red-first, identity, pre-commit, gate-lib, ci-check, build, test, clippy, 2d-guard, package-deps,
package-lists, xp, xp-faces, xp-paperclip, xp-notepad (and the other xp steps), matrix, virtual, hud,
tiling-audit, tiling-look, tiling-empty, chat-keys, layout, doors, last-words, cube, pull, placed,
construct-hud, construct-extra, out, winit, wm-pane, pane-keys, walk.
FAIL: `keys`: `FAIL: the Foot row of Programs started no foot` (fuzzel 1.9.2 in the lane container, the
known foreign cell). The gate stops after a failed step, so pixels, mc-paths, mc-sofa, mc-column, mc-handover,
launcher-click, voice-acp, voice-events, agent, e2e did not run. The change touches only shell scripts of the
red-first rule and a doc line, so I did not run them with --only (cost limit).
