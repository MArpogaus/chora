# Window handover between window managers

The gap matrix of charter item 19 and backlog item 13. Read on 2026-11-09 at branch
`claude/autonomous-dev-session-3w-i33pg7`. File references are `path:line` in the chora
repository.

## 1. The mechanism today

**Core.** One command moves a window: `remove` with `to` (D55).

* `scene.rs:436-451`: the giver must own the surface, and `to` must hold a claim. The surface
  leaves the tree, the core reserves it for `to` and emits `Effect::Reserved`.
* `scene.rs:1148-1171`: `remove_subtree` resets all props of the surface (`scene.rs:1164`).
  Opacity, effects, meta and transform go.
* `scene.rs:659-664`: an `attach` of anyone but `to` fails.
* `state.rs:692-700`: the core drops the keyboard focus of the surface and sends
  `surface.detached` with the surface id alone, only to `to`. No `from`, no size, no state, no
  drop point.
* `state.rs:151`, `state.rs:557-578`: `to` has 5 s. Then the giver gets it back, then the window
  manager of a new window, then the core shows it ownerless (`state.rs:583-625`). `execute_gone`
  does the same when `to` disconnects. Unit tests: `ipc.rs:2874`, `ipc.rs:2920`.
* `scene.rs:200-213`: `claimant_at`, the owner of the smallest claim under a point. The core
  uses it for pointer bindings (`ipc.rs:395`) and new toplevels (`scene.rs:346-361`,
  `handlers.rs:203`).

**chora-kit** (`crates/chora-kit/src/give.rs`, 109 lines).

* `give.rs:14`, `give.rs:31-42`: the name and the privileged export of `window.give` with
  `surface` and `to`.
* `give.rs:53-57`: `Conn::claimant_at`, the drop target, self excluded.
* `give.rs:67-82`: `Conn::give`, a `remove` with `to`; it refuses a give to itself.

That is all the kit shares. Each plugin writes its own dispatch, its own bookkeeping after a
give, its own drag and its own adoption.

**Callers.** `choractl give TO [--surface ID]` (`choractl/src/main.rs:1081-1098`) finds the
owner of the focused window and calls its `window.give`. The MCP server turns every export into
a tool `CLIENT.METHOD` (`mcp.rs:411-424`), so `tiling.window.give`, `xp.window.give` and so on.
Voice reaches the same tools through `chora_mcp` and asks before each privileged one
(`services/voice/src/tools.rs:132-147`).

## 2. Matrices

Legend. **G** works, a gate checks it (named). **B** works, only battle W14 checks it; W14
records findings and is no gate. **U** works through the shared code path, nothing checks it.
**P** partial. **M** missing.

| Pair | Key binding | Mouse drag | `window.give` | `choractl give` | MCP tool | Voice |
|------|-------------|------------|---------------|-----------------|----------|-------|
| tiling to xp | P1 | G handover 4, 7, 8 | G handover 2 | U | U | P3 |
| tiling to mc | P1 | U (W14 6, no check) | B | B (W14 3) | U | P3 |
| tiling to construct | P1 | U (W14 3, logs only) | B | U | U | P3 |
| xp to tiling | M | G handover 1 | G handover 7, 8 | U | U | P3 |
| xp to mc | M | U (W14 3, logs only) | B | G handover 3 | U | P3 |
| xp to construct | M | U | U | U | U | P3 |
| mc to tiling | P1 | M | G handover 4 | U | U | P3 |
| mc to xp | P1 | M | B | U | U | P3 |
| mc to construct | P1 | M | B | U | U | P3 |
| construct to tiling | P2, G construct-windows 3 | G construct-windows 5 | G construct-windows 3 | U | U | P3 |
| construct to xp | P2 | U | B | U | U | P3 |
| construct to mc | P2 | G construct-windows 5 | B | U | U | P3 |

Evidence for the codes:

* **P1.** tiling and mc have no give action (`chora-tiling/src/config.rs:51-102`,
  `chora-mc/src/config.rs:32-45`). Only a hand-written `run: choractl give xp` can do it. No test.
* **xp key M.** Its keys are a fixed table without a give (`chora-xp/src/wm.rs:142-160`).
* **P2.** construct: Tab, `g`, Return gives only a window without a visible pane
  (`main.rs:2400-2403`, `main.rs:2460-2473`). The gate moves the pane behind the camera first
  (`gate-construct-windows.sh:390`).
* **mc drag M.** mc has no drag code and grabs no pointer binding.
* **Drags that work.** tiling drops outside its outputs (`main.rs:598-627`), xp drops a title
  bar outside its bars (`wm.rs:2110-2115`, `wm.rs:2184-2201`), construct drops a pane on a
  plugin pane or off its output (`main.rs:1061-1097`). All three use `claimant_at`.
* **U for choractl and MCP.** Both end in the same `window.give` as the tested method. MCP needs
  the grants `methods.call` and `methods.privileged`, and the agent must look up the owner
  first.
* **P3.** Voice can call `OWNER.window.give` and confirms it. Nothing tests it.
  `prompt.md:38-39` tells the model that switching the window manager needs a restart of chora.
  `prompt.md:48` names give only as a thing not to do.
* **Receivers.** mc receives only with `--manage-windows` (`chora-mc/src/main.rs:94-98`,
  `main.rs:1335`). An overlay construct has no claim (`main.rs:1337`), so the core refuses a
  give to it (checked in construct-windows step 2).
* `gate-construct-windows.sh` fails today with exit 143 after its last ok line (baseline.md).
  Another agent fixes it; its G cells count once it is green.

What survives a move, per receiving window manager:

| Receiver | Position | Size | Focus | Fullscreen, maximized | Tag, desktop | D58 look | D72 token |
|----------|----------|------|-------|-----------------------|--------------|----------|-----------|
| tiling | own layout, on the current output of tiling, not the drop output (`wm.rs:217-221`) | tiled; the old size counts only in a floating layout (`main.rs:731-748`) | yes (`main.rs:750`), gate checks | reset (`wm.rs:226-228`) | shown tags of the current output | lost (`scene.rs:1164`), tiling `style` instead (`main.rs:751`) | not carried |
| xp | cascade, drop point ignored | the client keeps its last size; no clamp to the work area at the adopt | yes (`wm.rs:765`, `wm.rs:834-854`), gate checks | maximized reset by `restyle` (`wm.rs:857-864`, `window.rs:174-190`); xp has no fullscreen | current desktop (`wm.rs:754`) | lost; xp keeps the look on the frame group, which goes with the give | not carried |
| mc | its scene page, by design | mc's region size | yes (`main.rs:2199`), also for a give by an agent | n/a | a new transient scene by app id | lost; mc does not answer `node.requested`, so a later `update` fails | not carried |
| construct flavor | a pane in the view direction | resized to the whole view (`main.rs:1472`, `main.rs:1512`) | no; no `focus` at the adopt, a fly-in is needed | n/a | n/a | lost; no `node.requested` | not carried |

No window manager remembers a size from before. xp to tiling to xp returns the tile size to xp.
An open dialog stays with the giver, since only the one surface moves and the scene does not
know toplevel parents (`node.rs:204-218`). The token is single use, so not carrying it is right.

## 3. Extensibility

**A new window manager today** must, in its own code:

1. set `claim` on its root group, or it cannot receive;
2. subscribe to `surface.detached`, attach the surface and focus it;
3. export `give::method()` and handle it: check the window is its own, call `conn.give`, forget
   the window, relayout;
4. for a drag, call `claimant_at` at the release outside its claim.

That is 40 to 80 lines, written four times in four shapes (`chora-tiling/src/main.rs:569-627`,
`chora-xp/src/wm.rs:2184-2213`, `chora-mc/src/manage.rs:130-150`,
`chora-construct/src/main.rs:2548-2592`).

**A new input path today** must find the surface and its owner and call `window.give` of the
owner through `methods.call`, with the privileged grant. Each path repeats the owner lookup of
`choractl give`.

**Through chora-kit, a few lines each.**

* `chora_scene::give_target(nodes, focused, Target)`: a pure function beside `claimant_at`.
  `Target` is a client name, `next` or `previous`; `next` walks the claims sorted by x, then y,
  and wraps. choractl, the MCP server and the kit share it.
* `chora_kit::give::Windows`, a trait of two functions: `has(surface)` and `forget(surface)`.
  `Runtime::window_give(plugin, params)` answers `window.give` for any plugin. The plugin adds
  one match arm.
* `chora_kit::give::focused(conn, Target)`: gives the focused window, whoever owns it, through
  its owner. A window manager binds it as the action `give: next` or `give: NAME`. A key binding
  has one owner (D55), so the window manager that gets the grab serves all of them, and the
  others fall back as xp does (`wm.rs:140-160`).
* `chora_kit::give::Drag`: press on an own window with the shared `Super+BTN_LEFT`, release
  outside the own claim, give to the claimant.
* `Conn::give` also writes `meta.give` (see gap 5), and `Conn::handover(surface)` reads it back
  as `Handover {from, size, at, maximized, fullscreen, look}` at the adopt.

Then a new window manager is: a claim, `impl Windows`, one arm for `window.give`, and an adopt
that reads `Handover`. A new input path is one call of `give_target` and one `window.give`. Each
one adds a row or a column to the table of the guard (section 5).

## 4. Gaps, ranked by what a user hits first

1. **No key moves a window** in xp, tiling or mc; construct gives only hidden panes. Fix: the
   kit action `give` with `next`, `previous` or a name; default `Super+Shift+o` (free in tiling,
   mc and xp) for `next`; in construct `g` also on the pane flown into. Test: the key column of
   the guard, all 12 pairs.
2. **Voice and MCP need two steps and the prompt misleads.** Fix: an MCP builtin `give_window
   {to, surface?}` on the code of `choractl give` with `give_target`; `choractl give next`; in
   `prompt.md` replace line 38-39 with "With several window managers, move a window with
   give_window". Test: the MCP and voice columns of the guard.
3. **mc cannot give by mouse, and construct receives badly**: the window fills the view and gets
   no focus. Fix: kit `Drag` in mc; construct adopts at the window size clamped to the view and
   flies to it, as `window.show` does. Test: the drag row of mc; the construct state checks of
   the guard.
4. **A drop lands elsewhere.** tiling puts the window on its current output, xp cascades it.
   Fix: `meta.give.at` holds the drop point; tiling takes the output under it, xp places the
   frame there inside its work area. Test: the guard drops on the second half of an output and
   checks the output and the side.
5. **State is lost**: maximized, fullscreen, the floating size on a round trip, the D58 look.
   Fix: the core keeps the meta key `give` at a `remove` with `to` and resets the rest (3 lines
   at `scene.rs:436-451`); kit `give` writes `{from, size, at, maximized, fullscreen, look}`;
   tiling keeps the received size as the floating size and passes it on; receivers apply what
   fits. The look comes from the kit pass-through of D58 (roadmap X6). Test: a unit test in
   `chora-scene` (meta `give` survives, opacity does not); guard checks xp 800x600 to tiling to
   xp is 800x600, and maximized in xp stays maximized in tiling.
6. **A dialog stays behind.** Fix: the core moves the child toplevels of the same owner with a
   `remove` with `to`. Test: a core unit test, and a guard step with a probe window and its
   dialog.
7. **Too few checks.** 5 of 12 pairs are in a gate; MCP and voice are in none. Fix: the guard of
   section 5, first.
8. **The docs disagree on new windows.** `api.md:249-253` and the code (`scene.rs:340-361`) use
   the focus as rule 2; D55 and D72 say the focus does not decide. Not a move, but it decides
   where a window starts. Fix: one decision, then code or docs. Test: a unit test of
   `manager_of_new`.

## 5. The guard

One script, `tests/gate-handover.sh`, grows into the matrix. The current nine steps stay as its
first part.

* **Session.** Headless, GLES on the software driver. Four outputs of 1280x800 side by side: the
  construct flavor on the first (it starts first, as in W14), then `chora-xp --output`,
  `chora-tiling --output` and `chora-mc --manage-windows --output`. One more short part runs the
  construct as an overlay and expects the refusal.
* **Windows.** One foot with server decorations, one foot with client decorations for the title
  bar drag of tiling, and a probe with a dialog for gap 6.
* **Table.** At the top, one line per cell: pair, path, expected `pass` or `missing`. The script
  runs every cell. A missing cell must refuse or do nothing, and no window may get lost. The
  gate fails when a cell differs from the table, in both directions, so a fix must flip its
  line. At the end it prints the matrix.
* **Each move checks**: the owner; every surface attached, visible and of nonzero size
  (`settled` of today); the focus with the receiver; then the state columns that apply: output
  of the drop point, size, maximized.
* **Paths.** Key: `ctl inject` with a shortcut. Drag: injected pointer moves; xp by title bar,
  tiling and mc with Super, construct with Super on the pane centre. `window.give`: `ctl call`.
  `choractl give`: with and without `--surface`. MCP: `tests/mcp-client.py` gets a mode `give`
  that calls `give_window` and, until then, `OWNER.window.give`. Voice: the real `chora-voice`
  with the fakes of `tests/battle/w16.sh` (`w16-fakes.py servers`): grant `voice-mcp
  methods.call methods.privileged`, set `allow_methods` to the gives, then per cell `tool
  give_window {...}`, `reply "Done."`, `say "move it to xp"`, wait for `llm: stop`. One cell
  without the allow answers the question by the button `mc-answer-yes`.
* **Time.** 12 pairs by 6 paths is 72 moves. At about 1 s each, plus 12 voice turns of about 3 s
  and 15 s of start, the gate takes about 2.5 minutes. It stops at 4 minutes. Waits are 10 s
  only on failure.
* **Manual cells.** A real mouse across outputs of different scale; a real ASR and LLM, since
  the fake proves only the plumbing; construct WM panes and drop to give (K7, K8), which need
  virtual outputs; a tilted pane (K5); X11 windows where `xwayland-satellite` is missing. Nested
  sessions never exchange windows (D74), so they are no cell.

## 6. Steps

Each step is one commit with its test. The guard comes first, so each later step flips table
lines.

1. Guard with the table of today, including the missing cells. Test: it passes in under 4
   minutes, and a forced wrong line fails it.
2. `give_target` in `chora-scene`; `choractl give next|previous`. Test: unit tests for the order
   and the wrap; guard `choractl give` cells.
3. MCP builtin `give_window`. Test: a unit test of the tool list; guard MCP column.
4. Voice prompt line. Test: guard voice column with the w16 fakes.
5. Kit `Windows` and `Runtime::window_give`; the four plugins use them with no change of
   behavior. Test: the guard stays green.
6. Kit action `give` and `Super+Shift+o`: tiling and mc settings, the xp key table, construct
   `g` on the flown pane. Test: guard key column.
7. Kit `Drag`, used by mc. Test: guard drag row of mc.
8. Construct adopts at the window size and flies to it. Test: guard construct size and focus
   checks.
9. Core keeps meta `give`; kit writes and reads `Handover`. Test: the `chora-scene` unit test.
10. tiling and xp use `Handover`: drop output and point, size round trip, maximized and
    fullscreen. Test: guard state checks.
11. D58 look through the kit pass-through (X6) into `Handover.look`. Test:
    `gate-0.8-tiling-look.sh` gets a give to xp and back.
12. Dialogs move with their parent. Test: core unit test and guard step.
13. Decide the focus rule for new windows (gap 8). Test: unit test.
