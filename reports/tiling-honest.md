# tiling-honest

## r1

Lane tiling-honest r1 (sonnet, high, rc 1), claude/tiling-honest-r1 from bb4c07d9. **Head b02dbea9**, pushed.
Closes ti-52 (rpt1-1) and ti-53 (rpt1-2). Out of scope and untouched: rpt1-3..rpt1-10 (ti-54).

### Commits
- f068941a red (tests only, cites ti-52 and ti-53):
  - `tests/doors/tiling.sh` cell helper-door: after the click opens the chat it waits for the field to hold the
    keyboard, types `hallo` and Return, and wants the text in the log of the helper `coder` and no key in the probe window.
  - `tests/gate-tiling-reload-notice.sh` (new, step tiling-system): a valid reload (gap_inner 30), then a broken line
    `  gap_inner 20` (no colon) and a wrong action `MOD+Shift+y: frobnicate`, each with MOD+Ctrl+r; it wants a notice of
    chora-notify that names `chora-tiling.yaml` and the reason, a card on the screen, and gap_inner still 30.
  - Red on bb4c07d9:
    - `tiling: the click gave the chat field no keyboard: chat open yes, window enter, focused ["s1"]` and `cell tiling helper-door missing`
    - `ok: a valid file reloads: gap_inner 30`, then `FAIL: line: no notice names the file: []` (notify `shown: []`)
- 46760cf7 fix ti-52 and ti-53:
  - ti-52: the bar word called `choractl run -- chora-chat`, a new connection that holds no act, so its token was
    unstamped (ipc.rs:996-1011) and the chat field never took the keyboard. Now `on-click` is
    `choractl call tiling show_helper`: tiling starts the chat with its own token, the same call as MOD+a
    (`Host::run`). One core change was needed for that token to carry an act: `input.rs press_layer`. A real press on a
    layer surface (the bar) is noted as an act of the manager whose claim holds the point, else of the first manager (the bar
    lies outside the tiling claim, measured: with the claim only the cell stayed red). `ipc.rs` is unchanged. The method
    `show_helper` is privileged (a confined caller cannot call it) and is in docs/configuration.md.
  - ti-53: `plugins/chora-tiling/src/reload.rs`. On a failed MOD+Ctrl+r tiling shows one notice of chora-notify
    (the existing notify path, no new message type): "Settings not applied", the file by name, the parser's line and
    cause cut short (the "expected one of" list goes), "The old settings stay."; 12 s; the notice of an earlier failure is
    dismissed first (a `replaces` of a dismissed id closed the new notice in the same batch, found in the gate). German
    text under a German session. The settings were never changed by a failed reload (`undo`); the gate checks it.
- b02dbea9: cut-check forbids chora-tiling naming chora-chat: `show_helper` starts `chora_kit::launcher::HELPER_PROGRAM`
  (the program of the Apps row, now `pub`).
- Limits: a wrong value (a wrong action name) has no line number in the parser's text (settings.rs comment), so that
  notice names the file and the cause only; a line comes where the YAML parser has one. The card is chora-notify's
  neutral light card (rpt1-4, ti-54, not touched).

### Repros by hand on b02dbea9 (fresh `tests/persona/session.sh tiling 1280x720`, `--test-injected-press-is-real`)
- rpt1-1: `key Super+Return`, `click 932 24` (the bar word, capture at scale 1), `type "echo LEAKED-FIX"`: the chat opens
  beside the tile, the text stands in the chat field with its caret, foot shows only its prompt (`clear` before). Capture
  `media/2026-11-24/tiling-honest/rpt1-1-helper-click-keys.png`. (A first try with the capture at scale 0.625 missed the
  word and the text went to foot: a harness slip, not the product.)
- rpt1-2: `chora-tiling --print-config`, `  gap_inner 20` under `radius:`, Super+Ctrl+r, with `choractl run -- chora-notify`:
  `notify state.get` `shown`: `[{"summary":"Settings not applied","body":"chora-tiling.yaml: simple key expected ':' at line 24, column 3\nThe old settings stay.", ...}]`,
  the card stands at the bottom right over the tile. Capture `media/2026-11-24/tiling-honest/rpt1-2-reload-notice.png`.

### Gate
- Full run on b02dbea9 (`tests/merge-gate.sh --jobs 2`, no --only): all 65 steps PASS, `EXIT 0`, no foreign red
  (run directory /tmp/merge-gate.WNpIWk). An earlier full run on 46760cf7 stopped at ci-check:
  `FAIL: cut-check: names: chora-tiling names chora-chat on 1 lines, first plugins/chora-tiling/src/main.rs:890`
  (mine; fixed in b02dbea9).
- Last lines of /tmp/gate.log:
```
e2e        PASS   264s
tiling-bar PASS   156s
mc-host    PASS    17s
tiling-disc PASS   69s
tiling-theme PASS    18s
leak       PASS   309s
demo       PASS   200s
xp-run     PASS    11s
tiling-windows PASS   153s
watchdog-starved PASS   50s
mc-scale   PASS    11s
handover   PASS    23s
tiling-system PASS    13s
tiling-keys-more PASS    40s
agent      PASS    36s
voice-events PASS     1s
voice-acp  PASS    11s
mc-column  PASS    46s
launcher-click PASS    17s
all steps passed; no --push, nothing is pushed
EXIT 0
```
- Kept: tour tiling `28 acts, 28 green, 0 red`; tour handover `11 acts, 11 green, 0 red`; `cargo test -p chora-tiling` 197
  passed; `tools/sleep-ratchet` ok (sleep 778, sleep-all 929: below the list, no new sleep line); no `chora --headless` left.

## Review r1

Seeded review (opus, high) of claude/tiling-honest-r1 head b02dbea9 against base bb4c07d9 (11 files, +216 -4). One round, read-only on code.
Step 0: `git fetch --unshallow` and `cargo check --workspace` both OK. `cargo test -p chora input`: 85 passed. `cargo test -p chora-tiling`: 197 passed. No merge gate was run (the lane's run was green, 65 steps).

**Verdict: MERGE-OK** (0 BLOCK, 0 MAJOR, 6 MINOR, 2 NIT).

Checks:
1. Red first: yes. f068941a (tests only) comes before 46760cf7. b02dbea9 is a refactor: it makes the existing `HELPER_PROGRAM` constant `pub`, with the same string. The cut-check red on 46760cf7 covers it, so it is not a fix that lacks a red test.
2. Core `press_layer`:
   - The escape hatch is untouched (`grab_button` returns first).
   - `note_act` still refuses injected presses and a locked session.
   - A press noted by `press_node` keeps its act, because the same serial is not overwritten.
   - No key goes to a hovered client (D55).
   - The act goes only to manager connections, never to the layer's client.
   - Open points: rth-1, rth-2, rth-3.
3. ti-52: the bar click calls `show_helper`, which uses `Host::run` (the same path as `MOD+a {run: chora-chat}`). The change to `waybar/config.jsonc` is one line. The door cell types `hallo` and checks that it reached the helper and not the probe.
4. ti-53:
   - The old settings stay (`undo`; the gate checks gap_inner 30).
   - The notice is plain: "Settings not applied … The old settings stay." It names the file and the parser's line.
   - Open points: rth-4, rth-5, rth-6.
5. The gates poll with `until_ok` and add no new `sleep` line. The `merge-gate.sh` change is only the cell list. The `docs/configuration.md` row is mostly true (rth-7).

Findings:
- rth-1 | MINOR | crates/chora/src/input.rs:1037 | Outside every claim, the act of a layer press goes to the oldest manager (`first_manager`), not to the look whose bar was clicked. In a two-look session where tiling joined second (D119, for example xp first), a click on the tiling helper word gives the act to xp. Tiling's token is then unstamped and the chat opens without the keys (ti-52 again), while xp holds an act for a click it never took. | Fix: give the act to the manager whose claim lies on the output of the layer surface (else to every live manager), and test it in a two-manager unit test.
- rth-2 | MINOR | docs/api.md:941, docs/architecture.md:441, crates/chora/src/state.rs:1070 | A press on any layer surface (a clock, a notice card, fuzzel) is now an act of a manager. The three places that define an act still list only a grabbed key or button, a key to a node, and a press on a node. | Fix: add "a real press on a layer surface, to the manager of the claim at the point" to all three.
- rth-3 | MINOR | crates/chora/src/input.rs:1025 | No core unit test covers `press_layer`. Only the doors cell does, and it runs with `--test-injected-press-is-real`. Untested cases: an injected press gives no act, a locked session gives none, and a layer outside every claim goes to the right manager. | Fix: add one test in the core input tests for these three cases.
- rth-4 | MINOR | plugins/chora-tiling/src/reload.rs:62 | Without chora-notify, a failed reload is still silent on screen (stderr only). The real flavor starts it (chora-tiling.target Wants chora-tiling-notify.service), and rc1-pass p2 and p3 start it. The persona tiling session does not (tests/persona/session.sh:245-250, reports/tiling-doors-plan.md:13), so a persona still sees nothing. | Fix: when `show_or_log` answers None, show the short reason in tiling's own chip (`show_chip`), or start chora-notify as a tiling companion in session.sh.
- rth-5 | MINOR | plugins/chora-tiling/src/main.rs:1137 | A good reload after a failed one leaves the "Settings not applied" card up for up to 12 s, next to settings that did apply. `reload_notice` is dismissed only by the next failure. | Fix: on `Ok` of `reload_settings`, dismiss `reload_notice`, and add a gate step that checks it.
- rth-6 | MINOR | plugins/chora-tiling/src/reload.rs:17 | A wrong action name gets a notice with the file and the cause but no line, because the parser gives none. Done criterion (b) wanted the file and the line. The YAML error text ("simple key expected ':' at line 24, column 3") is parser wording, not plain words. | Fix: later, carry the line of the bad key from the YAML location into the cause, and say it as "line 24: a colon is missing after the name".
- rth-7 | NIT | docs/configuration.md:365 | "`MOD+a`: opens the chat…" reads as if `MOD+a` calls `show_helper`. In fact `MOD+a` is `{run: chora-chat}` (tiling.yaml:168), and a rebound `MOD+a` leaves the bar on chora-chat. | Fix: write "as `MOD+a` (`run: chora-chat`) does: opens …".
- rth-8 | NIT | tests/gate-tiling-reload-notice.sh:29 | `card_shows` compares the pixel at the card centre with a pixel taken earlier at (1000,700), a different point. The check holds only on a flat background. | Fix: take the earlier pixel at the same point as the card centre (capture before the reload, read it after `rect` is known).
