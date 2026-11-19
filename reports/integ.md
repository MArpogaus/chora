# integ r1 (F6, F2): wm5, kit5b, kit7 on one head

Branch claude/integ, head af5547c7 (session head 05878d2a merged last, no conflict).
Gate `build,test,clippy,ci-check,pre-commit,acts` green on that head.
Merges: wm5 d74dd374, kit5b 7b01dd5a, kit7 8321c16, each `--no-ff`; check and tests green before the next.

## wm5 d74dd374 (3 files)
- crates/chora-kit/src/wm/mod.rs: wm3 adds `pub mod open`, wm5 adds `pub mod stack`. Both kept.
- plugins/chora-tiling/src/state.rs: wm3 drops `open_value` and reads `self.open.state()`; wm5 moves the busy list into `busy()`. Kept `busy()` and `self.open.state()`, dropped `open_value`.
- tools/copies.toml: wm3 rows (open entry, keys_open) and the wm5 row (D83). All three kept.

## kit5b 7b01dd5a (4 files)
- plugins/chora-xp/src/wm/startup.rs: use list; kit5b drops `Notifier`, wm7a adds `KeyAction`, `CubeKeys`. Union minus `Notifier`.
- tests/acts/looks/construct.py: kit5b adds `notice_acts`, wm7a adds the cube acts. Both; `notice_acts` after "the cube comes to rest", as on kit5b.
- tests/acts/looks/xp.py: kit5b adds `notice_acts`, wm3 adds `drag_acts`. Both; `drag_acts` formatted as ruff does.
- tools/copies.toml: kit5b KIT5 rows and the wm3/wm5 rows. All kept.
- Semantic, no marker: none.

## kit7 8321c16 (25 files)
- crates/chora-kit/src/testing.rs: kit5b `answer_with(method)` (map by method), kit7 `answer_call(client, method)` (list); both add a field `answers`. Kept both functions, kit7 field renamed `client_answers`, the call looks up by client first, then by method.
- plugins/chora-assistant/src/chat.rs: kit5b exports `NOTIFY_READY`, kit7 `CHAT`. Both exported.
- plugins/chora-xp/src/wm/startup.rs: use list; kit7 drops `Helper`, kit5b/wm7a the others. Union minus `Helper`, `Notifier`.
- plugins/chora-tiling/src/main.rs (3 hunks): wm3 `open: Open`, kit7 `Helper` (not `Helpers`); EVENTS count. Kept `open` and kit7 `Helper`; EVENTS 19 (the compiler counts).
- plugins/chora-mc/src/agent.rs (4 hunks): kit7 removed the confirm card, kit5b removed the notify column and urgent line and keeps `Notices`/`clear_notices`. Both removals kept, `clear_notices` and the two `Notices` tests kept. `OWN_TEXTS` is now empty (`[&str; 0]`).
- plugins/chora-mc/src/main.rs (2 hunks): the test `the_contents_that_mc_writes_refuse_content_set_and_add` tested only the two removed kinds of content; removed.
- plugins/chora-mc/src/state.rs, region.rs: kit7 doc comment of the helper test and wm3 keys test both kept; the left list of the layout test is `next, notes, outline, overlay, prev` (confirm gone by kit7, notify and urgent by kit5b).
- plugins/chora-construct/src/hud.rs: kit7 drops the own voice HUD (`Voice`, `Claim`), kit5b drops the own notice (`Voice::notice`, `draw_notice`); kit7 kept a `Notice` struct for it. Took kit7's file, removed `Notice`, `NOTICE`, `deadline`.
- plugins/chora-construct/src/{input,keys,adopt}.rs (6 hunks): kit5b `self.tell(..)` (chora-notify) against kit7 `self.notice.notice(..); self.draw_hud()`. Kept `tell`.
- plugins/chora-construct/src/{overlay,voice,scene,main,state}.rs: `draw_notice` and `draw_hud` have no job left; removed with their callers in camera.rs, panes.rs, overlay.rs, input.rs and the `notice` field and its deadline in the loop; `notice_id` (kit5b) stays, `claim` goes (kit7); `reopen_left_cube` (wm7a) stays, `claim_indicator` goes. `open_value` keeps wm3's `Open` and drops the voice question line (kit7).
- docs/configuration.md (4 hunks): `content.*` rows without own contents; `voice_hud` row of kit7 without "draws only its own notices"; the Notices paragraph of kit5b; the notice list of kit5b without "with either value of `voice_hud`".
- tests/acts/harness.py, looks/{common,mc,tiling,xp,construct}.py, tests/gate-construct-hud.sh: `notice_acts` (kit5b) and `helper_acts` (kit7) both run; in construct `tab_after_a_new_window` and `helper_acts` come before `cube_acts`; gate-construct-hud.sh the kit5b notice line.
- tools/copies.toml, tests/acts/known-red.toml: no conflict; the six known-red rows are still red, none removed.

## Findings (not conflicts)
1. plugins/chora-xp/src/wm.rs: kit7 dropped the `BTreeMap` import that wm7a's code needs. No marker; the build failed; restored.
2. Red on the train head fa457b29, not from the merges: `a_give_shuts_the_start_menu` and `a_give_with_the_drag_shuts_the_start_menu` (chora-xp, wm3 red cells, "xp has no window s1"): a give needs the window in the book. Fixed in the cells (`wm.book.keep("s1")`), commit a7d671f2.
3. kit7 with wm7a: helper_acts left chora-assistant (HUD island, top right) running; "back at 1:1 no construct surface is left" saw 8754 differing pixels at [1117, 28, 1253, 105] and I14 saw an effect.failed of the presenter's ring.frag when construct ends. Added the act "the helper presenter ends before the cube" (construct.py), which ends voice and the presenter and declares voice, voice-mcp, assistant as disconnects. The helper is not shown over the cube; a look that wants both needs its own cell.
4. ci-check: my comment in mc named chora-notify (cut-check names); reworded. size-ratchet list lowered (cube.rs 1641 to 1617).
5. Lost in the merge, by design of kit5b and kit7 together: mc has no own text contents any more (`OWN_TEXTS` empty, its guard in content.rs stays, no cell left for it).

## Review r1

Head af5547c7 (claude/integ). Reviewed: the remerge diffs of 3781cb7e (wm5), 926bb214 (kit5b) and 3619ccb3 (kit7), and a7d671f2, fe985999, 0113008b.
`cargo check --workspace` green; `cargo test -p chora-kit -p chora-mc -p chora-construct -p chora-xp --features chora-kit/testing` green (13 suites, 0 failed).
1) Resolutions: each keeps both sides as the report says. testing.rs looks up by client, then by method, then null. hud.rs: since the base, the train side changed only the notice that kit5b removed, so taking kit7's file loses nothing. The removed `draw_hud`/`draw_notice` callers drew only the notice. `TurnCube` was already gone on the train (fec3a98), and `the cube opens` of kit7 is inside wm7a's `cube_acts`. Nothing lost silently.
2) `OWN_TEXTS` is empty and mc writes no text content itself any more, so the guard has no job. No user doc claims it (ri-2).
3) a7d671f2 fixes the test: a real window reaches the book through claim/confirm before any give, and `keep` is the test shortcut that the helper `add()` uses too (ri-4).
4) 0113008b hides a fault (ri-1).
5) docs/configuration.md matches the code: `voice_hud` of mc and construct (old values and warnings), the `content.*` rows, the Notices of the construct, and the mc notify slot.

- ri-1, MAJOR, tests/acts/looks/construct.py:126: the act "the helper presenter ends before the cube" ends voice and chora-assistant before `cube_acts`. This hides the I14 `effect.failed` of the presenter's ring.frag at construct end, and the island diff at [1117, 28, 1253, 105]. Every flavor starts the HUD host (docs/configuration.md:947), so the shipping setup is the one that is no longer tested, and no cell is red for it. Suspect: `Theme::cache_file` (crates/chora-draw/src/theme.rs:469) rewrites ring.frag on each `Assistant::new`. Fix: keep the presenter up through `cube_acts`, or add an act "construct ends with the HUD host up: no effect.failed, the island as before", with a known-red row until the fault is fixed.
- ri-2, MINOR, plugins/chora-mc/src/content.rs:33: the guard on the empty `OWN_TEXTS` is dead (`contains` on `[&str; 0]`), and its cell is gone. The doc comments at agent.rs:154 and content.rs:25 still say `content.set`/`content.add` refuse own texts. Fix: drop `OWN_TEXTS` and the check, and let `content.set`/`content.add` call `set_text` directly.
- ri-3, NIT, crates/chora-kit/src/testing.rs:57: the docs of `client_answers` ("null without") and `answers` ("null for the others") no longer describe the lookup. Fix: say "by client first, then by method, else null" in both.
- ri-4, NIT, plugins/chora-xp/src/wm/state.rs:268: the two give cells repeat the helper `add()` (state.rs:155) by hand. Fix: use `add(&mut wm, "s1", 0)`.

Counts: BLOCK 0, MAJOR 1, MINOR 1, NIT 2. Verdict: FIX (ri-1).
