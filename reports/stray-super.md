# stray-super

## r1

Lane stray-super-r1 (sonnet, high, cloud), branch `claude/stray-super-r1` from d40142e4, head `e1183766`.
Fixes rpx1-1 (xp-46) and rpx1-8 (mc-60): an unbound Super chord no longer types into a client.

### Commits

| sha | what |
|---|---|
| 664d2893 | red: unit tests (xp wm.rs, xp wm/tests_moved.rs, mc lifecycle/tests_moved.rs) and a gate step each (`tests/gate-xp-notepad.sh`, cell `stray-super` of `tests/gate-mc-paths.sh`) |
| 9d82b085 | fix: xp and mc call `grab_swallowed("Super", ..)` as tiling does (D118); mc gets the hidden `Look::Swallow` (`#[serde(skip)]`) |
| 50f78472 | gate steps wait with `until_ok`, not `sleep` (the first gate run was red at the sleep ratchet: 786 against 779) |
| e1183766 | fix: xp on a screen of its own swallows from its first window, not from start (see "Found on the way") |

### Red (664d2893)

```
test wm::tests::on_a_face_xp_swallows_the_super_chords_it_binds_nothing_for ... FAILED
xp-46: Super+x is free on a screen of xp, so it types into a window
test wm::tests_moved::keys::a_swallowed_chord_on_a_screen_of_xp_opens_no_start_menu ... FAILED
xp-46: the unbound chord is free on a screen of xp
test lifecycle::tests_moved::an_unbound_super_chord_is_grabbed_and_a_bound_one_keeps_its_action ... FAILED
mc-60: Super+x is free, so it types into a window page
FAIL: xp-46: the client typed 'start:x' and not start::end        (tests/gate-xp-notepad.sh)
FAIL: mc-60: the window page typed 'start:xkmq:end' and not start::end   (cell stray-super)
```

xp types `start:x` and the Start menu then takes the rest (the report's "Start opens on release");
mc types every chord letter, `xkmq`.

### Fix

- xp (`plugins/chora-xp/src/wm.rs`, `wm/windows.rs`): `grab_keys` swallows unbound Super chords on a face
  as before, and on a screen of its own once xp holds a window (`Chords::Swallowed`, set by
  `first_window` in `adopt`). The swallowed chord reaches `key()`, which marks the Super press as a
  combination, so the release of Super opens no Start (existing `tap` logic); `Super+Super_L` is not in
  the key list, so Super alone still opens Start.
- mc (`plugins/chora-mc/src/lifecycle.rs`, `config.rs`, `acts.rs`, `keys.rs`): `Look::Swallow`, grabbed by
  `grab_swallowed` at the end of `grab_all` (when the build is done) and after `grab_claimed` in `start`,
  so a claimed binding keeps a Super key of its own. The key list skips it.
- No new API, no core change.

### Found on the way: the swallow from the start broke the matrix cell xp-first

The first fix (9d82b085) swallowed in xp from the start. The full gate went red at
`matrix`, `gate-join-keys.sh` cell `xp-first`: `FAIL: xp-first: Super+Return opened no window`. Cause
(mine): xp starts first and held `Super+Return` as a swallow; tiling joins later, its grab of the
chord loses (the first grab wins, D118), so its terminal key is dead. Tiling-first is not hit, since tiling
holds the chords before xp. e1183766 swallows only once xp holds a window; before, the chords stay free
for a look that joins. Cost: a look that joins while xp already has a window gets none of the chords xp
swallowed (same as a joining xp under tiling today). The PM may prefer another rule (for instance a
release on join); this one needed no new API.

### Repros by hand on the fix (e1183766)

Fresh `CHORA_TEST_FLAGS=--test-injected-press-is-real tests/persona/session.sh`, `tests/persona/act.py`; a foot
that writes its raw input to a file is the focused client.

- xp 1280x800: Super, click Notepad (62 606), foot opened, `type start:`, `key Super+x`, `type "|"`,
  `key Super+k`, `type "|"`, `key Super+m`, `key Super+q`, `type :end`. File: `start:||:end`.
  Start menu `visible: false`; then `key Super_L` opens it (`true`).
  Before the fix (report): `abcxkmdef` and `start:x|m|:end` with Start open.
- mc 1280x720: foot `--app-id=mcstray` as a page, `key Return`, the same typing plus `Super+Shift+x`.
  File: `start:||:end`. Still acting by hand: Super+n hides the column (door count 1 -> 0), Super+F1
  shows the key list, Super+a opens the chat, Super+Space starts `fuzzel --dmenu`, `Right` turns the page
  (agenda -> morning), `XF86Back` goes back, `XF86HomePage` goes to agenda.

### Gate

`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` at e1183766: `all steps passed`, EXIT 0, 65 cells
(`xp`, `xp-run`, `mc-paths` with the new cell, `matrix` with `gate-join-keys` and `gate-start-tap`,
`tiling-windows` with `gate-tiling-swallow`, `acts`, `leak`, `e2e`). Last lines:

```
tiling-bar PASS   199s
mc-host    PASS    19s
tiling-disc PASS   70s
e2e        PASS   281s
all steps passed; no --push, nothing is pushed
EXIT 0
```

`tools/sleep-ratchet`: `ok: sleep-ratchet (sleep 779, sleep-all 930)`. `pgrep -fa 'chora --headless'`: empty.
Earlier runs: first at 50f78472~ red at ci-check (sleep 786), second at 9d82b085 red at matrix xp-first (above).

### Not done

No change to docs/decisions.md (D118 stays right; the xp rule above is a refinement the PM may want to
add to it). A joining look with an xp window already open still loses swallowed chords.

## Review r1

Reviewer lane review-stray-super-r1 (opus, high, cloud), read-only on `claude/stray-super-r1` e1183766, base d40142e4.
Step 0: `git fetch --unshallow` ok, `cargo check --workspace` ok; `cargo test -p chora-xp -p chora-mc`: 197 and 353 passed.
Verdict: **FIX** (0 BLOCK, 2 MAJOR, 3 MINOR, 1 NIT).

Checks:
1. Red first holds: 664d2893 touches tests only (unit tests, gate-xp-notepad step, mc-paths cell stray-super), before 9d82b085.
   The e1183766 rule has a test that was red before it: matrix `gate-join-keys.sh` xp-first (on d40142e4 since ti-38),
   red at 9d82b085 with the pasted line; the unit assert "no window, nothing to keep" is red on 9d82b085 too.
2. xp: Super alone opens Start, a swallowed chord marks the tap so the release opens nothing, swallow from `adopt`
   (unit test and gate step green). Before the first window no client of xp exists to type into. The swallow never ends (rss-3).
3. mc: Super+Space, Super+F1, Super+a, Super+n act (unit test, report repro); Right, XF86Back, XF86HomePage are no Super
   chords and the swallow list skips them; a claimed binding grabs before the swallow; `Look::Swallow` is `serde(skip)`, never
   in a config, `sheet` and `all` skip it. Ok.
4. Joining looks: judged MAJOR (rss-2). Grabs are global and exclusive (`crates/chora/src/ipc/grabs.rs:50`), so the swallow
   of one look takes the keys of any look that joins later, on every output. Against base it is a regression in a supported
   flow (D55, D119, the matrix): the person starts tiling beside xp with a window and tiling's focus and move keys do nothing,
   no POLS. D118 accepts "the first grab wins" for bound chords and counts the cost in grabs; it never weighed a joining look
   losing its own keys, and it names the manager of a face or pane, not xp on a screen of its own (rss-5).
5. No core change (diff: plugins/chora-xp, plugins/chora-mc, tests only); on a face xp swallows as before (`||` in
   `grab_keys`); the gate steps wait with `until_ok`, no new sleep line.

Repros on e1183766 (headless, scripts in the reviewer's scratchpad, not committed):
- mc `--manage-windows --output A` first, tiling `--output B` joins (the order of `gate-construct-windows.sh:477-479`):
  tiling log `grabbed already` 232 lines, among them `Super+1` .. `Super+8`; on d40142e4 23 (only mc's bound chords).
- xp with a foot window first, tiling joins on the same output: tiling 233 refused, among them `Super+h`, `Shift+Super+h`,
  `Ctrl+Super+h`, `Shift+Super+1`.

Findings:
- rss-1, MAJOR, plugins/chora-mc/src/lifecycle.rs:183, mc swallows every unbound Super chord from its start, also with `--output` beside another look, so a look that joins after mc loses them on its own output (repro: tiling Super+1..8 dead, 23 -> 232 refusals); this is the xp-first breakage that e1183766 fixed for xp, left in mc and untested; fix: give mc the xp rule (swallow once mc holds a window page) or the shared rule of rss-2, with a matrix cell mc-first plus tiling joining.
- rss-2, MAJOR, plugins/chora-xp/src/wm/windows.rs:40, once xp holds a window a joining tiling loses about 200 Super chords (Super+h, Shift+Super+1 ...), a regression against base in the join flow of D55/D119 with no test (the lane's stated cost); fix: one rule for the swallowing looks, for instance drop the swallow set when another window manager takes a claim and swallow again after its grabs, with a matrix cell xp-with-window plus tiling joining.
- rss-3, MINOR, plugins/chora-xp/src/wm/windows.rs:407, `Chords` never goes back to Free, so after the last window closed or was given away a joining look still loses the chords; fix: back to Free (and regrab) when xp holds no window, or let the rss-2 rule cover it.
- rss-4, MINOR, crates/chora-kit/src/runtime.rs:378, a swallow beside a look that already holds the chords logs one `grabbed already` line per chord (about 200 per start of mc after tiling or the construct, of xp beside tiling), new for mc and xp; fix: `grab_swallowed` takes a refusal quietly (chora-kit only, no core change).
- rss-5, MINOR, docs/decisions.md:167, D118 names the manager of a face or pane; swallow by xp on a screen of its own from its first window and by mc as a full look is a new rule without a decision row; fix: amend D118 with the rule and its join cost once rss-1/rss-2 settle it.
- rss-6, NIT, tests/gate-xp-notepad.sh:70, `menu_visible && fail` checks once right after the text arrived, while xp opens Start on its own handling of the release, so a late open passes; fix: poll a short while (`until_ok` on the menu staying closed for a few rounds) or check after a settle.

## r2

Lane stray-super-r2 (opus, high, cloud), branch `claude/stray-super-r2` from e1183766, head `1eb30643`.
Fixes rss-1, rss-2 (MAJOR), rss-3, rss-4, rss-6, rss-5 (doc); xp-46 and mc-60 stay fixed. No core change.

### Commits

| sha | what |
|---|---|
| 429ce20f | red: gate-join-keys cells `xp-window` and `mc-first` (each also types Super+x/k/m/q into the xp or mc client and wants `start::end`); chora-kit swallow unit tests for the yield, the retry and the quiet refusal |
| d88dee68 | fix: the kit swallow yields to a joining window manager and comes back when it went; a refused grab is asked again on `input.ungrabbed` (D117); a refused swallow logs nothing; tiling and mc subscribe to `input.ungrabbed` |
| 160b62fd | fix: revert of e1183766 (xp "from the first window", the `Chords` state of rss-3): the rule makes it needless, xp swallows from its start |
| 47aad40d | test: gate-xp-notepad polls that Start stays closed (`until_ok menu_visible 4 && fail`, rss-6); xp-window opens its second tile with Super+Return of tiling |
| 4455b2a3 | docs: D118 amended with the rule and its cost (rss-5) |
| 6a8bd3b9 | fix: `grab_swallowed` loops over no chord instead of an early return (gate run 1 red at ci-check, `tests/grab-gap-check.py`) |
| 1eb30643 | fix: the swallow grabs its chords in one batch, `call_all` (gate run 2 red at xp-notepad, see below) |

### The rule (kit only, `crates/chora-kit/src/swallow.rs`, `runtime.rs`)

- `runtime::dispatch` hands `client.connected`, `client.disconnected` and `input.ungrabbed` to `Runtime::swallow_event`
  (the existing events; no new core message). A window manager of `give::KEYS` (tiling, xp, mc, construct) that
  connects after the look: the look lets its swallow set go in one batch and swallows nothing while one is there
  (`grab_swallowed` records the wish only). When the last one went, the look swallows again.
- Every refused grab waits (`waiting`) and is asked again when the core says the binding is free (`input.ungrabbed`):
  so a joiner whose grab met the swallow before the swallow let go still gets the chord. A refused bound chord stays
  bound: the look's own swallow skips it (found by the mc-first cell: tiling's Super+4 went to its own swallow).
- So the last look that joined holds the unbound chords; the joiner gets every chord it binds that no other look
  binds (first grab wins for bound chords, as before). xp, mc, tiling and the construct share the code.
- `Plugin::Action` gets a `Clone` bound (every action already was).

### Red (429ce20f, on e1183766 binaries)

```
FAIL: xp-window: Super+h of tiling does nothing beside xp: the rects stay [[14.0,54.0,618.0,702.0],[648.0,54.0,618.0,702.0]]
xp-window: tiling refused 233 grabs
FAIL: mc-first: Super+2 of tiling does nothing beside mc: [{"output":"HEADLESS-2","shown":[1]}]
mc-first: tiling refused 232 grabs
swallow::tests::a_swallow_yields_to_a_window_manager_that_joins_and_comes_back_when_it_goes:
  rss-2: the swallow of xp holds Super+q against tiling, which joined
swallow::tests::a_refused_grab_is_asked_again_when_the_core_lets_it_go:
  rss-1: Super+1 of tiling stays dead after the swallow of mc let it go
swallow::tests::a_refused_swallow_logs_nothing:
  rss-4: chora-mc: Super+q: held by another client (code 1)
```

Both cells type `start::end` already at red: the stray part holds on r1, the join part does not.

### Review repros rerun (gate-join-keys logs, head)

- mc-first (mc `--manage-windows --output A`, tiling joins on B): tiling `grabbed already` 19 lines (base 23, r1 232),
  only chords mc binds (Super+F1, Super+a, Super+space, Super+n, the media keys, the give keys ...). Super+1..8 act.
- xp-window (xp with a foot, tiling joins): 15 lines (r1 233), only chords xp binds (Super+F1, Alt+Tab, ...);
  Super+Return, Super+h, Shift+Super+1 of tiling act after xp gave the foot. xp and mc logs: 0 refusals.

### r1 hand repros rerun (head)

Fresh `CHORA_TEST_FLAGS=--test-injected-press-is-real tests/persona/session.sh`, `act.py`; a foot with
`stty raw -echo; cat >FILE` is the focused client.
- xp 1280x800 (Super, click Notepad, then the foot): `type start:`, Super+x, `|`, Super+k, `|`, Super+m, Super+q,
  `:end`. File `start:||:end`; Start menu `visible: false`; then Super alone opens it (`true`).
- mc 1280x720, the foot a page, `key Return`, the same plus Super+Shift+x: `start:||:end`.

### Gate

Run 3 at 1eb30643: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`, EXIT 0, 65 PASS (`matrix` 285 s with
gate-join-keys, `xp-notepad` 51 s, `mc-paths` 120 s). Last lines:

```
tiling-windows PASS   150s
xp-run     PASS    11s
demo       PASS   194s
leak       PASS   309s
tiling-theme PASS    18s
tiling-disc PASS   70s
tiling-bar PASS   155s
mc-host    PASS    18s
e2e        PASS   265s
all steps passed; no --push, nothing is pushed
EXIT 0
```

Earlier runs, both mine: run 1 at 4455b2a3 red at `ci-check`
(`swallow.rs:300: grab_swallowed skips its grab on a branch`), fixed by 6a8bd3b9; run 2 at 6a8bd3b9 red at
`xp-notepad` (`FAIL: the start button opens no start menu`): xp now swallows from its start, and about 200 grabs, one
round trip each, sat between its scene and its `events.subscribe`, so the first click came while xp did not listen;
1eb30643 grabs them in one batch (green also with `TILING_LOAD=1`).
`tools/sleep-ratchet`: `ok: sleep-ratchet (sleep 779, sleep-all 930)`. `pgrep -fa 'chora --headless'`: empty.

### Found on the way (not fixed, outside the lane)

- When tiling joins, the focused window of xp or of mc loses the keyboard: no surface is focused
  (`ctl query '*'` shows none focused; xp state says holder xp; mc holder null) and in mc neither a click nor
  Return on the page gives it back. Seen on e1183766 binaries with waybar stubbed out, so not the bar; not checked on
  base. Against D119 ("the keyboard stays where the person is"). The cells work around it: xp-window clicks the foot,
  mc-first opens the page after tiling joined. Worth a finding (tiling or core).

### Cost

A join costs the earlier look one batch of about 200 `input.ungrab` and every subscriber of `input.ungrabbed` as
many events; while two looks run the unbound chords belong to the one that joined last. tiling and mc subscribe to
`input.ungrabbed` (one line each in `plugins/chora-tiling` and `plugins/chora-mc`, outside the listed key paths of
tiling, needed for the joiner's retry).

## Review r2

Reviewer lane review-stray-super-r2 (opus, high, cloud), read-only on `claude/stray-super-r2` 1eb30643, base e1183766.
Step 0: `git fetch --unshallow` ok, `cargo check --workspace` ok. `cargo test -p chora-kit -p chora-xp -p chora-mc -p chora-tiling`:
EXIT 0 (kit 330, xp 197, mc 353, tiling 193 passed). `bash tests/gate-join-keys.sh` on head: EXIT 0, 4 cells
(tiling-first, xp-first, xp-window: tiling refused 15, mc-first: tiling refused 19; Super+h, Shift+Super+1, Super+1..8 act;
`start::end` in both clients). `pgrep -fa 'chora --headless'` empty after.
Verdict: **FIX** (0 BLOCK, 1 MAJOR, 2 MINOR, 1 NIT).

Checks:
1. Red first holds. 429ce20f (cells xp-window, mc-first, three kit unit tests) before d88dee68 and 160b62fd. 6a8bd3b9: the existing
   ci-check `tests/grab-gap-check.py` is red at 4455b2a3 (`swallow.rs:394: grab_swallowed skips its grab on a branch`, rerun here)
   and clean at 6a8bd3b9. 1eb30643: the existing gate cell xp-notepad was red at 6a8bd3b9 (lane gate run 2, pasted line). No fix without a red.
2. The rule (`swallow.rs:212-264`, `runtime.rs:378-420`) holds for two joiners (a BTreeSet; the earlier look swallows again only
   when the set is empty), one leaving while another stays, a look reloading (a fresh runtime swallows, the others yield: D119
   "counts as the last"), and ungrab events before or after `client.disconnected` (waiting swallows are dropped at the yield,
   bound ones retried). A refused bound chord stays bound (`free_chords` skips it). But the rule assumes that the joiner always
   swallows and hears `input.ungrabbed` from its first grab: neither holds for the construct, and xp grabs before it subscribes
   (rss-7, rss-8, rss-9).
3. rss-1, rss-2 fixed (gate-join-keys on head, above); xp-46, mc-60 stay fixed in the cells and in xp-notepad (gate run 3).
4. 160b62fd: no `Chords`, `first_window` or `Swallowed` left in plugins/chora-xp; one dead binding in a test (rss-10).
5. Cost (about 200 `input.ungrab` per join, as many `input.ungrabbed` events, the unbound chords with the last joiner): accepted,
   no finding. It is one batch per join, D117 already counts the event cost, and who holds a swallow does not matter to the
   person as long as someone does: a swallow of tiling beside xp still ends xp's lone Super tap (`end_lone_taps_of_others`,
   `crates/chora/src/input.rs:322`), so Start stays closed. It breaks where the joiner swallows nothing (rss-7).
6. No change in crates/chora; 47aad40d polls (`until_ok menu_visible 4 && fail`), no new `sleep` in tests; D118 text matches
   the code except "the joiner gets every chord that it binds and swallows the rest itself", which rss-7..9 make untrue for the
   construct and racy for xp.

Repro of rss-7 on 1eb30643 (headless 1280x800, gate-join-keys helpers, script in the reviewer's scratchpad, not committed):
tiling, a typing foot focused, `start:` Super+x Super+k Super+m Super+q `:end` -> `start::end`; then `chora-construct --overlay`
starts (alive, waiting, no `construct.load`), the foot keeps the keyboard, the same typing -> `start:xq:end`. Control without
the construct: `start::endstart::end`. On e1183766 tiling never yields its swallow.

Findings:
- rss-7, MAJOR, crates/chora-kit/src/swallow.rs:215, every client of `give::KEYS` counts as a joiner, also `chora-construct --overlay`
  (docs/configuration.md:1092, a documented flow beside tiling, mc or xp) and the construct while no pane window has the keyboard,
  which swallow nothing (`camera.rs:567` swallows only with `keys_with_pane` and no cube): the flavor lets its swallow go and nobody
  holds the chords, so Super+x and Super+q type into a tiling window again (repro above), and by the same path into xp and mc
  windows (xp-46, mc-60 back) and tiling loses its D118 swallow of base; fix: yield only to a joiner that swallows (for instance the
  construct as joiner keeps the swallow of the earlier look, or the construct swallows from its start like the others), with a
  gate-join-keys cell tiling plus `construct --overlay` that types `start::end`.
- rss-8, MINOR, plugins/chora-xp/src/wm/startup.rs:118, xp grabs (`wm.grab()`) before `announce` subscribes, and the core sends no event to
  a connection without a subscription (`crates/chora/src/ipc.rs:645`): when xp joins and the earlier look's yield lands between xp's
  refused grab and its subscribe, xp never hears `input.ungrabbed` and the chord stays free (a bound key of xp dead, or an unbound
  chord typing into an xp window); the same window hides a `client.connected` of a manager that joins then, so xp never yields
  (rss-2 back); 1eb30643 narrowed the window (one batch), not closed it; fix: subscribe before the first grab, as tiling (`Runtime::start`)
  and mc (`lifecycle.rs:178`) do.
- rss-9, MINOR, plugins/chora-construct/src/main.rs:747, the construct's `EVENTS` lack `input.ungrabbed`, so as a joiner it never asks
  again for a binding that the swallow of an earlier look held at its grab, against D118 r2 "xp, mc, tiling and the construct share
  the code"; fix: add `input.ungrabbed` to the construct's events (one line, as tiling and mc got).
- rss-10, NIT, plugins/chora-xp/src/wm.rs:1205, `let _ = calls;` is left of the reverted call check in `a_moved_cube_key_has_no_stand_in`;
  fix: use `on_fake_core_refusing(..).0` or drop the recorder.
