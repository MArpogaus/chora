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

## Review r1

Reviewer lane (cloud, read-only on code) of claude/mc-page-end-r1, base 42418379, head 23cd2b5a (9 files, +386 -51). Step 0 green: `git fetch --unshallow`, `cargo check --workspace`.

1. Red first: `git log --oneline 42418379..23cd2b5a` gives 8a0249a2 (test) before 859cf3fe (fix), then f6f80bd8, 23cd2b5a. `cargo test -p chora-mc` at 8a0249a2: 206 passed, 1 failed, `lines cut with no card: ingredients/list 1 of 4, boil/side 1 of 3, sauce/side 1 of 5, morning/reading 0 of 3`; at 23cd2b5a: 208 passed. OK.
2. No card: `card_room` falls back to `foot_room` (footer size x baseline, 56 px), `page_bottom` ends the page over the footer line; a card counts only while it shows on mc's output at the foot and is taller than the footer room (rooms.rs `read_rooms`); `learn_rooms` rebuilds the pages on both changes and `rebuild_pages` clamps the scroll to the new `scroll_most` (layout.rs). Matches rule 13 and its new sentence. OK.
3. Keys: `page_key` (methods.rs) sends Down, Up, Next/Page_Down, Prior/Page_Up to `scroll_key` only on a NodeKey of the page node, under the same `by_person` rule as Left and Right; before they were dropped, so no client gets a key it did not get before. `more_update` counts the wrapped line ends (`text_height`, `fonts::lines_at`, at the same `w` that `Spot::text_at` gives `max_width`), OR'd with the rects. Second test `with_a_card_the_page_goes_on_and_the_keys_scroll_it` covers Down to the end (mark hides), Up (mark back), close (scroll 0, page whole). Probe on head, no card: ingredients mark on (the tip row lies past the footer), boil off and `scroll_most` 0, sauce on (25 px). OK.
4. `git diff --stat`: no chora-notify, no other look; no new `sleep`, no new allow. `tools/sleep-ratchet` ok (778/929), `tools/size-ratchet` ok, `tools/lint-ratchet` ok (allow 360); `cargo fmt --check` and `cargo clippy -p chora-mc --all-targets -D warnings` clean. OK.
5. Captures (media/2026-11-25/mc-page-end-r1/): m1-after shows all 4 list lines and the mark; m4-after shows "Then" 4 of 5 with the mark; m4card-after shows the page ending over the card, the mark beside "Then", the footer and the card uncovered. OK.

Findings:
- rpe-1, MINOR, plugins/chora-mc/src/keys.rs:181: the key list names ←→, Home and Esc for the page but not the new scroll keys, so a remote user sees the ↓ mark and is not told which key reaches the lines (the doc comments of `page_key` methods.rs:219 and `PageKeys::Plain` keys.rs:22 also leave them out); fix: add a "↑↓ Scroll the page" row beside Home while the page keys are plain, and name the keys in both doc comments.
- rpe-2, MINOR, plugins/chora-mc/src/lifecycle.rs:65: with the 135 px start room gone, a usual timer card now covers the page's last lines until the next rooms round (ROOMS_EVERY 2 x VOLUME_MS 500, up to about 1 s; gate-mc-sofa.sh:612 waits for it), where before it never covered them; fix: call `learn_rooms` right after mc's own `notify.push` (and on a shown/closed signal of the notice server if it has one).
- rpe-3, NIT, tests/gate-mc-sofa.sh:628: the "keeps the band of the closed card" fail prints `$page` as read while the card showed; fix: print a fresh `node_rect mc-s-morning-page` in that message.

Counts: BLOCK 0, MAJOR 0, MINOR 2, NIT 1. Verdict: **MERGE-OK**.
