
## r1
Code branch claude/h-guards head 9b25d65e (pushed); log branch commits below.

| Item | Red | Green |
|---|---|---|
| 1 ratchet-raise commit-msg hook (tests/ratchet-raise.sh, wired in .pre-commit-config.yaml, merge-gate, ci.yaml) | 93cbd2a3 | 9b25d65e |
| 2 SC2178 tests/gate-injected-presses.sh:54 | none | none: does not reproduce |
| 3 H-ID hooks (tools/hooks/pre-commit identity, tools/hooks/commit-msg trailers; install_hooks.sh installs them as .legacy hooks that the pre-commit framework hook runs first) | 6f31b9bd | c8a65b32 |
| 4 lanes.py Queue budget warning | 02429700 | 5e8dce4c |

Selftest output:
- ratchet-raise: ok raised count, no line (fail); ok with the line (pass); ok lower count (pass); ok same count (pass)
- identity: 5 ok lines (owner pass; another author, committer, mail fail; another branch pass); trailers: 4 ok lines
- lanes budget: 4 tests OK; tests/ 57 tests OK

Item 2: `pre-commit run --all-files shellcheck` (shellcheck-py 0.11.0.1) passes on a84effda, no SC2178 anywhere; nothing changed. If the warning came from another shellcheck version, send its output.

On HEAD of claude/session-log: tools/hooks/pre-commit exit 0, tools/hooks/commit-msg on the last message exit 0, lanes.py prints no budget flag (queue plans about 7 %, room 89.4 %).
Selftests for the log hooks and lanes sit in tools/ (not tests/) because the brief limits changes to tools/.

Gate: tests/merge-gate.sh --jobs 1 on a84effda plus the two code commits (autonomous branch already contained): all steps passed (51 lines, no red), log /tmp/merge-gate.A692lU.
