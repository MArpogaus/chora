# Review mc r6 (e84e4d30)

Verdict: MERGE-OK

No BLOCK or MAJOR. Two MINOR, one COSMETIC.

## 1. Cause and fix vs rv5-2
Cause is credible from the code: the mc root is created at ROOT after xp's root (xp and tiling start first), so it is drawn above
xp's nodes; during a title-bar drag the window is an xp node (hand-over only at release), so the opaque mc page covers it. The
fix (`Command::Move{id:"mc", parent:None, index:Some(0)}` in `build`, plugins/chora-mc/src/main.rs:1792, fn at :3417) puts the mc root
at the bottom of ROOT's children (scene.rs `link`: index.min(len), insert; `move_node` keeps the parent for `None`), so the xp
node is drawn above it during the drag; after the drop mc owns the window inside its own scene (cell drag-dropped, 462 ms).
It addresses rv5-2 as stated (visible during drag and after drop, no ~6 s gap). I could not check the tour media/frames; the claim
rests on the code reading and the builder's cells. Moving after apply_scenes is fine: mc-owned windows live under mc's own
nodes, and `unbuild` parks surfaces at ROOT only until the rebuild reparents them.

## 2. Side effects of mc below the other roots
- Notifications, assistant HUD/chat/glass/voice all draw under `screen:OUTPUT` (chora-notify card.rs:287, chora-assistant
  assistant.rs:470, chat/view.rs:1058, voice.rs:513), not under ROOT, so they are unaffected. mc's own cards, regions, HUD line
  are children of the mc root (regions_on_top reorders inside it), so order among ROOT roots does not touch them.
- Other plugin roots that can now cover the mc page: xp (WALLPAPERS/BARS/WINDOWS per its own outputs, wm.rs:650,693) and
  tiling (root at main.rs:518, windows only). They are limited to their `--output` outputs, so no cover in a normal multi-look
  session. If a second WM were started without `--output` on the mc output, its opaque wallpaper would now cover mc and swallow
  hits (hit_list(ROOT) is order-based, render.rs:1229). Before, a rebuild put mc back on top. This is a misconfiguration, so
  MINOR only (rv6-1).
- Only the build position changes: a plugin created after mc was already above it, and one created before is now above it; the
  order is now deterministic (mc at the bottom) instead of by start order. Unowned surfaces waiting at ROOT are now drawn above
  the page; acceptable.
- No unit test asserts the root order (only the gate); rv6-2.

## 3. Red test
tests/gate-mc-handover.sh: on 4105f8d1 mc root is above xp's node, the held drag leaves owner xp and the capture of the mc
output shows the page, so `drawn drag-held xp` finds no red pixel and fails after 1000 ms: red by reasoning, consistent with
the builder. give/key pass on base (mc owns the window inside the page), as reported. On 18795b9e the xp node is above the page
and the rect on the mc output is red: passes. The cells poll with a deadline (WITHIN_MS, default 1000, 50 ms steps, all loops
bounded), `ctl wait --timeout 20`; the sleeps are settling waits (`sleep 1` after settle), not asserts. Helpers it uses exist in
gate-lib.sh. Margin: measured 408-462 ms against 1000 ms on a quiet container; on a loaded CI runner this may flake (rv6-1 b, MINOR).
Not run here (needs full binaries; not requested).

## 4. merge-gate.sh
Pure additions: the comment step list, a description sentence and the step line `mc-handover|BIN=$repo/target/debug
tests/gate-mc-handover.sh` placed before e2e. Nothing dropped or weakened (diff reviewed, 5 lines changed, all additive).

## 5. e2e gate-tiling-apps-click
tests/gate-tiling-apps-click.sh does not reference mc/chora-mc (grep empty); the diff touches only plugins/chora-mc/src/main.rs,
a new gate script and merge-gate.sh. The scene API is unchanged, so the diff cannot affect it. The failure is pre-existing or
environmental (unproven on 4105f8d1, as the builder says).

## Findings
- MINOR rv6-1: plugins/chora-mc/src/main.rs:1792: order-dependent cover if another WM without `--output` shares the mc output
  (see 2). tests/gate-mc-handover.sh:46 `within_ms` 1000 vs ~460 ms measured: little margin on loaded CI; consider a larger
  default or a retry.
- MINOR rv6-2: no unit test for the new root order (mc root first child of ROOT after build); the gate is the only cover.
- COSMETIC rv6-3: plugins/chora-mc/src/main.rs:3416-3427: the doc comment "Keeps the regions over a scene group that was just
  created." now sits on `under_other_plugins` and `regions_on_top` (3429) lost its doc; the two comments are glued together.

## What I ran
- df -h /: 20 G avail, builds allowed.
- `cargo test -p chora-mc --locked` (target in rv-mc6/target): 125 passed, 0 failed.
- `cargo clippy -p chora-mc --locked -- -D warnings`: clean.
- Did not run tests/merge-gate.sh or the gate script.
