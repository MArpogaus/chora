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
