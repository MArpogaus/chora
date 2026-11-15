# security a r3 (rc 6): close the injected-input class in every look

Branch `claude/security-a` at 85476db0 (session branch merged first, b8de5925).
Each item has its red commit, then the fix.

| item | red | fix | earliest gate |
|---|---|---|---|
| 1 gate | tests/injected-optout.py red on the tree: 33 lines (98941c64, output kept) | wired in pre-commit 15289128, green | pre-commit and guards.sh (17 red/green cases), whole-file scan |
| 2 MAJOR 1 walk arm and cube keys | be97c369 `an_injected_key_does_not_drive_the_walk` | 1fbd6fc9 `walk_key` | unit test; compile time via the gate |
| 3 MINOR 4 mc page_key | c1b1f2ea | dfef5333 | unit test |
| 3 MINOR 5 xp (press, release, Menu key) | 05a2f49b `an_injected_press_or_key_does_nothing_in_xp` | 78d79db7 `hears` | unit test; e2e cell |
| 3 MINOR 5 tiling press | 05a2f49b (`..._in_tiling`) | be39d1bd | unit test; e2e cell |
| 4 MINOR 6 chora-kit | failed under CHORA_TEST_ACCEPT_INJECTED_KEYS=1 (kit-red.txt) | 78c64ca6 pin | unit test under the gate env |
| 4 COSMETIC 7 input.rs:636 | the gate | cf7e5e9c (list through `walk_key`, pattern carries a reason) | the gate |
| 5 MINOR 2, 3 gate script | | 622d3724 | e2e gate |
| found by the class sweep | 6bc2dda0 `an_injected_pointer_does_not_drive_the_look` | cf7e5e9c, 2f147e42 | unit test, the gate |

## What the sweep found beyond the review

- construct: an injected press on the look took the keyboard (`FocusNode`),
  and with the modifier moved a pane; its moves looked around. An injected
  press on the look also closed the open list. `look_pointer` covers press,
  move and release.
- construct help: took every injected key; it now uses `walk_key`.
- chora-notify: an injected press on a card ran its action.
- `chora_kit::runtime::from_person` is the one test of the flag; a kit test
  reads event.rs and fails when a variant with `injected` is missing there.

## Gate (tests/injected-optout.py)

Reads the events with `injected: bool` from crates/chora-scene/src/event.rs.
Refuses `injected: _` and a rest pattern `..` in any pattern on them unless
the line (or the line after it, where rustfmt puts a comment behind a brace)
has `// injected: <reason>`. Deviation from the brief: a pattern that names
`injected` (`injected,` or `injected: true`) may end in `..`, since the field
is decided there. The comment check allows `// injected:`.

## Deviations and open points

- No method opens the construct window list, and I added none (API surface).
  The gate opens it with an injected Tab while the switch is on, then
  `settings.set test_accept_injected_keys false`.
- tiling cell sets `focus: click`: with focus-follows-mouse the injected
  pointer move alone focuses a window (not an input flag; open design point).
- The xp and tiling e2e cells were not run red on an old build (budget); their
  unit tests were run red. The list cells and mc cell were red in r2.
- Open: `PointerMoved` and `PointerReleased` of mc and assistant carry no
  decision beyond their press (annotated or named); assistant shows a card
  that says an injected press does not count, by design.

## Gate runs

- `tests/merge-gate.sh` (full, serial) on 85476db0: all steps passed. Skipped
  inside e2e (tests/e2e-skips.txt, tools missing here): gate-0.9-session-units,
  gate-0.7-6, gate-0.9-screencast, gate-0.9-x11, gate-0.8-xp-token.
- `tests/ci-local.sh`: msrv, docs, pre-commit, deny, machete, packaging passed.
- `cargo test --workspace` with the env unset and =1: no failures.

## Self-check

1. State: no new flag, grab or timer; `Drag.injected` removed.
2. Skips: the five above.
3. Commit types: test, fix, ci, docs; each fix after its red.
4. CHANGELOG untouched.
5. Docs: configuration.md lists what the switch covers.
6. CI-only checks: tests/ci-local.sh above.
