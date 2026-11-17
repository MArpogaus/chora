# Review: arch-plan-fable, feasibility and migration risk
Code head `29e2cd77` (chora session branch); log worktree as of 2026-11-17. Read-only review: nothing was built, run or committed.
Independent reviewer; angle: feasibility and migration risk, checked against the code and not against the plan's own claims. Production lines = lines above the trailing `#[cfg(test)] mod tests`.

## Summary

The direction holds (kit library, net before ports, moves before logic). The plan as written does not hold:
- the stage-0 order contradicts itself;
- the IO-free `kit::wm` has no path for refused transactions;
- the text invariants cannot see most of the text on screen;
- `cube.drag` needs a core change the plan does not list;
- the week costs about 42-45 %, not 33 %.

## A. Order and dependencies (Q1)

- **R1 BLOCK, 4.1/4.5, N1 N2 Z2.** The heading says "nothing else merges before N1 and N2", but the week table runs Z2 "beside" them, and Z2's gate ("`acts` unchanged") needs N2 first. Both slices touch the same code. N2 adds `Plugin::state` to the `impl Plugin` blocks (xp wm.rs:428, tiling main.rs:429, mc main.rs:806), which Z2/AR8 moves. N1 rewrites mc's test module (main.rs:3456 on, testing.rs), which AR8 moves to tests.rs. The conflicts are certain. Change: Z1, then Z2. A move is proven by `--color-moved` and the existing 40-step gate; the acts add nothing to that proof. Then N1, then N2 on the moved files. Drop "acts unchanged" from Z2.
- **R2 MAJOR, 4.1, N2 vs N3.** I9 and I10 read `boxes` (table 3.3), but they sit in N2, and `scene.boxes` arrives only in N3. Change: N2 keeps I1, I3, I4, I7, I8, I14; I9 and I10 move to N3.
- **R3 MAJOR, Z1 vs N3 and W2.** The ratchet "fails when a listed file grows". N3 adds boxes on the render path, but render.rs already has 1,858 production lines (tests start at :1859), over 1,500 and missing from the plan's list. W2 grows input.rs (1,729). Change: N3 puts boxes in a new `boxes.rs` that reuses the render placement functions, and the ratchet lets a file grow only in a merge that also extracts code from it.
- **R4 MAJOR, 4.5, throughput.** Only one gate runs at a time (merge-gate.sh:6-7). The full gate already passed 30 min at step 27 (friction.md:339). The week has 22 merges, each also gated on its lane, which is about 15 h of serial gate before any rerun, and N2 adds a step. Change: plan for at most 14 merges; batch Z1 with G1; run the per-look ports as trains.
- **R5 MAJOR, 3.2.** "`acts` replaces `doors` and shrinks `matrix`" removes coverage exactly while the looks are being ported. Change: acts run beside doors and matrix until W1b-d and A2b-d have merged. The deletion is a later slice of its own.

## B. kit::wm against the four looks (Q2)

- **R6 MAJOR, 2.2/W1, refused transactions.** The plan's `apply/act -> Vec<Change>` has no outcome path. The core can refuse an adopt: tiling main.rs:1555-1575, mc main.rs:2929-2939 ("Another window manager took it first"), construct adopt.rs:66-75. Tiling forgets a window only after the core reports back (main.rs:766, `leaves_tiling` :1873). A give is `Windows::hand`, which can fail (give.rs:68, :258). Change: make it two-phase. `Change::Adopt`, `Give` and `Close` become proposals that the look settles with `Wm::confirm(id)` or `Wm::refused(id)`. The W1 FakeCore script includes a refused attach and a refused give.
- **R7 MAJOR, 2.2, `Policy.rank: fn(&Facts) -> u8`.** Tiling's rank depends on the layout, the focus and the last raised window of the output. `arrange_output` (wm.rs:642-690) calls `tiled_rank(window, top, stacks && i == top_at)` (:1064) and `float(.., top, focused)` (:1076). Within a rank, a stable sort keeps the layout order (:618). A function pointer cannot capture the config. Change: the look supplies the order per output (`trait Policy { fn order(&self, view) -> Vec<Id> }`); `kit::wm::stack` enforces only modal above parent and held on top.
- **R8 MAJOR, 1.1, "detach is free".** mc puts a detached window of its own scene back in place (main.rs:3006-3023). xp and tiling forget the window and adopt it again, which loses its frame and desktop (xp wm.rs:786-790; tiling main.rs:657-659). Change: the book emits `Change::Detached` and the look decides. Record which behaviour is right before W1; otherwise the golden test freezes whichever one W1 picks.
- **R9 MAJOR, W1 pilot.** mc has one output, scenes keyed by app name, docks, `PARKED` and `fitting` (main.rs:2884-2955, :3012-3023), and no stack and no desks. It is the least window-manager-like look, and its pilot would fix the core types. Change: pilot on tiling, whose wm.rs is the pure model the plan itself names, with restore and focus tests; mc goes second.
- **R10 MAJOR, W1/W3/W4, restart.** Each look keeps its model in node meta. Tiling writes `window`, `current` and `tags:<output>`, with focus history and raised ranks (wm.rs:957-1045). xp writes key `xp` (restart.rs:17-40: frame, desktop, restore, z, opened, minimized). Every port that moves history, stack or desks into kit changes that format. A restart with `resume` across an update then silently drops desks and stack. Nothing catches it: tests/xp.sh:469-481 only counts windows. Change: `kit::wm::kept` with a versioned key and readers for the rc formats. Red test in every port: restore from meta written by the rc binary.
- **R11 MAJOR, 1.1/3.1/I1, construct panes are views.** A pane can show a window that another client owns (`Hold::Shown`, `Hold::Taken`, construct main.rs:254-263). It is not "mine" for the book, and if it is listed, I1 reads it as one window in two looks. Change: add `views: [{surface, owner, pane}]` to State, apart from `windows`; I1 counts owners only.
- **R12 MINOR, DeskRule.** Tiling tags are per output (`selected` on each `Output`, wm.rs:62-72). xp desktops are global groups `xp-desk-N` with one `desktop` field (desktops.rs:27, wm.rs:338). `One{count}` and `Tags{count}` cannot express the difference. Change: add `scope: Global | PerOutput`.
- **R13 MINOR, `Wm<W: Default>`.** Kept state needs `W: Serialize + DeserializeOwned`. xp's per-window data needs context at adopt: the icon cache and `places.turn` (wm.rs:1086-1092). Change: start with a plain `Wm` plus a look-side `HashMap<Id, W>`.

## C. Desks and the cube role (Q3)

- **R14 BLOCK (for F3 "designed"), 1.2/W5, `cube.drag`.** The core moves a button grab only together with a window given in the same transaction (input.rs:1146-1183 `move_grab`). It ends a given grab whose holder lacks the window (`drop_stray_grab` :1185-1213). A press through a pane stays pinned to that pane until release (output.rs:341-351); `unpin_press` frees it only for the pane's owner (:370-381). construct's drag code handles a press on a face in navigate mode only (cube.rs:797-806). `cube.drag` therefore needs a new core method: a grab hand-over without a window, with an ACL and a D-record, in an input.rs already over 1,500 lines. W5 lists only construct and xp. Change: add a core slice, "grab hand-over to a pane owner" (opus, about 3 %), before W5. Or drop the hand-over and let construct grab Super+button itself under the smallest-claim rule (ipc.rs:441-460).
- **R15 MAJOR, 1.2 and decision 3, what the user loses.**
  - xp without construct, the default xp flavor: no turn animation, no Super+drag turn, no carry-with-turn (desktops.rs:36-118), and no `turn_to` (tests/xp.sh:464-468, gate step `xp`).
  - xp with the cube on: the whole session runs on virtual outputs. Direct scanout is lost, although D67 gives "direct scanout stays" as its reason. A face cannot be screen-shared through the portal, and a virtual output has no pointer (D69). A manager whose claims are all virtual gets its keys only while the pointer is in its claim (input.rs:461-519).
  - `cube.open` starts its own xp (methods.rs:94-100, config.rs:316). An xp flavor that starts construct, which starts xp, is circular.
  - Change: keep xp/cube.rs as the flat-screen effect behind the `cube.*` client until a hosted session is measured. Decision 3 needs the owner's explicit OK as a visible change.
- **R16 MINOR, 1.2, cube API.** `cube.turn {to}|{steps}` renames the existing `{by, ms}` (methods.rs:44-51), which gate-construct-cube.sh:196, :343, :576, :584 and construct-captures.sh:107 call. `construct.cube {front, turning}` does not exist. Change: keep `by`, add `to`, and list the event as new work in W5.
- **R17 MINOR, 1.2, lazy cube.** `Desks::bind/unbind` during a turn moves client windows between real and virtual outputs. Each move sends output enter and leave and reconfigures scale and fullscreen size in the middle of the animation. Change: measure it before calling it a fallback, or drop it.

## D. The safety net (Q4)

- **R18 BLOCK, 3.3, I5/I6/I10 (I9 partly).** Most text on screen never becomes a text node.
  - xp, the assistant and notify create no `NodeKind::Text` (29 Image nodes, 0 Text); they draw their text into images with chora-draw.
  - Tiling's bar is waybar and its notices are mako, both external clients. The class I5 cites, j1-4 "No helperVolume 40%", was waybar text (review-tour-v5.md:10, flavors/tiling/waybar/helper.sh).
  - `scene.boxes` cannot see the Start menu, the taskbar, the dialogs, the helper HUD, the toasts or the tiling bar.
  - Change: every image drawer reports its text runs (`[{text, rect}]` from kit hud and chora-draw, in node meta or in `state.get`). waybar and mako stay pixel cells. Until then, label I5, I6 and I10 "text nodes only" and do not count them for F7.
- **R19 MAJOR, 3.2, stepped clock.** Only construct exports `clock.step` (methods.rs:195, plugin.rs:105). xp, tiling and mc animate on the wall clock. A stepped clock that gets no step for 2 s runs again (kit clock.rs:27-31). "Wait until quiet" is therefore real time on CI llvmpipe. Change: N1 adds kit clock to every look, or the acts run on the wall clock and read `busy` from `state.get`, without `record`.
- **R20 MAJOR, 4.1, N2 "door acts pass on the rc head".** The adapters are part of N2, so the rc head cannot pass by definition. Tours v3-v8 left these glitch classes open: if I4, I7 or I9 fire, N2 cannot merge without fixes outside its scope. Change: an `xfail` list keyed by finding id, which fails when an entry unexpectedly passes and is reported with F7.
- **R21 MAJOR, flaky checks and false alarms.**
  - I2: a buffer within 500 ms under `--jobs 3` on llvmpipe is flaky. xp floats windows off their virtual output by design (wm/float.rs), so the 32 px rule fires in pull and cube acts.
  - I7: tiling's wallpaper is swaybg, an external client, so the output is flat or black in headless gates. An xp without a wallpaper shows a flat `BACKGROUND` (desktops.rs:23).
  - I9: doors replace notify-send with a stand-in (doors/lib.sh:84), and tiling hands notices to mako. Nothing is drawn, so I9 fires on every screenshot act.
  - I14: `effects.dropped` is expected under the budget on slow renderers (D94). Deliberate disconnects (xp restart, `cube.close`, gives) would need declared events. Plugin-driven animations (the xp lamp and cube) have no duration the core knows.
  - I13: "yaw 45" is mid-turn, not "at rest". co-21 is about faces that do not share an edge, and an axis-aligned `rect` cannot show that. Change: boxes give the four projected corners.
  - I3: in two-look acts, `focus.changed` can still be in flight. xp ignores `focus.changed` without a surface (wm.rs:850-853).
- **R22 MAJOR, 2.2, `FakeCore::check`.** I4 is defined per act (the opens it declares), not per `send`. I12 needs layout and boxes, which mc's fake core lacks ("without a renderer and without layout", testing.rs:1-2). Change: `check` runs I1 and I3; I4 runs per act in `Script`.
- **R23 MAJOR, 3.1, golden test.** "Answers must not change" conflicts with slices that change behaviour: W1 claims row 2 (state kept at a give) and Phase events, W3 changes how desks are represented, W6 changes `open`. Surface ids also differ from run to run. Change: compare only the fields a slice declares untouched, with ids normalised by app_id and title. New goldens go in a separate commit, reviewed as a behaviour change.
- **R24 MAJOR, 3/4, what a port breaks that nothing catches.**
  - Restart with `resume` (R10).
  - The order of normal windows (tiling's seven ranks; max shows the last raised window).
  - Tile geometry and gaps.
  - Placement of new windows (xp `places.turn`, tiling `at`).
  - Decoration (xp `redecorate`, wm.rs:1344).
  - The grab set (tiling `grab_all`, main.rs:576: give keys, `move`, swallowed bindings).
  - Taskbar order (`opened`).
  - Launch notices that close on the token (xp wm.rs:781).
  - The magic lamp, and D58 looks kept across layouts.
  - Per-tag layout parameters.
  - Words.
  - Change: per look, add a restart act, an `arrange()` snapshot per layout, a snapshot of the core's grab table and a taskbar-order cell.

## E. Red tests (Q5)

- **R25 MAJOR, stated tests that pass on a wrong implementation.**
  - W1/W1b-d: "the same script per look" tests kit, not how the look handles a `Change`. The golden passes while the adapter still answers. Require the adapter deleted in each port and a check that `state.get == Wm::state()`.
  - W3: tests that "move with the model" are rewritten, not red, and xp-faces covers virtual outputs only. Add acts for view tag, move to tag, urgent, and an xp desktop switch on a screen.
  - A2: with only mc ported, "the same action in every look" is trivially true.
  - A3: the test reads kit's output, while xp draws its sheet as images (help.rs). A look that ignores the kit sheet passes (see R18).
  - N3's text unit test is vacuous without fonts (text.rs:619 `fonts_present`; no font means no layout, :673).
  - Cannot be red first: Z1, Z2, N1, G1 and N2. Its "break on purpose" cells need fault hooks for I1, I10 and I14 that are not scoped. The gate enforces red-first only for fix and feat commits (merge-gate.sh:25-36). Call these selftests.

## F. Sizes, models, week (Q6)

- **R26 BLOCK, 4.5, costs (sonnet lane 1-2 %, opus 2.5-4 %).**
  - N2: two opus lanes, 6-8 %. It covers the harness, 8 invariants with break cells, 4 adapters, and 20 door cells × 4 looks with the stand-ins of doors/lib.sh.
  - N3 (render path): 4-5 %.
  - Z2: 3-6 %. xp wm.rs has 4,273 production lines. mc main.rs has 3,456, and AR8's three moves leave about 3,000, still not under 1,500.
  - W1 after R6-R10: 3.5-4 %. W1b-d and A2b-d: 4-5 % each, xp included (no Action enum, keys hard-coded). G1: 1.5-2 %.
  - Total: about 42-45 % against 33 %.
  - Cut first, in this order:
    - A2b-d and W1b-d. F2 cannot close this week anyway, since notices and launching are deferred and "partly" counts as open.
    - A3 and G2.
    - Shrink N3 to surface and layer rects for I2 and I12, plus the DB14 cell as a construct gate cell.
  - Keep: Z1, Z2, N1, N2 (cut per R2), G1, W1 (tiling pilot), A1, the A2 pilot and A4 (security). That is about 31 %.
- **R27 MAJOR, 4.5, criteria that cannot close.**
  - F4 cannot be met: render.rs 1,858, input.rs 1,729, ipc.rs 1,720, chora-scene scene.rs 2,033, mc region.rs 2,272, construct cube.rs 1,642, voice chat.rs 1,536 production lines.
  - F7 as redefined needs every playbook beat and the video checker (plan-after-foundation.md:37-44, proposal-glitch-gates.md:31-32). The plan defers both to 3.4.
  - F5's "no flaky gate" is at risk from R21.
  - "A phase opens only when the previous one is met" (plan-after-foundation.md:27), so P1 is open by design.
  - Change: get the owner's written re-scope of F2, F4 and F7 for this week now.

## G. Statements about the code that are false (Q7)

- **R28 MINOR, 4.5 (lines 347-349).** Voice core.rs, agents.rs and assistant chat.rs are under 1,500 production lines (test modules at :1392, :1411, :1405). render.rs (1,858) and chora-scene scene.rs (2,033) are over 1,500 and missing from the list.
- **R29 MINOR, other false or loose citations.**
  - 1.2 "press, drag and release code exists, cube.rs:800-855": that code is navigate mode only (R14).
  - 3.2 "stepped clock": only construct has one (R19).
  - The I5 example is waybar text (R18).
  - W2 cites input.rs:433, which is the injected branch; the pointer gate is at :461-519.
  - 3.1 "chat answers through kit": chora-chat (services/voice/chat) has no kit `Plugin`.
  - The Z1 rule "above the first `#[cfg(test)]`" reads 15 lines for mc main.rs, because of `#[cfg(test)] mod audit;` at :16. Use the trailing `mod tests`.

Verdict, the three changes that matter most:
1. Re-sequence stage 0 as moves (Z1, Z2), then N1, then N2. N2 gets an xfail list and the state invariants only. The text invariants apply only to drawers that report their text runs.
2. Redesign W1 before it fixes the types: two-phase adopt, give and close, look-owned stacking order, a tiling pilot, and versioned kept meta with a restart red test in every port.
3. Cut the week to about 31 % (drop the b-d ports, A3 and G2, shrink N3). Add a core grab hand-over slice before W5. Get the owner to re-scope F2, F4 and F7 now.
