# mc-page-end

## r1

Branch claude/mc-page-end-r1, base 42418379 (mc-minor r3), head 23cd2b5a (red 8a0249a2, fix 859cf3fe, f6f80bd8, gate cell 23cd2b5a). Step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`.
Diffstat 42418379..23cd2b5a: 9 files, +386 -51 (docs/mc-style.md, chora-mc build.rs, config.rs, fonts.rs, layout.rs, main.rs, methods.rs, rooms.rs; tests/gate-mc-sofa.sh).

**Step 1: the cut is still on 42418379.** Headless mc at 1280x720 with the tour's recipe scenes and chora-notify (no card on screen): `mc-s-ingredients-page` rect [0,56,1280,503] (ends at y 559); `mc-s-ingredients-panels-list` [56,528,776,31], `-list-lines` 29 px high (1 of 4 lines, 112 px); `mc-s-boil-panels-side-lines` 29 px (1 of 3); `mc-s-sauce-panels-side-lines` 29 px (1 of 5). The more mark was hidden on boil and sauce. mc-minor r3's card room did not change this, because the 135 px start room applies with no card too.

**Red** 8a0249a2 `test(mc): a recipe's last row shows whole with no card (rv9ka-1)`, a unit test in rooms.rs:
`lines cut with no card: ingredients/list 1 of 4, boil/side 1 of 3, sauce/side 1 of 5, morning/reading 0 of 3`
**Fix** 859cf3fe `fix(mc): the page ends at the footer, a card's band only while shown`; f6f80bd8 drops the two cast allows that the lint-ratchet counted (allow 362 > list 360), so it adds no new allow. 208 chora-mc tests pass; clippy is clean; lint, size and sleep ratchets are ok (sleep unchanged at 778/929).
**Gate cell** 23cd2b5a: mc-sofa `page-end` checked the page once, right after the card showed. mc reads the card on its next round of the rooms (about 1 s), so the cell now waits until the page clears the card. After `notify.clear` it waits for the band to come back to the footer, asserts that the page stays over the footer line, and then looks for a cut line at the new edge.

**The design choice.** The page reserves a card band only while a card shows at the foot of mc's output. With no card, the page ends over the footer line (`foot_room` = footer size x baseline, 56 px; page node to y 643 at 720). A foot card that reaches above that room takes its band while it shows and gives it back on close. The page is rebuilt on both changes, and the scroll is clamped to the new end, so a page that fits is whole again. At 720 px some lines still cannot fit: rows keep their `min_panel` height (152 px), so a 2-row lead stacked over a 2-row side ends past the footer. In v8 those lines showed only because the page ran on under the text block. For those pages, mc now says that the page goes on and lets the remote reach the rest:
- The more mark counts where a panel's lines end, measured as the core wraps them (`fonts::lines_at`, glyph advances without kerning, so a little wide). Before, it counted the page-clipped text rect, which never passes the edge.
- `scroll_most` ends at the last line of text.
- Down, Up, Page Down and Page Up on the page scroll it, as the wheel does.

I tried growing the rows to their lines first and dropped it: the cause is the page end, not the share, and it would have changed other looks. chora-notify is not changed. rmm2-2's "a card that fits the room never moves the pages" still holds for the footer room; its test card moved from room 100 to room 50, because a 100 px card now reaches the page.

Second test (the scroll path): `with_a_card_the_page_goes_on_and_the_keys_scroll_it`. A card [160,585,291,79] gives a 135 px band, the Timer is cut and the mark shows; Down x4 shows all 3 lines and hides the mark; Up shows the mark again; on close the band comes back, the scroll is 0 and the page is whole. The red test's check is the brief's own "or": every line shows, or the page reaches the footer and shows its mark (sauce "Then" 4 of 5 and the morning reading lie past the footer at 720). Note: the brief names "morning"; I took the tour's sauce page ("Then", act m4) and the built-in morning page both.

Headless after (no card): page [0,56,1280,587] to y 643; `list-lines` 112 (4 of 4), boil `side-lines` 84 (3 of 3), sauce `side-lines` 113 (4 of 5, more mark shown, Down reaches line 5). With the card: page to 559, mark shown, Down moves the panels by 28 px per press. When the card expired the page grew back and the scroll was clamped.

docs/mc-style.md rule 13: the page ends over the footer line, and over a card only while one shows; it scrolls with the wheel and with Down, Up, Page Down and Page Up.

Captures at 1280x720 (`media/2026-11-25/mc-page-end-r1/`, each under 80 KB): `m1-before.jpg`, `m1-after.jpg` (ingredients); `m4-before.jpg`, `m4-after.jpg` (sauce, "Then"); `m4card-before.jpg`, `m4card-after.jpg` (the timer card at the foot: the mark shows after).

Gate, no step filter (`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`) on f6f80bd8: red-first .. mc-paths 43 PASS, then **mc-sofa FAIL** (the gate stops there), with two red cells:
- `volume-wheel`, foreign: `FAIL: a wheel up over the volume door does not raise the volume (rpx1-15): get-volume @DEFAULT_AUDIO_SINK@|get-volume @DEFAULT_AUDIO_SINK@|`. This is the known train141 red, and it passed in the rerun below.
- `page-end`, mine: `the page [0.0,56.0,1280.0,587.0] reaches under the timer card [160,585,291,79] (rpx1-11)`. Fixed by 23cd2b5a.

On head 23cd2b5a, following the tiling-doors r4 precedent: `--only` red-first, identity, pre-commit, ci-check and every step after mc-sofa (mc-column .. e2e) gave 24 PASS, EXIT 0; then `--only mc-sofa` gave PASS (`ok: page-end (dark)`), EXIT 0. A single unfiltered run on 23cd2b5a was not made. `pgrep -fa 'chora --headless'` is empty.
