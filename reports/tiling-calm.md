
## r1

Lane tiling-calm r1 (sonnet, rc 2, 8), claude/tiling-calm from ad0093ba. **Head e9387fb2**, pushed. Full gate (`tests/merge-gate.sh --jobs 2`, no --only) on e9387fb2: every step PASS, rc 0, no foreign red (gate logs /tmp/merge-gate.74ikxa). Two earlier full runs stopped on my own changes and were fixed (see Gate).

### Commits (red first)
- 1f722eb3 red ti-27: `tests/gate-tiling-power.sh` (step tiling-look) opens the key list, then the power menu, wants the list gone, Escape, wants no overlay; the same for the Apps list; `cargo test -p chora-tiling`: `an_opening_helper_panel_closes_the_key_list`, `the_method_hide_keys_closes_the_key_list_and_is_idempotent`. Red on ad0093ba: `FAIL: the key list stays open under the power menu (ti-27)`; both unit tests FAILED (10 passed, 2 failed).
- 6b526318 red ti-50: `tests/gate-tiling-apps.sh`, a stand-in `chora-chat --door` says "atlas · idle": `FAIL: no launcher row "Chat with a helper · atlas" ...: alpha|Dark look|...|Zebra|Chat with a helper` (the red test named the row with a middle dot; the ASCII form came with 985c3373).
- 4a284cdc red rtr-1/rtr-3: `tests/gate-tiling-one-bar.sh lock` (new cell, no session, fake waybar and choractl): `FAIL: lock: waybar inherits the descriptor of panel.lock (9)`. With only `9>&-` added: `ok` for that line, then `FAIL: lock: with no flock the lock is skipped silently ... (rtr-3)` (checked by hand on the base script with the one change).
- af75e588 fix ti-27: new exported method `hide_keys` (idempotent; docs/configuration.md); `chora-tiling --launch power` and `--launch fuzzel` (no args; Super+space, Super+r, the Apps word of the bar and the power icon all go through it) call it first on a connection of their own (1 s); the helper panel that opens (`assistant.chat` open with a rect) closes the list in `helper_panel`. Escape therefore closes the one layer that is on top and nothing stays behind it.
- 0576d51a, 985c3373 fix ti-50: the Apps row of the helper reads `Chat with a helper (NAME)` when `chora-chat --door` names a helper, plain `Chat with a helper` otherwise; `chora_kit::launcher::{helper_of_door, helper_named, helper_name}` (the door read lives in chora-kit because cut-check forbids chora-tiling naming chora-chat). It is the same row and the same command (`chora-chat`) as Super+a and the bar word. ASCII on purpose: fuzzel in the C locale matches no row with a multibyte character (measured below).
- 36ca0845 fix rtr-1, rtr-3: `waybar ... 9>&- &` in flavors/tiling/waybar/waybar.sh (the lock lives with the script); without `flock` one stderr line: `waybar.sh: flock (util-linux) is not installed; this start cannot tell whether the panel already runs`. Cell `gate-tiling-one-bar.sh` (all of lock, product, unit, persona) stays green in the gate step tiling-look.
- 9af3321f: one `launch_conn` with the single lint allow (lint-ratchet 360, was 361 after the first version).
- e9387fb2: `tests/doors/tiling.sh` cell launcher-chat accepts `Chat with a helper` with an optional ` (name)`; it failed the gate (`launcher-chat missing` for tiling) after ti-50.

### ti-50 and the key list row
The finding "typing helper finds nothing" does not hold on the plain row (measured: fuzzel with the tiling fuzzel.ini finds `Chat with a helper` for "helper" in a 21-row list); it is the helper's name ("atlas") that matched nothing, and the row stands last of 21 (lines=9). Now typing atlas or helper finds `Chat with a helper (atlas)`, Return opens the chat (same `chora-chat` as Super+a; not clicked end to end here, the gate cell checks the row text and the doors cell `launcher-chat` the row with the name). The key list row for Super+a keeps its words "Chat with a helper": the same words, without the name (the key list is built from the binding file, not from the presenter; adding the name there is a second source of truth, not done).

### ti-51 (verify only, no fix)
Script and captures: media/2026-11-24/tiling-calm-r1/ti51/ (ti51.sh, key-list-alone.png, key-list-under-popup.png). One tiling session on e9387fb2-equivalent binaries, key list open, `tests/png-contrast.py` over each of the 100 `tiling-keys-body-*` text boxes, once alone and once with a popup (`fuzzel --dmenu`, box x 464-814 y 240-560 found from the pixel difference of the two captures):
- alone: 100 rows, 0 under 7:1.
- under the popup: 1 row under 7:1, `1.00:1 'Tags' tiling-keys-body-72`, and its box lies inside the popup box.
**Verdict: the check reads hidden rows.** A text node under an opaque layer-shell popup is measured on the popup's pixels (uniform, 1.00:1); the audit has no node for fuzzel, so it cannot know the row is covered. The rows are really readable (0 of 100 under 7:1 alone). With ti-27 the product no longer leaves the list under a popup through the bar, the keys or the Apps list, so the case no longer arises on those paths; the audit itself still reads hidden rows if something else covers the list (`fuzzel` started by hand). No fix to the check made (rule: no fix unless 1) does not remove the case).

### Captures (media/2026-11-24/tiling-calm-r1/, 1280x800)
1-key-list.png, 2-power-menu-key-list-closed.png (power icon clicked with the list open: the menu only, list gone), 3-after-escape-nothing-open.png (one Escape: nothing open), 4-key-list-again.png, 5-launcher-key-list-closed-helper-row.png (Super+space with the list open: launcher only), 6-launcher-typed-atlas.png (typed atlas: the one row `Chat with a helper (atlas)`, the bar says `atlas · idle`). Harness note: the helper name is a stand-in `chora-chat --door`; no voice agent ran.

### Gate
- Full run 1 (e.g. gate3): killed by the 30 min cap of my own background wrapper at step layout, all earlier steps PASS; run 2: `ci-check` FAIL `lint-ratchet: allow is 361` and `cut-check: chora-tiling names chora-chat` (fixed, 985c3373, 9af3321f); run 3: `doors` FAIL `launcher-chat missing` for tiling (fixed, e9387fb2); run 4 on e9387fb2: every step PASS, rc 0.
- Foreign reds: none.
- `cargo test -p chora-tiling -p chora-kit`: all ok (chora-tiling 197, chora-kit 328 incl. the new tests).

### Left / notes
- A bar click on Keys with the Apps list or power menu open is not handled (the popups hold the keyboard and are layer surfaces; tiling hears nothing of them). The helper opening over a key list closes the list; the key list opening over an open helper panel leaves the panel (not asked; the panel is a side, not a popup).
- `launch` now makes one `methods.call` (1 s at most) before the launcher or the power menu; with no tiling listening it ends silently and the launcher opens as before.
- Review hint: the `chora-chat --door` read in `helper_name()` waits at most 2 s on a thread; a hung presenter delays the launcher by that.

## Review r1

Review of claude/tiling-calm r1 (head e9387fb2, base ad0093ba; 12 files, +283 -22) by the build lane review-tiling-calm-r1 (sonnet, high; no merge gate run, the lane gate is all PASS on e9387fb2). Step 0: `git fetch --unshallow`, `cargo check --workspace` ok, `cargo test -p chora-tiling -p chora-kit` ok (kit 328, tiling 197).

**Verdict: MERGE-OK** (0 BLOCK, 0 MAJOR, 5 MINOR, 2 NIT).

### Checks
1. Red first: built the tree at 4a284cdc (tests of 1f722eb3, 6b526318, 4a284cdc over the code of ad0093ba) and ran each cell, then at e9387fb2.
   - ti-27: `tests/gate-tiling-power.sh` red: `FAIL: the key list stays open under the power menu (ti-27)`; head: rc 0. `cargo test -p chora-tiling -- an_opening_helper_panel the_method_hide_keys`: both FAILED at the red commit.
   - ti-50: `tests/gate-tiling-apps.sh` red: `FAIL: no launcher row "Chat with a helper · atlas" ... alpha|Dark look|...|Zebra|Chat with a helper`; head: rc 0 (`ok: the launcher has one helper row, with the name of the helper (ti-50)`).
   - rtr-1/3: `tests/gate-tiling-one-bar.sh lock` red: `FAIL: lock: waybar inherits the descriptor of panel.lock (9), so a child that outlives it keeps the lock (rtr-1)`; head: both `ok` lines. Every fix has a red before it.
2. ti-27: `launch` calls `close_key_list` only for `power` and for `fuzzel` with no args (main.rs:311-313; Super+space, Super+r, bar Apps, power icon go through them); `fuzzel --dmenu ...` and every other program leave the list. The helper panel closes it only when it opens with a rect (main.rs:751). Super+slash alone still opens and holds the list (`open_list` in the power cell). Capture 2 and 3 show menu only, then nothing. Focus after close: see rtc1-3.
3. ti-50: the row is the same `chora-chat` row as Super+a (`helper_named` renames only the row whose last word is `chora-chat`, launcher.rs:117-124); typing atlas or help finds it (capture 6); ASCII choice stated in the doc comment (launcher.rs:111-114). The doors cell regex `^row Chat with a helper( \([^)]+\))?$` (tests/doors/tiling.sh:121) still demands the row and only allows the name: no assert dropped.
4. rtr-1/3: `9>&-` at waybar.sh:98 (the only `&` line of the script); the else branch prints one line (waybar.sh:46-47). `tools/lint-ratchet`: ok (allow 360); the one `#[allow]` moved from `tell_launch_error` to `launch_conn` (net 0), no `ratchet-raise:` needed.
5. ti-51: the lane's verdict "check reads hidden rows" is right and a harness fault (rtc1-5), not a product one.
6. KISS/POLS/look: one new exported method `hide_keys` (documented, listed in methods.rs and its tests), no new mode, setting or look. crates/chora-kit callers: `launcher::run` (chora-mc main.rs:460, chora-xp launch.rs:43) passes `None`, so their lists are unchanged (kit unit tests; xp and mc launcher cells); only chora-tiling passes `helper_last` and the name.

### Findings
- rtc1-1, MINOR, crates/chora-kit/src/launcher.rs:96-107: `helper_name` runs `chora-chat --door` before every Apps list; a hung presenter delays the list 2 s and the child is left running (the thread is dropped, nobody kills it). Fix: read the door with a deadline that kills the child, or take the name from the bar state that tiling already holds.
- rtc1-2, MINOR, crates/chora-kit/src/launcher.rs:117-124: the name comes from the door as it is; a helper named with a multibyte character (Zoë) gives a row that, by the lane's own measurement (fuzzel in the C locale), no typed text finds, not even "helper", worse than the plain row. Fix: append the name only when it is ASCII, else keep the plain row; add the case to `the_helper_row_names_the_helper_that_the_door_names`.
- rtc1-3, MINOR, tests/gate-tiling-power.sh:105-130: the cell has no window, so "no lost focus after the layer closes" is not asserted (hide_keys sends `FocusNode none`). Fix: run one foot first and want it focused after the Escape of each sequence.
- rtc1-4, MINOR, tests/gate-tiling-apps.sh:76-83: no cell chooses the helper row and sees `chora-chat` start (lane: not clicked end to end). Fix: let the stand-in `chora-chat` write a mark and pick the row with the fake fuzzel's answer.
- rtc1-5, MINOR (harness, ti-51), tests/acts/invariants.py:565-580: `guide_breaches` measures each text node's rect on the capture with no check that a layer-shell popup covers it, so rows under a fuzzel popup read 1.00:1. Fix: skip a node (and name it) whose rect lies inside a region that differs from the scene render or when a layer surface covers it; no product fix.
- rtc1-6, NIT, tests/gate-tiling-apps.sh:76: the final `(atlas)` form was never red first (the red cell asked for the middle dot, 6b526318; the form changed with the fix 985c3373). Fix: none needed; say so in the report.
- rtc1-7, NIT, plugins/chora-tiling/src/main.rs:751: the key list opening over an open helper panel leaves the panel (the lane says so, not asked). Fix: none; keep as a known edge.

Known, not findings: ti-47, ti-48, sh-39, sh-40, co-*, xp-*.

## r2

Head bb4c07d9 on claude/tiling-calm-r2 (merge of d40142e4 = 9649ef3f, fix bb4c07d9).

- Red: after the merge `tools/sleep-ratchet` said "sleep is 782, the list holds 779" and "sleep-all is 933, the list holds 930".
- Fix (test only; message cites sh-guard sleep ratchet, ti-27, rtr-1; no ratchet-raise on it): open_list and list_gone use `until_ok` (10 rounds = 5 s, as the old 20 x 0.25 s); the fake waybar is `exec tail -f /dev/null`; the tool list is `... kill sed tail sleep` (waybar.sh itself calls sleep; the word ends the line, so nothing counts).
- Asserts, before and after, unchanged: the loop ending in `fail "Super+slash opened no key list"` becomes `until_ok list_open 10 || fail "Super+slash opened no key list"`; list_gone still fails when the list stays and the caller keeps `fail "the key list stays open under the power menu (ti-27)"`; the lock cell asserts are untouched.
- Green: `tools/sleep-ratchet` ok (sleep 778, sleep-all 929). gate-tiling-power 5/5, gate-tiling-one-bar 5/5.
- The merge commit carries a `ratchet-raise:` line only because tools/sleep-ratchet.list arrives as a new file with the merge (779/930, no raise).
- Full gate (--jobs 2): every cell PASS except tiling-bar: "FAIL: persona session.sh tiling: 2 waybar processes on one config" (a load race in the persona part, untouched by r2). gate-tiling-one-bar.sh alone then ran green. Not caused by r2.
- `pgrep -fa 'chora --headless'` empty.
