# security a r1: the MEDIUM security findings of review-plugins-r1 (mc, construct, assistant, notify)

Branch `claude/security-a`, from the session branch at 61d885f0, merged
again with it at 0e303d60. All seven findings still held on the newest
session branch (the code had moved since 498d9de; the line numbers below
are the ones on 61d885f0). Each got a red test commit, then the fix.

| id | holds | red commit | fix commit | earliest gate |
|---|---|---|---|---|
| MC-02 | yes: `content.set`/`content.add` (main.rs:1022-1039) wrote any text content, `confirm` too | 329fafe8 | 16b3d668 | unit test (`cargo test`) |
| MC-03 | yes: `notify`/`urgent` writable as MC-02; `hud.set`/`hud.say` (main.rs:656-676) unprivileged | 329fafe8, f1706e45 | 16b3d668, f07e1387 | unit test, the pinned list of privileged methods |
| MC-04 | yes: `press` (main.rs:1131) gave `injected` only to `click` | 4d2902df | 5e36eb46 | unit test |
| CON-A-04 | yes: `ends` (hud.rs:218) one candidate per character, no cap | a25322e9 | aeee5f78 | unit test |
| CON-B-10 | yes: the list arm (input.rs:616) took any button and injected presses | a05419be (after refactor ac69b3a5) | 907d2b53 | unit test |
| AS-B-05 | yes: `BufReader::lines()` and an unbounded `channel()` (field.rs:83) | cb101924 | c52fa46f | unit test |
| NO-03 | yes: unconfined `load_raster`/`read_tree` (chora-draw theme.rs:877-900) check nothing outside a HUD folder | a4edef7c (after refactor 5aa80312) | be4f6bac | unit test |

## Fixes

- MC-02, MC-03: `content.set` and `content.add` refuse `notify`, `urgent` and
  `confirm` (`agent::OWN_TEXTS`), which only `notify.push` and `voice.confirm`
  write. `hud.set` and `hud.say` are privileged, as `agent.set` already was.
- MC-04: `press` runs `click_agent` and `click_hit` only for a press of the
  person, or with `test_accept_injected_keys` (or the test environment
  variable), as the bindings do. `click_mic` and `click` already checked.
- CON-A-04: `ends` starts at most `END_CHARS` (3 x `LINE_CHARS` = 360)
  characters before the end, after an ellipsis: at most 361 candidates and
  probes per text.
- CON-B-10: `pressed_row(event, accepts_injected)` chooses a row only for a
  left press of the person (or in a test session); the list still takes
  every press on it.
- AS-B-05: the host drops a line of the field over 64 KiB and reads past it;
  the channel is `sync_channel(64)`, so a flooding field waits on its pipe.
- NO-03: `Picture::foreign` marks the notification icon; such a file is read
  at most `hud::MAX_FILE` (1 MiB), a PNG/JPEG at most `MAX_SIDE` (2048) on a
  side, an SVG through `check_tree` with pictures only from a data URI
  (`assets::data_resolver`), and its tree is not cached. A refused icon
  leaves its place on the card empty (docs/configuration.md).

Two `refactor` commits come before a red test so that the test drives the
real code without a session: construct has no in-process harness, and the
draw layer needed the flag to name a foreign file.

## Earliest gate and the class

Every finding is caught by a unit test of the changed function; no gate
script would have caught MC-04 or CON-B-10, since every gate session sets
`CHORA_TEST_ACCEPT_INJECTED_KEYS=1` and so accepts injected input. No cheap
compile-time gate closes a whole class here: privilege is per method and
not per parameter (MC-02/03), the `injected` flag is a field of a shared
IPC event that each handler destructures (MC-04, CON-B-10), and the trust
of a file path is a property of its origin, not of its type (NO-03). Not
added.

## Gate

- `tests/merge-gate.sh` (full, serial) on 443a1570: all 28 steps passed
  (build 176 s, test 86 s, e2e 273 s); pushed to `claude/security-a`.
- `tests/ci-local.sh`: msrv, docs, pre-commit, deny, machete, packaging
  passed, no job skipped.

## Self-check

1. State: no new flag, grab or timer; the relay thread of the field ends
   when the host drops the receiver or the field's output ends.
2. Skips: the e2e step skipped the five scripts that tests/e2e-skips.txt
   lists (gate-0.7-6 fkeys, gate-0.8-xp-token thunar, gate-0.9-screencast
   pipewire, gate-0.9-session-units desktop-file-validate, gate-0.9-x11
   xwayland-satellite: tools missing here); none touches this change.
3. Commit types: test, fix, refactor, docs; each fix after its red test.
4. CHANGELOG untouched.
5. Docs: configuration.md (content.set/add, hud.set/say, mc
   `test_accept_injected_keys`, the notification icon) in the same branch.
6. CI-only checks: tests/ci-local.sh above.
