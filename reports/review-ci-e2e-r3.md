# Review ci e2e r3 (claude/ci-e2e3), opus, read-only, 11-15 02:0x UTC

Verdict: FIX (skips count as passes). Test-logic fixes sound; no security assertion weakened (the confined refusal is the unit test of 1403eb50; the injected-keys variable only enables plugins' accept_injected; step 11 of agent.sh stays).
| id | sev | file:line | text |
|---|---|---|---|
| F1 | MAJOR | tests/gate-0.7-6.sh:15 | the xkb skip exits 0; in CI a missing rule would hide the grab-any check. Fail when $CI is set. |
| F2 | MAJOR | tests/e2e-sequence.sh:29-30 | only single-line run: tests/... lines; the multi-line font step (ci.yaml:154) is dropped, so gate-0.8-9 always skips locally; if/env/name ignored. |
| F3 | MAJOR | tests/e2e-sequence.sh:38-51 | friction 47: skipped scripts count as passed; missing tools only printed. Count SKIP lines and fail (or report them apart). |
| F4 | MINOR | tests/mcp-client.py:~225,~277,~400 | docstrings still say the agent moves the camera; fixed sleeps replace animation.done; add a check that the confined camera call is refused. |
| F5 | MINOR | tests/agent.sh:24 | the injected-keys variable exported for the whole script. |
| F6 | MINOR | tests/e2e-sequence.sh:19 | leaves a mktemp dir. |
