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
