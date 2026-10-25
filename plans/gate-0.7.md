# Gate of 0.7, checks 4 to 10 (run 2026-10-25, commit e898140)

Each gate check of roadmap section 0.7 from 4 to 10 has one script,
`tests/gate-0.7-<n>.sh`. A script answers `ok:` or `SKIP: <why>` with
exit 0, and `FAIL: <why>` with exit 1. Each starts its own headless
session and cleans it up. Run all of them with:

```bash
for n in 4 5 6 7 8 9 10; do tests/gate-0.7-$n.sh || echo "gate $n failed"; done
```

Checks 6 and 10 need the binaries in `target/debug` (`BIN` overrides).
Check 9 runs unit tests through `micromamba run -n chora cargo test`.
Check 7 answers FAIL until T14 removes program start from the core; the
plan of the night says that is expected.

| check | result | note |
|---|---|---|
| 4 Podman container with only `clients/agent/` | SKIP | no podman here; the CI job `podman` in `.github/workflows/ci.yaml` runs the script on an ubuntu runner, where podman is preinstalled; the script queries, draws a green rect seen in a capture, and fails `grants.set` and a `hello` with another name |
| 5 Quadlet unit survives a chora restart | SKIP | no podman and no answering user systemd here; on a host with both, plus `chora@.service` and the agent image, the script restarts chora and lets the untouched unit connect again |
| 6 `input.grab-any` grabs F13 and a letter | ok | guest without the capability gets code 4; agent grabs F13 and `a`, injects both, and its connection sees both presses as `input.key`; the script sets `XKB_DEFAULT_OPTIONS=fkeys:basic_13-24`, because the stock pc105 keymap binds no key between F1 and F12 |
| 7 no fork in the core except the startup command | FAIL | expected until T14: the core still serves `process.spawn`, `process.wait` and `plugins.restart`; the script lists the sites |
| 8 `choractl run foot` with its scope and token | SKIP | foot is not installed here; the script checks the pid, the window, the `.scope` in the cgroup when the user manager answers, and the token in `surface.created` |
| 9 every error code has a test | ok | the eleven rows of the Error codes table in `docs/api.md`, each with the green test that provokes it; a table row without a test in the script fails |
| 10 battle workflows without pixel clicks | ok | no workflow but W3 uses `pointer_move` or a click; W1 and W2 ran headless and left no new findings in `tests/battle/findings.md`; the full round W1 to W8 stays with the battle driver |

What would turn the SKIP rows green here: podman (checks 4 and 5), a
running user systemd plus the installed units and the agent image
(check 5), and foot (check 8). Check 7 turns green with T14.
