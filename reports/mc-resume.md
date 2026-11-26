# mc-resume

## r1

Brief: briefs/mc-resume-r1.md (rfs2-1 MINOR, rmm5-2 NIT). Lane: cloud, sonnet.

- Base: 3e3b1b2d, then the merge of the mc-minor r5 head 1ab2e6a7 (fcb20c37; `cargo check` and `cargo build --workspace` green). Head: 329a13e5 on claude/mc-resume-r1.
- Commits: 25a58d97 red (gate case in tests/gate-mc-host.sh), 329a13e5 fix (host, unit test, wheel test).
- Red on the base, the log line of the gate case: `FAIL: light: the host that started again shows the old config: figure [1206.0,34.0,16.0,16.0], was [1206.0,34.0,16.0,16.0] (assistant: command 0: node "hud" exists already (code 1))`. The host started again, its create of the group `hud` failed on the node the core had kept, and the old figure stayed.
- Cause: the kit connects with `resume`, so within the 10 s grace of the core the restarted host gets its old nodes back; the host then created its figure again and the core refused it. Keeping the nodes is the design of the core.
- Fix (plugins/chora-assistant/src/main.rs): on start the host asks for `owner:assistant` and removes the roots of the nodes it got back (`drop_kept_nodes`, called from `ready`), then builds its look from the config it has now. No option, no timer, no kit helper. Unit test `a_start_removes_the_nodes_the_core_kept_for_the_old_config` on the fake core.
- rmm5-2: `wheel_steps` takes the `Instant` of the event; the two wheel tests pass one (a gap of 600 ms is `start + 600 ms`), no sleep. acts::tests_moved 19 of 19.
- Gate case: state panel `margin: [56, 34]` to `[56, 134]` in the hud.yaml, kill the host, start it again; within 5 s hud-figure is 100 px lower, and the nodes of the host are the nodes before it. Node count: 6 before and 6 after; `hud-top`, the empty group that a card or the badge leaves, is not made again and is left out of the comparison (7 before it was dropped).
- Node value before and after the restart (light, 1280x800): hud-figure [1206,34,16,16], hud-word [1120,29,80,26], 6 nodes; after: hud-figure [1206,134,16,16], hud-word [1115,129,85,26], 6 nodes.
- Captures: media/2026-11-26/mc-resume-r1/before-restart.png (96 KB), after-restart.png (96 KB). The chip stands over the helper line at the moved place: that is the test value, not a look.
- Diffstat (merge to head): plugins/chora-assistant/src/main.rs +58 -; plugins/chora-mc/src/acts.rs 5; plugins/chora-mc/src/acts/tests_moved.rs 38; tests/gate-mc-host.sh +23; 4 files, 108 insertions, 16 deletions.
- Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`, no step filter, EXIT 0 (docs ran PASS here). mc-host PASS 24 s. Last 20 lines:

```
launcher-mc PASS    40s /tmp/merge-gate.5m7j5j/launcher-mc.log
voice-acp  PASS    13s /tmp/merge-gate.5m7j5j/voice-acp.log
voice-events PASS     2s /tmp/merge-gate.5m7j5j/voice-events.log
agent      PASS    42s /tmp/merge-gate.5m7j5j/agent.log
tiling-keys-more PASS    42s /tmp/merge-gate.5m7j5j/tiling-keys-more.log
acts       PASS  2149s /tmp/merge-gate.5m7j5j/acts.log
tiling-system PASS    16s /tmp/merge-gate.5m7j5j/tiling-system.log
handover   PASS    28s /tmp/merge-gate.5m7j5j/handover.log
mc-scale   PASS    14s /tmp/merge-gate.5m7j5j/mc-scale.log
watchdog-starved PASS    56s /tmp/merge-gate.5m7j5j/watchdog-starved.log
tiling-windows PASS   204s /tmp/merge-gate.5m7j5j/tiling-windows.log
xp-run     PASS    14s /tmp/merge-gate.5m7j5j/xp-run.log
demo       PASS   281s /tmp/merge-gate.5m7j5j/demo.log
leak       PASS   484s /tmp/merge-gate.5m7j5j/leak.log
tiling-theme PASS    19s /tmp/merge-gate.5m7j5j/tiling-theme.log
tiling-bar PASS   195s /tmp/merge-gate.5m7j5j/tiling-bar.log
mc-host    PASS    24s /tmp/merge-gate.5m7j5j/mc-host.log
tiling-disc PASS    75s /tmp/merge-gate.5m7j5j/tiling-disc.log
e2e        PASS   288s /tmp/merge-gate.5m7j5j/e2e.log
all steps passed; no --push, nothing is pushed
```

## Review r1 (PM, 108 lines, under 200: PM review allowed)
Base fcb20c37 (merge of 1ab2e6a7), head 329a13e5; red 25a58d97 (gate-mc-host restart case) before fix 329a13e5.
- Done-when: red before fix (yes); captures 2 x 96 KB (yes); full gate EXIT 0, docs ran, mc-host PASS 24 s (yes); report r1 (yes).
- drop_kept_nodes removes only roots of `owner:assistant` (children go with them, unit test); on a fresh start the query is empty and the apply is a no-op. No option, timer or kit copy: KISS holds.
- rmm5-2: Instant passed in, the 600 ms sleep is gone; test-only change of a pure function, no red needed.
- rmr1-1 NIT: the empty `hud-top` group is not made again after a restart; a fresh start does not make it either, so restart equals a fresh start. Accept.
Verdict (computed): no open MAJOR: MERGE-OK 329a13e5, joins train 152 with xp-busy 88951ae4.
