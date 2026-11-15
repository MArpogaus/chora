# security a r2: fix round of review-security-a-r1

Branch `claude/security-a` at 0a3d0040, merged with the session branch
first (8d963afc). Every item has a red commit, then the fix.

| item | red | fix | earliest gate |
|---|---|---|---|
| 1 MAJOR 1: injected keys in the window list | ab2fcaf9 (after refactor 7c0b374d) | 91640e75 | compile time for the field, unit test `an_injected_key_does_not_drive_the_window_list`; e2e cell of gate-injected-presses.sh failed on a build from before the fix |
| 2 MAJOR 2: `panel.set` on agents and overview | 9157d5ae | 2cfc4cb5 | unit test |
| 3 MINOR 3: `notify.clear` privileged | 71785fa7 (pinned list fails) | ac90a07b | the pinned list of privileged methods |
| 3 MINOR 4: `click` and injected | cf54d1b6 | c189764c | unit test |
| 3 MINOR 5: `inject_ok` in the harness | `CHORA_TEST_ACCEPT_INJECTED_KEYS=1 cargo test -p chora-mc` failed an_injected_press_on_a_hit_does_nothing (output kept) | dc980b09 (a test-harness commit, type `test`) | unit test under the gate environment |
| 4 MINOR 6: e2e cell, switch off | b4f71dae (new gate, red on the old build) | the same fixes; wired in by 65b0f6c8 | e2e gate |
| 5 COSMETIC 7, 8: doc line, docs | | 0a3d0040 | review only |

## What changed

- `NodeKey` has `injected` (set in `crates/chora/src/input.rs` from the
  source of the key; the release and the repeat carry it). Every handler
  in construct, mc, tiling and xp now destructures all fields of the
  event without `..`, so a new field fails to compile until each handler
  decides. clippy forbids `injected: _, ..`, hence the full patterns.
- The window list takes a key of the person, or of `input.inject` while
  `test_accept_injected_keys` or the test session allows it (`list_key`).
  An injected key on the look is taken and ignored while the list is open.
- `panel.set` refuses `agents` and `overview` (`agent::OWN_SCENES`).
- `notify.clear` is privileged; docs say so.
- `click` of mc returns for an injected press unless mc accepts injected
  input, so the overview lines and the windows are covered like the hits.
- `Runtime::pin_accepts_injected`; the mc harness pins it off.
- `tests/gate-injected-presses.sh`, in the matrix step of the merge gate:
  in a session with the switch off an injected click on the urgent line of
  mc, and an injected Return and an injected click on a row of the window
  list, do nothing; each has a control with the switch on that acts.
- docs/configuration.md: the Injected keys section, the construct
  `test_accept_injected_keys` row, `panel.set`, and the `set_text` doc line.

## Deviation and open point

- construct has no in-process harness (it builds its whole state in
  `play`), so the red unit test calls `list_key`, which `list_input` uses,
  not `list_input` itself. The e2e cell covers `list_input` end to end.
- Open, not in this round: the walk arm of the construct (`input.rs`,
  `action(&key, ...)`: terminal, close, windows, cube, walk keys) still
  takes injected keys with the switch off. `tests/gate-construct-3d.sh`
  walks with injected keys and the switch off, so closing it needs that
  gate changed first. An injected `close` key closes the pane ahead.

## Gate

- `tests/merge-gate.sh` (full, serial) on 0a3d0040: all 28 steps passed
  (build 35 s, test 107 s, matrix 270 s with the new gate, e2e 280 s).
  Skipped inside the e2e step (tests/e2e-skips.txt, tools missing here):
  gate-0.9-session-units (desktop-file-validate), gate-0.7-6 (fkeys),
  gate-0.9-screencast (pipewire), gate-0.9-x11 (xwayland-satellite),
  gate-0.8-xp-token (thunar). None touches this change.
- `tests/ci-local.sh`: msrv, docs, pre-commit, deny, machete, packaging
  passed; no job skipped.

## Self-check

1. State: no new flag, grab or timer.
2. Skips: the five above.
3. Commit types: refactor, test, fix, docs, ci; each fix after its red.
4. CHANGELOG untouched.
5. Docs: in the same branch.
6. CI-only checks: tests/ci-local.sh above.
