
## r1
- Head: 6ec50752 on claude/ci-head (base 1c599c74, already contains origin/claude/autonomous-dev-session-3w-i33pg7).
- Cause: tests/gate-injected-presses.sh:54,61,63,65: SC2178/SC2128 on `probes` (counter). Renamed to `probe_n`; no behaviour change.
- Step miss: tests/merge-gate.sh pre-commit step ran `--from-ref base --to-ref HEAD` (changed files only). Now `pre-commit run --all-files` (SKIP unchanged). `pre-commit --all-files` took about 2m02s.
- Red test: tests/merge-gate-selftest.sh checks the fake pre-commit gets `--all-files`. Red before (got `run --from-ref HEAD --to-ref HEAD`), green after.
- All-files run on 1c599c74: only shellcheck was red.
- Full gate `--jobs 1`: all steps passed, rc 0, no red cells.
