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
