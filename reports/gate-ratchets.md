# gate-ratchets

## r1

Branch claude/gate-ratchets-r1, head 0094bffd (base 721be592).

- Red 3bfcb52d (tests/gate-ratchets-selftest.sh): "unknown step ratchets" on the clean tree and with the added wait lines (no line names sleep-ratchet).
- Fix 39566610: step `ratchets` after `identity` (sleep-ratchet, size-ratchet, lint-ratchet; FAIL line `FAIL: <ratchet>: <count line>`), also in --worker mode; header, --list and CONTRIBUTING.md name it.
- d53aae56: the self-test adds lines past the slack of the list (tree 778, list 779, so one line only reaches the list). 0094bffd: tests/merge-gate-selftest.sh counts the step in --worker.
- Self-test: `gate-ratchets-selftest: ok`. `--list` shows `ratchets` third.
- Full gate (--jobs 2): all steps PASS, EXIT 0; `pgrep -fa 'chora --headless'` empty. One earlier run was cut by a worker restart and rerun.

Last 20 lines of /tmp/gate.log:

```
voice-acp  PASS    13s /tmp/merge-gate.Uk2wCy/voice-acp.log
voice-events PASS     3s /tmp/merge-gate.Uk2wCy/voice-events.log
agent      PASS    43s /tmp/merge-gate.Uk2wCy/agent.log
tiling-keys-more PASS    43s /tmp/merge-gate.Uk2wCy/tiling-keys-more.log
tiling-windows PASS   224s /tmp/merge-gate.Uk2wCy/tiling-windows.log
tiling-system PASS    17s /tmp/merge-gate.Uk2wCy/tiling-system.log
handover   PASS    30s /tmp/merge-gate.Uk2wCy/handover.log
mc-scale   PASS    19s /tmp/merge-gate.Uk2wCy/mc-scale.log
watchdog-starved PASS    55s /tmp/merge-gate.Uk2wCy/watchdog-starved.log
acts       PASS  2446s /tmp/merge-gate.Uk2wCy/acts.log
xp-run     PASS    15s /tmp/merge-gate.Uk2wCy/xp-run.log
demo       PASS   296s /tmp/merge-gate.Uk2wCy/demo.log
leak       PASS   487s /tmp/merge-gate.Uk2wCy/leak.log
tiling-theme PASS    19s /tmp/merge-gate.Uk2wCy/tiling-theme.log
tiling-disc PASS    81s /tmp/merge-gate.Uk2wCy/tiling-disc.log
mc-host    PASS    23s /tmp/merge-gate.Uk2wCy/mc-host.log
tiling-bar PASS   211s /tmp/merge-gate.Uk2wCy/tiling-bar.log
e2e        PASS   293s /tmp/merge-gate.Uk2wCy/e2e.log
all steps passed; no --push, nothing is pushed
EXIT 0
```
