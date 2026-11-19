# id-gate

## r1

Branch claude/id-gate, head a9fd91f3 (from train 72 merge 219d5afe).

- tests/merge-gate.sh: new step `identity` right after `red-first` (Steps
  comment, default steps, serial list, `--identity-check`, new `--list`
  because none existed). The expected identity is the one variable
  `expected_identity` at the top; `MERGE_GATE_IDENTITY` overrides it for
  the selftest. Checks author and committer of every commit of
  BASE..HEAD, merges included; one `FAIL: <sha> author|committer is X,
  want Y` line each.
- tests/merge-gate-selftest.sh: throwaway repo; bad committer, bad
  author, bad merge committer fail; all good and a bad commit before the
  base pass; the step follows red-first. Red first in ec2a69bc (8 checks
  failed), green in a9fd91f3.
- CONTRIBUTING.md: the merge gate paragraph names `identity` and `--list`.
- Gate `--jobs 3 --only pre-commit,ci-check,red-first,identity,gate-lib`:
  all PASS (ci-check 101 s). merge-gate-selftest passes.
- Not done: the stop-hook side (start_lane.sh check) stays in the backlog.

## r1 PM review (2026-11-19 21:06 UTC; PM, not the builder)
Base 219d5afe, head a9fd91f3; 3 files, +92 -3. Identity of both commits: Marcel Arpogaus (author, committer). Red first: ec2a69bc selftest before a9fd91f3.
- Item 1: step `identity` after red-first, serial; one variable `expected_identity`; FAIL line per sha with role and identity; merges included (git log BASE..HEAD). PM check: `--identity-check --base` session head on a9fd91f3 green; `--list` prints the steps.
- Item 2: tests/merge-gate-selftest.sh 7 identity cases (good, bad committer, bad author, merge with bad committer, before base). PM run: selftest all ok (72 s).
- Item 3: CONTRIBUTING.md names the step.
- MINOR ig-1 (pre-existing, harness): no gate step and no CI job runs tests/merge-gate-selftest.sh, so the new cases guard nothing on their own. To backlog: a gate-lib (or ci-check) call of the selftest.
- NIT ig-2: MERGE_GATE_IDENTITY in the environment replaces the expected identity without a line in the log; print it when set.
Findings: no MAJOR. Verdict (computed): MERGE-OK a9fd91f3. Harness branch: its own train next (train 74), right after train 73 (one gate at a time).
