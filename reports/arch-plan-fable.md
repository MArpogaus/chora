# Architecture and refactor plan (foundation F1, arch r1)

Architect lane (Fable), 2026-11-17. Code head read: `29e2cd77` (session branch after the rc docs merge).
Read-only: nothing built or run. Facts come from grep, `wc -l` and reading; file:line refers to that head.
Starting point is reports/architecture-rc.md (ARC); its duplication map, cut breaches and kit modules
hold and are not repeated. This plan adds what ARC left open (priorities-sprint1 section 2 rows 1, 2, 6,
10), the safety net (owner 11-17 14:24) and the order. Costs are estimates in % of the week, not measured.

## 0. In brief

- **Yes: one shared window-management core, as a library `chora_kit::wm`, not a plugin.** It is a model
  without IO: events and intents go in, changes come out, the look turns changes into scene commands.
  Shared: lifecycle (ARC's `book`), focus, stacking order, desks (desktops, tags, faces), handover state,
  the list of open shell surfaces. Look policy: geometry, layout, decoration, motion, widgets.
  `chora-tiling/src/wm.rs` already has this shape (pure state, 1,716 lines, unit tests); it is the model.
- **The same model is the safety net.** Its serialized state is the answer of a new `state.get` every
  plugin exports through kit. Act contracts and invariants read it. A look that ports to `kit::wm` must
  give the same `state.get` answers for the same acts as before: that is the red test of every port.
- **Desks live in the look's model; an output is how a desk shows.** One desk can be bound to one
  output. On a screen the desks of a look share the output and one shows; in the cube each desk is bound
  to a virtual output (D69) on a face. xp already runs both ways (`Window.desktop`, wm.rs:1089, against
  one taskbar per virtual output, wm.rs:1412-1416, 1488-1492); the model makes them one.
- **One cube: construct exports the role `cube.*` (D71); looks call the role, never the plugin.** The
  cube owns faces and outputs, no window state. A drag is handed over once (`cube.drag`), not sent per
  motion. Without the role a look switches desks flat and says so in its key sheet.
- **Fluent is the store, a typed kit facade is the API** (revises D92; section 1.4).
- **Order: net first (state, acts, invariants), then moves without logic, then `kit::wm` and the ARC
  modules look by look.** The week reaches the net, the size moves, lifecycle, actions with config, the
  question card and the cut gate in report mode. Focus, desks, stack and the cube are designed here and
  built after the week, in that order.

## 1. Answers to the five questions

### 1.1 Shared WM core (question 1)

Decision: `chora_kit::wm`, a module tree in kit, used in-process by every look.

| part | shared mechanism (kit::wm) | stays look policy |
|---|---|---|
| `book` | which windows are mine: claim filter `to != NAME`, detach is free, adopt, forget, outputs, primary output, parent of a dialog, origin of a window (new with token, given with its handover state, kept over a restart) | where and how big a window shows |
| `focus` | one holder of the keyboard: a window, a shell surface of the look, nobody, or another look; history per desk; what gets the focus after a close, a desk switch, a give; reconcile with `focus.changed` of the core; the rule which look owns a pointer-gated grab | focus by click or by pointer (a `Policy` value) |
| `stack` | raise order plus a rank per window; a dialog above its parent; the held window on top; one ordered list per output | the ranks (tiling has seven, wm.rs:18-24; xp three) |
| `desks` | desk membership as a bit set, shown desks per output, stacked or bound to outputs, occupied and urgent, one event `<look>.desks` for every pager, bar and the cube | how many desks, one or many shown (xp one of four, tiling tags), names |
| `open` | the list of open shell surfaces (menu, popup, dialog, sheet, launcher), who opened each, which is modal, one Esc order | what the surfaces are and look like |
| `give` | handover state written at a give and read at an adopt (`from, size, at, maximized, fullscreen`); the drag out of the claim (kit give.rs has the calls) | where a received window lands |

Honest limit, as the planner said: mc and construct use `book`, `focus`, `open` and `give`; mc has no
overlapping windows and no desks, construct shows panes. They pass `Policy { desks: None, .. }`. The
gain for them is lifecycle, focus and the shared state, not workspaces.

Relation to ARC and D73: `kit::book` of ARC is `kit::wm::book`, the first module, unchanged in content.
A plugin `chora-stack` (D73) is later a neutral look built from `kit::wm`; the library comes first
because all four looks need it in their own process (ARC section 0: no IPC round trips, no z-order fight).

### 1.2 Cube ownership (question 2)

construct owns the cube and exports a role; `chora_kit::cube` is the client and finds the exporter with
`methods.list` (D71). No look names construct, so the cut gate holds (ARC section 3, names).

| method | does |
|---|---|
| `cube.list {}` | faces with output, manager, health; the front face; whether it turns |
| `cube.turn {to}` or `{steps}` | turns to a face; answers when accepted; `construct.cube {front, turning}` reports the end |
| `cube.drag {}` | the caller hands its running pointer drag to the cube; construct follows the pointer until the release and settles (its press, drag and release code exists, cube.rs:800-855). No motion crosses IPC |
| `cube.pull {surface}` | lifts a window off its face into the space as a pane (pull.rs today, by drag only) |
| `cube.open`, `cube.close` | exist (methods.rs) |

Carrying a window to another desk needs no cube method: the look moves the window to desk j in its
`desks` and calls `cube.turn {to: j}`.

Without the role (construct not installed or not running): the look switches desks flat (hide and show,
the cheapest desktop, D67), the pager works, the key sheet says "Next desktop", not "Turn the cube", and
the turn keys of a drag do nothing. `xp/cube.rs` (327) and the turn half of `wm/desktops.rs` (296) go.

How a look gets a cube: hosted, as today (`cube` gate step: one xp on four virtual outputs on the faces).
The xp flavor with the cube on starts construct stepped in at 1:1 (D76), with the manager found through
manager entries (ARC breach 3). Open risk, to measure before DB12 is built: a hosted look loses direct
scanout at rest. If the frame-budget gate shows a cost, the same API allows a lazy cube (desks bind to
virtual outputs only while the cube turns: `Desks::bind` and `unbind`); look code is the same either way.

### 1.3 Where desks live (question 3)

In the look's `kit::wm::Desks`, not in the scene and not in the cube. A virtual output is the way a desk
is shown on a face; whoever runs on that output with `--output` is its manager. This gives D4 (each face
its own manager) without a cube change: a face is an output, and tiling tags can bind to faces as xp
desktops do. virtual-outputs steps 8-10 follow DB12 in P3, not earlier.

### 1.4 Tools, grants, words (question 4)

- **One registry: the method table of the core.** `methods.export` already carries description, schema and
  `privileged` (kit runtime.rs:568-582), and MCP makes one tool per exported method (D34). Add tool data
  to the export: `tool: { risk: safe | ask | refuse, words: Key }`. `kit::tool::declare` is the only
  place a plugin states it. Grants stay in the core (`grants.set`); a call with `risk: ask` goes through
  `kit::ask`, the only question model. System services are a plugin `chora-system` exporting the role
  `system.*`; `kit::actions::perform` calls the role when present and the shared scripts otherwise.
  Notices are not tools: `kit::notice` is a client of the role `notify.*`. No second registry.
- **Words: Fluent files as the store, typed keys as the API** (settles backlog:167 against ARC:146).
  Reasons for the owner's wording: new languages as files without code, drafts marked unreviewed, a term
  per look (`-helper`), plurals. Reasons for ARC's wording: compile-time completeness, no heavy
  dependency. Both hold with: `kit::words::Key` (exhaustive enums per module: actions, keysheet, ask,
  look words), `Catalog::get(Key, &Args) -> String`, `.ftl` files inside each plugin crate read with
  `include_str!` (cut-clean), `fluent-bundle` only (no i18n-embed), and a gate that every key exists in
  every reviewed language plus a pseudo-locale cell. Step 1 (this week) is the facade over today's En/De
  tables, so `actions`, `keysheet` and `ask` freeze keys, not strings. Step 2 swaps the store inside
  `kit::words` only; no look changes again.

### 1.5 Sequence and rework (question 5)

- Build only on the new modules: priorities items 5, 11 (after mc-32 is verified), 12, 13, 17 and all
  xp cube work. Safe now: mc-32 verify, P1, item 19, DB14 seam cell (it becomes invariant I13).
  sh-26 and sh-27 are fixed by the `kit::ask` slice (A4) in the week, not in place.
- The WM core is not after `book` and `actions`; `book` is its first slice. `actions` runs beside it
  (other files of each look).
- Config promise: every config of the rc loads after C10 and gives the same action table. Old spellings
  (`ShowKeys`, `Help`, `VolumeUp`) stay as serde aliases for one release and are named in the release
  notes; xp gains `bindings` whose defaults are today's hard-coded keys. Gate: `tests/configs/rc/*.yaml`
  (the shipped and documented configs at the rc tag) load in a unit test per look.

## 2. Target structure

### 2.1 Crates and modules

No new crate. The allowed set for plugins stays chora-scene, chora-kit, chora-draw (D95).

```
chora-scene   + paths (runtime, config, state dirs; one private-dir check)
chora (core)  + scene.boxes (read-only) + layers (D89) + linger (S3) + grab rule fix (sh-24)
chora-kit
  wm/{book, focus, stack, desks, open, give, state}      new; section 1.1
  cube        client of the role cube.*                   new
  actions, keysheet, ask, launch, notice, settings::Common, log    as ARC section 2
  words       typed keys, catalog                         reshaped (1.4)
  tool        tool data on exports                        new, small
  testing     FakeCore, Script, invariants (feature `testing`, dev-dependency only)
  runtime     + Plugin::state(), serves state.get
plugins       looks: xp, tiling, mc, construct; shared: notify, assistant; later: system, effects,
              D73 blocks (stack, panel, desk)
tests/acts    act contracts, invariants, drivers (Python, over choractl)
tools/cut-check
```

### 2.2 APIs (signatures, not code)

```rust
// chora_kit::wm
pub struct Wm<W> { /* book, focus, stack, desks, open */ }          // W: the look's data per window
pub struct Policy { pub focus: FocusRule, pub desks: DeskRule, pub rank: fn(&Facts) -> u8 }
pub enum FocusRule { Click, Pointer }
pub enum DeskRule { None, One { count: usize }, Tags { count: usize } }
impl<W: Default> Wm<W> {
    pub fn new(name: &str, policy: Policy) -> Self;
    pub fn apply(&mut self, event: &Event) -> Vec<Change>;           // core events
    pub fn act(&mut self, intent: Intent) -> Vec<Change>;            // what the person or a call asks
    pub fn window(&self, id: &str) -> Option<&Window<W>>;
    pub fn windows(&self) -> impl Iterator<Item = &Window<W>>;       // bottom first, per output
    pub fn state(&self) -> State;                                    // the wm part of state.get
}
pub enum Intent { Focus(Id), Raise(Id), Close(Id), Minimize(Id), Restore(Id), View { output, desks },
    MoveTo { id, desks }, Give { id, to }, Open(Shell), Shut(ShellId), Escape, Bind { desk, output } }
pub enum Change { Adopt { id, origin }, Forget { id, why }, Resized(Id), Retitled(Id),
    Requested { id, request }, Outputs, Focus { from: Holder, to: Holder }, Restack { output },
    Show(Id), Hide(Id), Desks { output }, Phase { id, phase }, Shut(ShellId) }
pub enum Origin { New { token: Option<String> }, Given(Handover), Kept }
pub enum Holder { Window(Id), Shell(ShellId), Nobody, Foreign }
pub enum Phase { Opened, Closing, Closed, Minimized, Restored, Given }
pub struct State { windows, focus, desks, open, outputs }            // serde; schema in docs/api.md

// chora_kit::runtime
trait Plugin { fn state(&self) -> serde_json::Value { Value::Null } /* state.get, read-only */ }

// chora_kit::cube
pub struct Cube; impl Cube { pub fn find(conn) -> Option<Cube>; pub fn list(..) -> Faces;
    pub fn turn(.., to: Face); pub fn drag(..); pub fn pull(.., surface: &str); }

// chora_kit::testing (feature)
pub struct FakeCore; impl FakeCore { pub fn new() -> Self; pub fn plugin<P: Plugin>(..) -> P;
    pub fn send(&mut self, event: Event); pub fn scene(&self) -> &Scene; pub fn calls(&self) -> &[Call];
    pub fn check(&self, state: &Value) -> Result<(), Vec<Broken>>; } // invariants I1, I3, I4, I12

// core, read-only
scene.boxes { output? } -> [{ id, owner, kind, output, rect, ink?, opacity, clipped, z }]
```

`actions`, `keysheet`, `ask`, `launch`, `notice`, `settings::Common`: as ARC section 2. Two changes:
`System` gains `Desk(Step)` and `Cube(Turn)`, and every `System` value and every tool has a `words::Key`.

### 2.3 Effect hook for S3 (part of F3)

An effects plugin needs three things, none of them an effect API of its own (`effects` on nodes and the
foreign `update` of D58 exist, api.md:56, 291):
1. Window phases as events: `kit::wm` emits `<look>.window {surface, node, phase, rect}` for every
   `Change::Phase` (opened, closing, closed, minimized, restored, given). Lands with slice W1.
2. Pass-through: the book applies a foreign `effects` or `opacity` update on a window group
   (`node.requested`, D58) in every look alike. Lands with W1.
3. `linger` in the core: a toplevel whose group carries `linger: ms` keeps its last buffer as a ghost
   (no input) after the client is gone, until the time ends or the owner removes it; `Phase::Closing`
   starts it. Without it a burn has nothing to burn. Core slice K1, the first slice of P4.
Frame budget: `render.stats` exists; the gate cell of an effect reads it.

### 2.4 Cut gate (`tools/cut-check`, steps `ci-check` and merge gate)

ARC's four checks (deps, assets, names, packaging) plus:
- **Plugin to plugin**: `cargo metadata` shows no workspace dependency from a crate under `plugins/` or
  `services/` to another one there. Blocking from its first day (it holds today).
- **Copies** (`tools/copies.toml`): one row per mechanism that moved to kit: a pattern, the paths that
  may hold it, the count allowed outside. The count only falls; at 0 the row blocks. First rows:
  `Event::SurfaceCreated` and `to != NAME` outside kit, `"confirm.answer"`, `fn language`,
  `XDG_RUNTIME_DIR`, `UnixListener::bind` in plugin tests, `Command::Focus` outside `kit::wm`,
  per-look `enum Action` variants that `System` has. This is the "no copy left" check of F2.
- Report mode until the asset moves (ARC steps 3, 4); then blocking.

## 3. Safety net: state, act contracts, invariants (F7; first slices)

### 3.1 State (`state.get`, `choractl state`)

Every plugin answers `state.get` through kit (as `settings.get`). `choractl state` joins the answers of
all clients with core facts (focus, outputs, toplevels with owner). Shape, per plugin:
`{ windows: [{surface, title, app_id, output, desks, rect, shown, minimized, maximized, fullscreen,
floating, z}], focus: {holder, surface?}, desks: [{output, shown, count}], open: [{id, kind, modal,
by}], busy: [..] }`; assistant, chat and notify answer `{state: idle|working|asking|no_helper, shown:
[..]}`. Before a look uses `kit::wm`, a thin adapter (one new file `state.rs` per look) fills the shape
from the look's own fields. After its port the answer comes from `Wm::state()`, and the recorded answers
of the acts must not change: the golden test of the port.

### 3.2 Act contracts (`tests/acts/`)

```python
with Session(looks=["xp"]) as s:                      # headless, stepped clock (choractl record)
    s.act("open notepad", cues=[key("Super+r"), text("notepad\n")],
          opens=[], expect=Expect(new_window={"app_id": "notepad", "owner": "xp", "focused": True}))
```
`act` runs the cues, waits until the session is quiet (no running animation, no pending transaction, the
expected state or a timeout), reads `choractl state` and `scene.boxes`, compares with `expect` (a diff of
state before and after: windows new, gone, owner, focus, open, notice, helper state), then runs every
invariant. The first mismatch fails the run with the act name, the state diff and a still.
An act declares what it leaves open (`opens=[...]`); anything else left open breaks I4.

Where they run:
1. Unit level, in process: `FakeCore::check` after every `send` in every plugin test on `kit::testing`.
2. Merge gate step `acts`: per look one list of acts (the door acts of tests/doors, start a program,
   close, give to each other look, desk switch, menu open and shut, key sheet, ask the helper) plus the
   two-look lists (handover, two panes). It replaces `doors` and shrinks `matrix` to real-window cells.
3. Later: the simulated user test and the new tour call the same `act` (section 3.4).

### 3.3 First invariants (each from a glitch class of tours v3 to v8 or an open finding)

| # | invariant | read from | class |
|---|---|---|---|
| I1 | every toplevel of the core is attached, owned by one look and in that look's `windows`; no window in two looks | state, scene | lost window (co-18, sd-h1) |
| I2 | a shown window has a size, lies with at least 32 px inside its output and has a drawn buffer within 500 ms of its adopt | boxes | off-screen; window never drawn (rv5-2) |
| I3 | the focus of the core is a shown, not minimized window of the look that says it holds the focus; no second look claims it | state | focus stays with the other look (sh-23) |
| I4 | `open` after the act is `open` before plus what the act declared | state | menu left open (v3-v8, sd-h2) |
| I5 | no two text ink boxes of visible nodes on one output overlap, unless an opaque node lies between them | boxes | overprint ("No helperVolume 40%", j1-4) |
| I6 | the ink box of a text lies inside its panel | boxes | text overflow (mc long keys) |
| I7 | no real output and no shown face is black or one flat colour | screen.capture | black screen, black pane (sh-23) |
| I8 | the helper is not `working` after its answer event; one helper surface per output | state | stale chat state |
| I9 | a notice accepted during the act is drawn within 1 s, inside the output, not under the chat | state, boxes | notice not drawn (ti-22, sh-18) |
| I10 | no visible text matches a raw entry (`.desktop`, `%U`, a reverse-DNS id, a binary name) | boxes | raw launcher entries |
| I11 | a probe client with the focus receives no key of a binding that a look holds | probe log | stray key (sh-24) |
| I12 | per output: no bar or panel over a fullscreen window, the held window on top, a modal above its parent | boxes, state | bar over fullscreen (F29), cover (B5) |
| I13 | at rest the edges of neighbouring cube faces meet within 1 px at yaw 0, 45, 90 | boxes | cube seams (co-21, DB14) |
| I14 | during the act: no `effects.dropped`, `effect.failed`, client disconnect or panic line; no animation runs past its duration | events, logs | flicker, effects lost, crash |

A finding seen by eye becomes a new row before it is fixed (proposal 4). I1, I3, I4, I12 also run in
`FakeCore::check`.

### 3.4 Hooks for what follows

- Simulated user test (P2): `Session.observe()` returns state, boxes and a still for the persona model;
  `act(cues)` without `expect` runs the invariants only; a broken invariant is written as a finding row
  (`act, invariant, still`) that triage.py ranks.
- Visual checks against design/looks: `scene.boxes` gives rects, ink boxes, font size and colours; a
  `guide.yaml` per look (grid, margins, minimal contrast, taken from design/looks/*.md) lives in the
  look's flavor folder; the checker measures spacing, alignment, contrast and text fit and flags frames
  for opus vision.
- Video checker: `choractl record` writes `frames.jsonl` (frame number, time, damage hash per output);
  black, frozen and reverting frames are found from it without decoding the video.

## 4. Slices

One merge each, through tests/merge-gate.sh, red first. Pattern for every shared part: kit module with
FakeCore tests and one pilot look in one merge; then the other looks, one lane per look, in parallel.
Every port deletes the look code it replaces in the same merge and lowers a row of `copies.toml`.
"Rows" are priorities-sprint1 section 1 items; "C" are rework-audit clusters. Model: sonnet unless named.

### 4.1 Stage 0: the net and the moves (first; nothing else merges before N1 and N2)

| id | slice | red test or gate | look | size, model, cost | removes or unblocks |
|---|---|---|---|---|---|
| N1 | `kit::testing` FakeCore (from mc testing.rs, on `chora_scene::Scene`), `Plugin::state`, `state.get`, `choractl state` | kit test: a plugin on FakeCore answers `state.get`; mc tests pass on the kit FakeCore, mc testing.rs deleted | mc | M, 2 % | C9, row 18; every later slice |
| N2 | `tests/acts`: `act`, `Expect`, invariants I1, I3, I4, I7-I10, I14; `state.rs` adapter in each look; gate step `acts` with the door acts | each invariant has a cell that breaks it on purpose and must fail; the door acts pass on the rc head | all four | M-L, opus (the contract is the design), 3.5 % | C10, P2, P3, P5; F7 |
| N3 | core `scene.boxes`; invariants I2, I5, I6, I12, I13 (seam cell DB14) | core unit test on a placed text and a rotated plane; I13 red on today's seams | core, construct | M, opus (render path), 2.5 % | co-21 gate; visual checks; F7 |
| Z1 | size ratchet on production lines (AR10); test modules over 500 lines move to `x/tests.rs` | gate fails when a listed file grows | all | S, 0.5 % | C11; F4 measure |
| Z2 | step-0 moves, no logic: xp wm.rs (AR7 step 0), mc main.rs, tiling main.rs (AR8), three lanes at once | `git diff --color-moved` review; unit tests and `acts` unchanged; each file under 1,500 | xp, mc, tiling | M each, 3 % | C11; ports stop conflicting |
| G1 | `tools/cut-check` in report mode: deps (blocking), assets, names, packaging, `copies.toml` | selftest with a planted breach per check | all | S-M, 1 % | row 14, C15; F6 |

### 4.2 Stage 1: the WM core

| id | slice | red test or gate | look | size, model, cost | removes or unblocks |
|---|---|---|---|---|---|
| W1 | `kit::wm::{book, give, state}` with `Phase` events and the D58 pass-through; pilot mc | FakeCore script (create, detach, destroy, output removed, give in and out) gives the listed `Change`s; `state.get` answers of the `acts` unchanged | mc | M, opus (it fixes the core types), 2.5 % | row 2 (state kept), C7; S3 hook parts 1, 2 |
| W1b-d | port construct, tiling, xp to `book` | same script per look; `copies.toml` rows for `SurfaceCreated`, `to != NAME` fall to 0 | one each | S-M each, 3.5 % | C7; ARC row 1 |
| W2 | `kit::wm::focus` and the grab rule; core fix of the pointer-gated grab (input.rs:433) | I3 and I11 cells red first: two panes (sh-23), stray key (sh-24), keyboard-only after a stale pointer | core, then all | M, opus, 3 % | rows 3, 16; A17, A18, F1 |
| W3 | `kit::wm::desks` and `<look>.desks`; tiling first (tags), then xp (desktops, both stacked and bound) | wm unit tests of tiling move with the model; xp `xp-faces` step and pager cells unchanged | tiling, xp | M-L, opus pilot, 4 % | rows 1, 15, 16; needed by W5 |
| W4 | `kit::wm::stack`; scene layers (D89) with a `held` layer; mc root reorder (D83) goes | I12 cells; core unit test of the layer order | core, tiling, xp, mc | M, 3 % | B5, D3 of debt-rc; row 17 |
| W5 | role `cube.*` (list, turn, drag, pull) and `kit::cube`; xp drives it; `xp/cube.rs` and the turn code of `wm/desktops.rs` deleted; flat fallback | acts: turn by key, by pager, by drag from inside xp; the same acts without construct end flat; I1 and I13 hold through a drag-turn (co-18) | construct, xp | L, opus, 6 % | row 1 (DB12, co-16, 18, 20), rows 15, 16 |
| W6 | `kit::wm::open` in every look: one Esc order, menus shut at a drag or a give | I4 cells: Start menu open through a drag (sd-h2) red first | all | M, 2.5 % | rows 10, 16 |

### 4.3 Stage 2: shell mechanisms (ARC steps, reordered)

| id | slice | red test or gate | look | size, model, cost | removes or unblocks |
|---|---|---|---|---|---|
| A1 | `chora_scene::paths`, `kit::words` typed keys with `Language::of_session`, `kit::log` | unit: empty `LC_ALL` does not mask `LANG`; a private dir with a wrong owner is refused | all | S-M, 1.5 % | C14; row 8 step 1; freezes keys for A2-A4 |
| A2 | `kit::actions` with `settings::Common` (C10); pilot mc | one YAML binding gives the same `System` action in every look; rc configs load | mc | M, 1.5 % | rows 4, 10, 12; C2, C12 |
| A2b-d | port construct, tiling, xp (xp gains `bindings`) | same test per look; `acts` door list unchanged | one each | S-M each, 3 % | C2, C3 base |
| A3 | `kit::keysheet` (needs A2) | one binding has one spelling and one wording in all four sheets | all | S-M, 1.5 % | row 10; C3 |
| A4 | `kit::ask` the only question model in mc and construct | a prompt with a bidi character shows sanitized; a press inside 400 ms is refused; buttons follow the language | mc, construct | S-M, review by opus (security), 1.5 % | row 5 base; sh-26, sh-27; C6 |
| A5 | `kit::notice`, chora-notify the only server, zbus out of kit; mc slot | I9 in every look; `cargo tree -p chora-kit` without zbus | all | M, design pass for mc, 3 % | rows 6, 11; C4 |
| A6 | `kit::launch` (D87 revised: pidfd on the unit's main pid) | FakeCore: exit before a window gives `Exited` at once; no status file | xp, tiling | M, 2 % | row 12; C13 |
| A7 | chora-assistant the only helper presenter, state words in one place | I8 in mc and construct; one state machine test | mc, construct | L, opus design pass, 5 % | rows 5, 9, 11; C5 |
| G2 | asset moves and `look.theme` event (ARC steps 3, 4); cut gate blocking | cut-check has no breach; a fifth fake look gets a costume | assistant, tiling | S-M, 1.5 % | row 14; C15 |
| A8 | `kit::words` store on Fluent; pseudo-locale cell; key gate | every key in every reviewed language; pseudo-locale shows no English literal in the acts | all | M, 2.5 % | row 8; C12 backlog |
| K1 | core `linger` (2.3) | core unit test: a destroyed toplevel with `linger` stays drawn and takes no input | core | M, opus, 2.5 % | S3 (P4) |
| T1 | `kit::tool` data on exports; MCP reads `risk`; `chora-system` skeleton with one service | an `ask` tool called by MCP shows the `kit::ask` card | voice, assistant | M, 3 % | rows 7, 9 |

Later, not sliced here: D73 blocks on `kit::wm` (row 17), split packages (ARC step 12), AR5 pointer part
(kit `dispatch` drops injected keys only, runtime.rs:482), AR11, AR14, the remaining files over 1,500.

### 4.4 Rank by work unblocked

N1, N2 (every slice; F5, F7) > W1 (7 rows with W2-W4) > A2 (4 to 5 rows) > W2 > W3 then W5 (4 rows, the
owner's cube) > A4 > A5 > A3 > W4 > W6 > A6 > A7 > T1 > A8 > K1.

### 4.5 What the week reaches (to Sat 03:59; about 33 % after this lane)

| day (UTC) | merges | cost |
|---|---|---|
| Tue-Wed | N1, then N2; beside them Z1, Z2 (three lanes), G1 | 10 % |
| Wed-Thu | N3; W1 (mc); A1; A2 (mc) | 8 % |
| Thu-Fri | W1b-d; A2b-d; A4 | 8 % |
| Fri | A3; G2; docs (F8), debt recount (F4), coverage of kit (F5) | 5 % |
| reserve | 2 %; first cut if short: A3, then G2 | |

Against the criteria: F1 this plan. F2 this week: lifecycle (W1), actions and config (A2), key sheet
(A3), question card (A4); notices (A5) and launching (A6) go to the backlog for P2/P3, named, not hidden.
F3: design here, W1 merged with the S3 hook parts 1 and 2; W2 to W6 and K1 after the week. F4: the
ratchet and the three look files; the other files over 1,500 production lines (core input.rs and ipc.rs,
voice chat.rs, core.rs, agents.rs, mc region.rs, assistant chat.rs, construct cube.rs and more) stay
open: F4 in full costs about 5 % more and does not fit. F5: FakeCore tests for every moved module;
coverage measured Friday. F6: cut gate in report mode, blocking for dependencies. F7: N2 and N3.
W2 (focus) is the first slice after the week; it is not in the week because it needs a core input change
and a design review, and a late core change with no reserve is the wrong risk.

## 5. Proposed decision records (the PM files them)

1. **WM core.** Window-management mechanism (lifecycle, focus, stacking order, desks, open shell
   surfaces, handover state) is the library `chora_kit::wm`, a model without IO; a look keeps geometry,
   layout, decoration and motion. Cost: every look ports; kit grows.
2. **Desks.** A desk is a set of windows in the look's model; it shows on the look's output or is bound
   to an output. A cube face is a virtual output; the cube holds no window state. Refines D67, D76.
3. **Cube role.** construct exports `cube.*` (list, turn, drag, pull, open, close); looks reach it as a
   role (D71) through `chora_kit::cube`; without it they switch desks flat. `chora-xp` loses its cube.
   Replaces the last sentence of D73 on the XP cube. Cost: a look with a cube runs hosted by construct.
4. **State contract.** Every plugin exports `state.get` through kit; `choractl state` joins them; act
   contracts and invariants read state and `scene.boxes`, not pixels, except I7.
5. **`scene.boxes`.** A read-only core method that lists the placed rectangle and the ink box of the
   visible nodes of an output.
6. **Shared first.** A feature or fix that touches two looks starts as a kit function with a test on
   `kit::testing`, then one look, then the others; moves without logic come before logic; `copies.toml`
   only falls.
7. **Tools.** The method table is the only tool registry; an export carries `risk` and a word key; grants
   stay in the core; `risk: ask` uses `kit::ask`; system services are the role `system.*`.
8. **Effect hook.** Window phases as `<look>.window` events, foreign effect updates applied by
   `kit::wm` (D58), and `linger` in the core. No effect API beyond `effects` on nodes.
9. **D92 revised.** Typed keys in `kit::words`; Fluent files per plugin as the store (`fluent-bundle`
   only); the tables stay until the store swap (A8); a term per look.
10. **D87 revised.** pidfd on the main pid of the unit that `choractl run` started, checked against the
    unit state; no `waitid` (D42: `systemd-run` is the parent). As ARC section 9, D1.
11. **D89 kept, extended.** Layers background, normal, held, overlay, lock; the held layer replaces the
    drag-layer question of ARC D3.
12. **D93 kept, extended.** Old spellings load as aliases for one release; the rc configs are a test.
13. **D95 amended.** The cut gate also blocks a plugin-to-plugin dependency and counts copies; managers
    are found through entries under `/usr/share/chora/managers/`.

## 6. Limits

- Nothing was built or run; sizes and costs are judgement. The cost of a hosted cube at rest is unknown.
- I did not read construct `panes.rs`, the voice crate or the assistant in depth; A5 for mc and A7 keep
  ARC's "design pass" mark for that reason.
- `Policy.rank` as a function and `Wm<W>` as a generic are the proposal; the W1 lane may find a simpler
  shape and should say so before it merges.
