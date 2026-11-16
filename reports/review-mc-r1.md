# Review mc r1 (sonnet, read only)

Base caf9c80d, head c2c9c353 (11 commits, 5 files). Verdict: MERGE-OK (no open BLOCK or MAJOR among the fixed items).

| # | sev | where | finding |
|---|---|---|---|
| 1 | holds | main.rs:3229-3236 (replace_panel), 1186 | mc-32 BLOCK: take_free_keyboard after a panel that moved/filled/emptied; it only acts when nothing has focus, so a click-focused window or node is not stolen. Correct. |
| 2 | holds | region.rs:824, 1745, main.rs rebuild_pages | mc-37: a hidden side has width 0, a Left/Right toggle rebuilds all pages. Correct; restores the keyboard. |
| 3 | holds | region.rs:402-428, 1319-1330, mic.rs:75 | mc-36 overlap: Put::After puts Mute beside the state word only beside the host. End/Stack unchanged, so xp/tiling/construct (no host-mc case) unaffected. |
| 4 | holds | region.rs:797 | mc-35 empty head: `region != Right` means the column never draws a title for an empty text, also on a first explicit open. Fine (that was the bug); Left unchanged. |
| 5 | verdicts | mc-33, mc-36 "No helper" | Both "not a product fault" accepted. "No helper" is the host's word for no helper and a resting agent; the tour fed hud.say of mc, which the host never sees. mc-33: the door moved 84 px after the helper row appeared; the new `door` cell waits for HELPERS and passes. Code agrees (mic.rs, the open-chat test the_open_chat_leaves_the_words_of_no_helper_to_the_host). |
| 6 | LOW | region.rs:2003-2011 | `left_of` was inserted between the doc comment of `lead` and `lead`: the comment "The room that an item keeps before its text for its close mark" now sits on left_of. Move it back. |
| 7 | LOW | mic.rs:62-66 | doc comment line over 100 chars, rustfmt-unfriendly. |
| 8 | LOW | rebuild_pages | Rebuilds every scene on a side toggle, not only the current one; fine at this size. |
| 9 | LOW | dda31e6a | mc-33 test is green at its red commit by design (script fault); no red-first needed, but the commit type `test` only. |
| 10 | open, not blocking | mc-34 MAJOR, mc-35 (critical card, overflow guard) | Declared open for the next round. |

## Rerun
- `cargo test -p chora-mc` at HEAD: 122 passed, 0 failed.
- Reds at their commits (sources touched, since a first run on stale mtimes falsely passed): 37d0803b a_page_keeps... FAILED (left None, right Some("mc-s-agenda-page")); d4938d1c the_page_widens... FAILED; 5a34fb82 a_column_that_came_back... FAILED (extra notes-title). All three green at HEAD.
- tests/gate-mc-sofa.sh: unverified. It needs the built binaries (chora, choractl, chora-mc, chora-assistant, chora-chat); the workspace build failed with "No space left on device" (volume had 5 MB free). Read the script: cells remote, wide, door, status, empty-head assert the right things; merge-gate.sh step mc-sofa is wired. Builder's 10/10 runs accepted unchecked.
- Build dir /home/user/wt/rev-mc/target removed.
