# Review ci e2e r4 (claude/ci-e2e3 @ 50e3a436, base d349a6da), opus, read-only, 11-15

Scope: 9 commits, tests and ci.yaml only; no product code changed. Red-first check passes locally.
r3 items: F1 closed (gate-0.7-6.sh:15-18 fails under CI; GitHub sets CI=true, also inside `container:` jobs).
F3 closed for the local sequence: a script that prints `SKIP:` fails unless allowlisted
(e2e-sequence.sh:55-61), and the summary names every skip. F2 closed: the font step is a one-line run
(ci.yaml:35), so the sequence runs it and gates 0.8-9/0.8-10. F4-F6 closed: the camera refusal is asserted
against the real core text (mcp-client.py:278-282 vs ipc/clients.rs:326), the panel waits for
animation.done, the variable is set only on the chora command (agent.sh:69), and the temp dir is removed on pass (e2e-sequence.sh:75).
Security: the free client runs on the main socket under the user's own name, as the user could. No
confined path gained `screen:*` or camera access (clients.rs:326 and the unit test in ipc.rs:2614 are unchanged).

| id | sev | file:line | finding |
|---|---|---|---|
| F1 | MAJOR | tests/e2e-sequence.sh:11,59; .github/workflows/ci.yaml:16-65 | The claim "with CI set it is ignored, so that no skip passes in CI" is false. CI never runs e2e-sequence.sh (no reference in .github/); each e2e step calls its script directly, and gate-lib.sh:13-15 `skip` exits 0. Scenario: a Fedora update (dnf installs current packages) moves xdg-desktop-portal-wlr or drops a `need` tool. gate-0.9-screencast/x11/0.7-8/0.8-4/xp-token/session-units then print SKIP, and the CI job is green. This is the class of friction 47, and in CI only gate-0.7-6 is fixed. Fix: run `tests/e2e-sequence.sh target/debug e2e` as the job's test step, or make `skip` in gate-lib.sh fail when $CI is set (with the allowlist rule), plus a gate-lib-selftest case. Earliest gate: gate-lib-selftest.sh (a skip under CI=1 must exit non-zero). |
| F2 | MAJOR | tests/gate-0.8-10.sh:2-3,36; ci.yaml:50; docs/roadmap.md:236 | Gate 0.8-10 still says "an MCP client puts a text node under screen:NAME", but the node now comes from a free client (`free_apply`). The confined rule (docs/api.md:1075) forbids what the roadmap gate asks, so the gate now passes without checking what it names. Scenario: the 0.8 release list ticks 0.8-10 as met. Fix: escalate the conflict to the PM. Then restate the gate (roadmap line, script header, CI step name) as what it checks, and add an assertion that the confined create under screen:HEADLESS-1 is refused, as panel() does for the camera. Earliest gate: review against docs/roadmap.md, because no automated check compares a gate header with its roadmap line. |
| F3 | MINOR | tests/e2e-sequence.sh:36-37 | If the job is renamed, or every step becomes a multi-line `run: \|`, `commands` is empty. The sequence then prints "passed" with 0 scripts. A multi-line step that runs tests/ is dropped without a message. Fail when number=0, and warn on a `run: \|` in the e2e job. |
| F4 | MINOR | tests/e2e-sequence.sh:59 | The allowlist match is `^$name\|`. An entry with an empty reason is accepted, against the "with a reason" claim, and `.` in the name is a regex wildcard. Use `grep -qF` on the prefix and check that the reason is not empty. |
| F5 | MINOR | tests/e2e-fonts.sh:3-4,95 | The comment says the text checks then skip. They do not, because the directory always exists and gate-0.8-9.sh:16 tests only `-d`. In CI a missing font path now passes without a message, where the old `ln -s` failed the step. Fail on a missing font when $CI is set, and correct the comment. |
| F6 | COSMETIC | tests/merge-gate.sh:24 | "a script whose tool is missing here skips" is stale: an unlisted skip now fails the step. |
| F7 | COSMETIC | tests/gate_mcp.py:81; tests/mcp-client.py:196 | `free_apply` is duplicated. |
| F8 | COSMETIC | commit subjects | The commits change only test and CI code but use fix(test)/feat(test) with `No test:`. The repo mostly uses `test:`/`ci:` (123 of the last 400 subjects), and the brief allows ci/test. Red-first passes, so this is a style point only. All subjects are 68 characters or fewer, and all body lines are 72 or fewer. |

Not verified here: the 3/3 sequence runs and the full merge gate (the binaries are not built in this worktree).
This container has system CJK and emoji fonts (wqy-zenhei, Noto Color Emoji). Gates 0.8-9 and 0.8-10 can
pass from those through the system fonts of text.rs:273, so the passes are plausible.

Verdict: FIX (F1, F2 MAJOR)
