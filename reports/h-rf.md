# h-rf: red-first per finding id

## r1

Branch `claude/h-rf` from bc153929. Head `fff3f1ab`.

### Commits

- `73525d9c` test(rf): red-first cases per finding id. 8 new cases red against the old rule (before the change, with the rest of the new cases green)
  (the other new cases hold before and after).
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
New cases (`per id` and `the hook` lines): fix sh-1 with test sh-1 green; fix sh-2 with test sh-1 red;
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
- 11-23 14:50 PM: item 3 decided: keep the strict rule (the red commit names the fix ids); 0483bbeb, 3a625261 are history below every future base. Seeded review: briefs/review-h-rf-r1.md.

## Review r1

Seeded review of `claude/h-rf` (head `fff3f1ab`, base `bc153929`), one round, rule 8 guard. `cargo check --workspace` green. No merge gate run.

Verdict: **FIX** (1 MAJOR, 2 MINOR, 3 NIT; no BLOCK).

### Checks

1. Red first: OK. `bash tests/merge-gate-selftest.sh` at `73525d9c`: 195 ok, 10 FAIL (the per id and the hook cases; the report says 8), rc 1. At `fff3f1ab`: 206 ok, no FAIL, rc 0. `73525d9c` is an ancestor of `fff3f1ab` and holds only the selftest (+110).
2. Rule exact (tests/red-first.sh): id regex, fix only, per id earlier non-merge test commit or a test in the fix, `No test:`, MERGE_HEAD, and the no-base, base-branch and unborn-HEAD exits all hold as specified. A false red on a valid commit: rhr-1, rhr-2. A false green: rhr-4 (rare).
3. Gate step: same rule over the range, prints the sha and one line per id. It re-implements the verdict loop instead of calling the hook's function: rhr-3.
4. In flight (`tests/merge-gate.sh --red-first-check --base origin/claude/autonomous-dev-session-3w-i33pg7` with the scripts of `fff3f1ab` over each branch head). Report, not findings:
   - claude/co-reach `bc153929`, claude/docs-rc-r2 `bc153929`: empty range, pass.
   - claude/leak `677790dc` (3 commits), claude/guard-3 `49f65382` (4 commits): pass.
   - claude/xp-readable `2d2413f6` (8 commits): 6 fix commits fail, each a valid pair whose red `13d31263` names no id (it names only xp-9): `d78fc4bb` (xp-19, xp-39), `7f5e3f39` (xp-40), `f1693fb3` (xp-41), `54f810dc` (xp-10), `792ffae4` (xp-13, in the body), `0e9e0bd3` (xp-13). Fix on that branch: the red commit names these ids (reword `13d31263`), or `No test:` is wrong here since the red exists.
   - Extra: over the last 290 non-merge commits of the session head the rule fails 7 fixes (the two known, `0483bbeb`, `3a625261`, plus `f5bdc61f` co-40/co-41, `ac327bc0` rw-4, `cfc805d4` co-42, `92695718` rv-6, `e22f824d` rd-5); all are history below every base.
5. Scope: OK. Four files only: tests/red-first.sh, tests/merge-gate.sh, tests/merge-gate-selftest.sh, CONTRIBUTING.md.

### Findings

- rhr-1, MAJOR, tests/red-first.sh:109,115: the hook reads ids from the whole message file; `git commit -v` leaves the staged diff below the scissors line in that file (checked: the commit-msg hook receives it), so every id-shaped word of the diff (`utf-8`, `sha-256`, an id in a comment) becomes a named id and a valid fix is refused. Fix: cut the message at the `# ------------------------ >8 ------------------------` line before `grep -v '^#'`, and add a selftest case with a diff below the scissors line.
- rhr-2, MINOR, tests/red-first.sh:41-43: the regex also matches words that are no finding id: `utf-8`, `sha-256`, `base-64`, `ipv-6`, `gate-0.8` (reads `gate-0`; real: `f666a025` body), `gtk-3.0` (reads `gtk-3`); such a fix is red until some test commit repeats the word. Fix: skip a word followed by `.` and a digit, and a short denylist (utf, sha, base, ipv, gate, gtk), or say in CONTRIBUTING.md that these words need the same word in the test commit.
- rhr-3, MINOR, tests/merge-gate.sh:187-219: the gate step has its own loop and test-id accumulation next to the hook's `id_verdict`/`branch_test_ids`; two copies of the per-id verdict (the brief asks for one code path). Fix: one function in tests/red-first.sh taking the sha, its ids and the earlier test ids, used by the hook (HEAD) and by the gate loop.
- rhr-4, NIT, tests/red-first.sh:42: ids glued by a hyphen or a letter run (`sh-33-sh-34`, `sh-33ab`, `sh-1-`) match nothing, so such a fix is green with no test; `sh-1.2` reads `sh-1`. Fix: split also on `-` where a digit run ends (or accept and note it in CONTRIBUTING.md).
- rhr-5, NIT, tests/red-first.sh:115 (design note): a secondary id mentioned in the body as a side note makes the fix red (`ac327bc0`: "rw-4: the doc paragraph moves"). The rule says "names"; the PM decision 11-23 14:50 keeps it strict. No change asked; lane briefs should say "name in the fix only the ids it closes".
- rhr-6, NIT, reports/h-rf.md "## r1": says 8 new cases are red at `73525d9c`; the selftest shows 10 FAIL lines. Fix: correct the number.
- 11-23 15:00 PM: review r1 FIX (rhr-1 MAJOR). Decision: in-flight commits before the cut keep the old rule (no reword, no force). r2: briefs/h-rf-r2.md.

## r2

Branch `claude/h-rf` from `fff3f1ab` (r1 head). Head `c9d8d9f0`, pushed. `cargo check --workspace` green.
Scope held: tests/red-first.sh, tests/merge-gate.sh, tests/merge-gate-selftest.sh, CONTRIBUTING.md. No history change.

### Shas per id (test commit red first, then the fix)

| id | red (selftest only) | fix | red count at the red commit |
| --- | --- | --- | --- |
| rhr-1 MAJOR | `ad2c46a4` | `0e4ce1b6` | 1 FAIL: `the hook reads no id below the scissors line` (full selftest: 204 ok, 1 FAIL, rc 1) |
| rhr-2 MINOR | `370344ac` | `b8db5679` | 7 FAIL: utf-8, sha-256, base-64, ipv-6, gate-0.8, gtk-3.0, `(gate-0.8).` (`v0.9.0-rc.3` holds before and after) |
| rhr-4 NIT | `cbab2f3d` | `20e4ce3f` | 1 FAIL: `sh-33-sh-34`, the id without a test is not named |
| rhr-3 MINOR | `2d06e357` | `723aae31` | 1 FAIL: the hook and the gate word a missing id alike |
| in flight | `caa69301` | `6a27ad04` | 2 FAIL: an old pair passes by the old rule; the old fix beside a new one is not named |
| cut constant | | `c9d8d9f0` | `per_id_rule_from=1795446505`, committer date of this head (`2026-11-23T15:08:25Z`, forced with GIT_COMMITTER_DATE) |

Red counts come from the red-first section of the selftest run alone (6 s) on the red commit.

- rhr-1: `sed '/^# -* >8 -*$/,$d'` before `grep -v '^#'`. Cases: a diff below the scissors line with `sh-99` and `utf-8` passes; an id above the scissors line is still read (red for sh-2).
- rhr-2: `finding_ids` drops a word followed by `.` and a digit, and the prefixes utf, sha, base, ipv, gate, gtk. A full stop after an id still reads the id (`sh-1.` green; `sh-1, sh-2.` names sh-2).
- rhr-3: `id_verdict IDS TEST_IDS LABEL` in tests/red-first.sh prints `red-first: <label> misses a test commit before it that names <id>` per missing id; the hook (`hook_id_verdict`, label `this commit`) and the gate step (label the short sha) call it. The gate loop keeps only its accumulation of the test ids, no second verdict.
- rhr-4: split where a digit run ends (`sh-33-sh-34` = sh-33, sh-34). A word with `.digit` (`sh-1.2`) is no id by rhr-2.
- In flight (PM decision 11-23 15:00): `fix_ids` returns no ids for a commit whose committer time is before `per_id_rule_from`; the gate step then applies the any-test rule (a test in the fix or before it). Selftest: old pair (test with no id, then fix naming sh-1, dated 2026-11-01) green; the same pair dated now red naming sh-1; an old fix without any test still red; a new fix beside an old pair read per id. The hook sees only commits made now, so it always applies the per id rule. No reword, no history change on any branch.
- rhr-6: the "## r1" count of 8 new cases red at `73525d9c` is wrong: review r1 counted 10 FAIL lines at that commit (the per id and the hook cases). Correct number: 10.
- rhr-5: no change (design note); CONTRIBUTING.md now says: name in a fix only the ids it closes.

### Selftest

`bash tests/merge-gate-selftest.sh` at `c9d8d9f0`: 226 ok, 0 FAIL, `ok: merge-gate-selftest`, rc 0.

### Item 4 (`--red-first-check --base origin/claude/autonomous-dev-session-3w-i33pg7`, scripts of `c9d8d9f0` over each head)

| branch | head | non-merge commits | result |
| --- | --- | --- | --- |
| claude/xp-readable | `fc9675ac` | 13 | pass (the 6 fixes of r1, dated before the cut, use the old rule) |
| claude/guard-3 | `49f65382` | 4 | pass |
| claude/co-reach | `bc153929` | 0 | pass (empty range) |
| claude/tiling-disc | `8df47cc8` | 13 | pass |
| claude/leak | `677790dc` | 3 | pass |

### Gate (`tests/merge-gate.sh --jobs 2`, `c9d8d9f0` detached; the session head `bc153929` is already in it, so the merge is a no-op)

Exit 1. PASS: red-first, identity, pre-commit, gate-lib, ci-check, build, test, clippy, 2d-guard,
package-deps, package-lists, xp, virtual, xp-faces, xp-paperclip, xp-question, xp-chat-idle, xp-helper, matrix,
xp-notepad, hud, tiling-audit, tiling-look, tiling-empty, chat-keys, layout, doors, last-words, cube, pull,
placed, construct-hud, construct-extra, out, winit, wm-pane, pane-keys, walk.
SKIP (foreign, fuzzel 1.9.2 < 1.11, sh-34): `xp-fields`: `SKIP: cell runbox_cell: fuzzel 1.9.2 < 1.11 does not
read the file of xp: the baseline assert ...` (xp-fields.log).
FAIL (foreign, same cell as r1): `keys`: `FAIL: the Foot row of Programs started no foot` (keys.log, fuzzel 1.9.2).
The gate stops after the failed step, so pixels, mc-paths, mc-sofa, mc-column, mc-handover, launcher-click,
voice-acp, voice-events, agent, e2e did not run (cost limit; the change is shell scripts of the red-first rule
and a doc paragraph, no Rust). I did not rerun to green.
