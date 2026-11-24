
## r1

Lane tiling-calm r1 (sonnet, rc 2, 8), claude/tiling-calm from ad0093ba. **Head e9387fb2**, pushed. Full gate (`tests/merge-gate.sh --jobs 2`, no --only) on e9387fb2: every step PASS, rc 0, no foreign red (gate logs /tmp/merge-gate.74ikxa). Two earlier full runs stopped on my own changes and were fixed (see Gate).

### Commits (red first)
- 1f722eb3 red ti-27: `tests/gate-tiling-power.sh` (step tiling-look) opens the key list, then the power menu, wants the list gone, Escape, wants no overlay; the same for the Apps list; `cargo test -p chora-tiling`: `an_opening_helper_panel_closes_the_key_list`, `the_method_hide_keys_closes_the_key_list_and_is_idempotent`. Red on ad0093ba: `FAIL: the key list stays open under the power menu (ti-27)`; both unit tests FAILED (10 passed, 2 failed).
- 6b526318 red ti-50: `tests/gate-tiling-apps.sh`, a stand-in `chora-chat --door` says "atlas · idle": `FAIL: no launcher row "Chat with a helper (atlas)" ...: alpha|Dark look|...|Zebra|Chat with a helper`.
- 4a284cdc red rtr-1/rtr-3: `tests/gate-tiling-one-bar.sh lock` (new cell, no session, fake waybar and choractl): `FAIL: lock: waybar inherits the descriptor of panel.lock (9)`. With only `9>&-` added: `ok` for that line, then `FAIL: lock: with no flock the lock is skipped silently ... (rtr-3)` (checked by hand on the base script with the one change).
- af75e588 fix ti-27: new exported method `hide_keys` (idempotent; docs/configuration.md); `chora-tiling --launch power` and `--launch fuzzel` (no args; the door of the bar, Super+Return-free paths: Super+space, Super+r, the Apps word, the power icon all go through it) call it first on a connection of their own (1 s); the helper panel that opens (`assistant.chat` open with a rect) closes the list in `helper_panel`. Escape therefore closes the one layer that is on top and nothing stays behind it.
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
- Full run 1 (e.g. gate3): killed by the 30 min cap of my own background wrapper at step layout, all earlier steps PASS; run 2: `ci-check` FAIL `lint-ratchet: allow is 361` and `cut-check: chora-tiling names chora-chat` (fixed, 985c3373, 9af3321f); run 3: `doors` FAIL `launcher-chat missing` for tiling (fixed, e9387fb2); run 4 on e9387fb2: all 66 steps PASS, rc 0.
- Foreign reds: none.
- `cargo test -p chora-tiling -p chora-kit`: all ok (chora-tiling 197, chora-kit 328 incl. the new tests).

### Left / notes
- A bar click on Keys with the Apps list or power menu open is not handled (the popups hold the keyboard and are layer surfaces; tiling hears nothing of them). The helper opening over a key list closes the list; the key list opening over an open helper panel leaves the panel (not asked; the panel is a side, not a popup).
- `launch` now makes one `methods.call` (1 s at most) before the launcher or the power menu; with no tiling listening it ends silently and the launcher opens as before.
- Review hint: the `chora-chat --door` read in `helper_name()` waits at most 2 s on a thread; a hung presenter delays the launcher by that.
