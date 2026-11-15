# Review ci e2e r5 (claude/ci-e2e3 @ ad5041d0, base 886cbb1e), opus, read-only, 11-15

Scope: 5 commits, all `test:`, tests/docs/ci.yaml only; no product code. Red-first verified by running the
selftest on extracted trees: 2bc70557 fails ("FAIL: a skip under CI exits 0", rc 1), 977db9cf and HEAD pass.

r4 items:
- MAJOR 1 closed. gate-lib.sh:14-21 `skip` prints `SKIP:` and, when `$CI` is non-empty, also `FAIL:` and
  exits 1. GitHub Actions sets CI=true for every step, including steps in `container:` jobs (the runner
  passes it to the container through docker exec -e). ci.yaml sets no `CI:` override. I grepped all 26
  `run: tests/...` scripts of the e2e job (ci.yaml:121-190): every skip goes through `skip` or `need`.
  No script has its own `exit 0` after a message, a `command -v x || exit 0`, or a skip after `|| true`.
  The `|| true` hits are on kill, grim or pgrep cleanup. The python skip paths (gate-0.8-3 rc 44, gate-0.8-6
  "rmcp (T6)") end in `skip` and now fail under CI, which is correct; rmcp is in Cargo.toml:75. The EXIT
  traps do not call `exit`, so they keep the status 1. The selftest case (gate-lib-selftest.sh:71-83) checks
  both directions and checks the reason text, so a lock failure cannot make it pass by accident.
- gate 0.7-6 (gate-0.7-6.sh:15): its behavior under CI is unchanged. It failed there before and fails
  through `skip` now. Fedora's xkeyboard-config has had `fkeys:basic_13-24` (in rules/evdev.lst, evdev.xml
  and evdev) since 2.42, and Fedora 44 ships a newer version, pulled in through libxkbcommon. I am fairly
  confident of this but have not checked the F44 package itself. If the option is missing, CI turns red
  because of the distro, not the product, which is the intended loud signal.
- MAJOR 2 closed. gate-0.8-10.sh:38-40 asserts that the confined `apply_commands` create under
  screen:HEADLESS-1 returns isError with "no screen:* parents". That is the real core text
  (ipc/clients.rs:324, unit test clients/tests.rs:1014), and gate_mcp.Mcp.call returns the raw result. The
  script header (lines 2-6), docs/roadmap.md:236-238 and the ci.yaml:170 step name now all say "a user client
  puts the node, the confined create is refused".
- F3 closed: e2e-sequence.sh:42-49 fails on an empty list and on `run: |` in the e2e job. The range ends at
  `  podman:`, and 26 of the job's 29 runs are tests/ (the other 3 are dnf, install and cargo).
- F4 closed: e2e-sequence.sh:36-38 uses awk `$1 == name && length($2) > 0`. I tested it with entries `a.b|`
  and `ab|`, and both are refused.
- F5 closed: e2e-fonts.sh:14-19 fails when CI is set and a font is missing, and the comment is correct.
- F6 closed (merge-gate.sh:24). F7 closed (one free_apply in gate_mcp.py:79; mcp-client.py adds its own dir to
  sys.path, which is needed under `python3 -I`). F8 closed (all subjects are `test:`).

| id | sev | file:line | finding |
|---|---|---|---|
| G1 | MINOR | ci.yaml:248,251-252; gate-0.9-vkms-hotplug.sh:96,137; gate-0.7-3.sh:15 | The vkms job runs its gates through `sudo`, and sudo's env_reset drops CI. Those skips ("needs root and the vkms module", "no vkms connector in sysfs") therefore still exit 0. Scenario: the runner's Azure kernel loses vkms, and the vkms job stays green. This is outside the e2e job that the brief scoped, but it is the same class of problem. Fix: `sudo CI="$CI" ...` or `sudo --preserve-env=CI`. |
| G2 | MINOR | e2e-sequence.sh:40-42 | Only `run: \|` is detected. A `run: >` or `run: >-` step, or one that runs `bash tests/x.sh` or `./tests/x.sh`, is still dropped from the local sequence without a message. Fix: warn on any `run:` in the e2e job that is neither a tests/ line nor one of the known setup commands. |
| G3 | MINOR | e2e-fonts.sh:12-13 | The Fedora paths of the two fonts (`google-noto-sans-cjk-vf-fonts/NotoSansCJK-VF.ttc`, `twemoji/Twemoji.ttf`) are not verified against F44. I believe both are right. If one is wrong, CI now fails at this step, loudly, but for a reason in the harness. Check the first CI run. |
| G4 | COSMETIC | plans/rest-0.7-0.8.md:216 | The plan still says "an MCP client puts a text node under screen:NAME". The roadmap is the authority, so this is a stale history line only. |
| G5 | COSMETIC | gate-lib.sh:3-5 | The header comment wraps oddly ("... (non-zero\n# when CI is set), or\n# ..."). |

Not verified here: the CI run on Fedora 44 and the 27-step merge gate. No binaries are built in this
worktree; the lane reports a pass on its second try after a known flake, words::tests::a_slow_backtrace.
Shell: shellcheck is not installed here. Reading the changed lines by hand found no quoting or status bugs.
`skip_status` passes `"$repo"` as `$1` correctly, and `sh -c "$command"` is unchanged.

Verdict: MERGE-OK
