# Architecture and refactor plan (foundation F1, arch r1)

Architect lane (Fable), 2026-11-17. Code head read: `29e2cd77` (session branch). Read-only: nothing built
or run; facts come from grep, `wc -l` and reading; file:line refers to that head. Starting point is
reports/architecture-rc.md (ARC): its duplication map, cut breaches and kit modules hold and are not
repeated. Revised once after two independent opus reviews (reports/review-arch-plan-risk.md, R-numbers;
reports/review-arch-plan-fit.md); section 6 says what changed and what I did not take. Costs are
estimates in % of the week. "Production lines" are the lines above the trailing `mod tests`.

## 0. In brief

- **Yes: one shared window-management core, a library `chora_kit::wm`, not a plugin.** A model without
  IO: events and intents in, changes out; the look turns changes into scene commands and settles them.
  Shared: lifecycle (ARC's `book`), focus, the open shell surfaces, desks, handover state, kept state
  over a restart. The look keeps geometry, layout, the stacking order, decoration and motion.
  `chora-tiling/src/wm.rs` (1,206 production lines, pure state, unit tests) has this shape and is the pilot.
- **The same model is the safety net.** Its state is the answer of a new `state.get` that every plugin
  exports through kit. Act contracts and invariants read it; a port to `kit::wm` must leave it unchanged.
- **Desks live in the look's model; an output is how a desk shows.** On a screen the desks of a look share
  the output; on a cube each desk is bound to a virtual output on a face (D69). xp has both today
  (`Window.desktop`, wm.rs:1089; one taskbar per virtual output, wm.rs:1412-1416).
- **One cube: construct exports the role `cube.*` (D71); looks call the role, never the plugin.** Whether
  xp then runs hosted on construct's outputs is not settled by reading: two measurements and the owner's
  choice come first (1.2). Until then `xp/cube.rs` stays.
- **Words: D92 holds (kit tables); typed keys are the API, so the store can change in one module** (1.4).
- **Order: moves without logic, then the net (state, acts, invariants), then `kit::wm` and the ARC modules
  look by look.** The refactor does not fit the week. The week reaches the moves, the net for state, the
  lifecycle pilot, actions with config in one look, the question card and the cut gate in report mode
  (about 31 %). The ports, focus and the open-surface list must merge before the simulated user test.

## 1. Answers to the five questions
### 1.1 Shared WM core (question 1)
| part | shared mechanism (kit::wm) | stays look policy |
|---|---|---|
| `book` | which windows are mine: claim filter `to != NAME`, adopt and forget as proposals the look settles, outputs, primary output, parent of a dialog, origin (new with token, given with handover state, kept over a restart) | where and how big a window shows; what a detached own window does (mc puts it back, main.rs:3006-3023; xp and tiling adopt it anew) |
| `focus` | one holder of the keyboard: a window, a shell surface, nobody, another look; history per desk; reconcile with `focus.changed`; which look owns a pointer-gated grab | click, pointer or explicit focus; whether the pointer may focus now (tiling: not in magnifier, config.rs:46-48); who gets the focus after a close or a switch (hook) |
| `open` | the open shell surfaces (menu, popup, dialog, sheet, launcher), who opened each, which is modal, one Esc order, shut at a drag or a give | what they are and look like |
| `desks` | membership as a bit set; shown desks, global or per output; stacked or bound to outputs; occupied, urgent; one event `<look>.desks` for pager, bar, cube | count, one or many shown, names, data per desk (tiling layout per tag, wm.rs:55-58) |
| `stack` | two rules only: a modal above its parent, the held window on top; the scene layer of the look root (D89) | the order of the windows of an output (tiling ranks depend on layout, focus and last raise, wm.rs:642-690) |
| `give`, `kept` | handover state written at a give and read at an adopt; restart state in node meta under one versioned key, with readers for today's keys (tiling wm.rs:957-1045, xp restart.rs:17-40) | where a received window lands |

Honest limit: mc and construct use `book`, `focus`, `open`, `give` and the shared state. mc has no
overlapping windows and no desks; construct shows panes, some of them views of windows another client
owns (`Hold::Shown`, main.rs:254-263), and depends on depth order and `placed` (D84), which stay its own.
Faces are outputs of the cube, not desks of construct.

Relation to ARC and D73: `kit::book` of ARC is `kit::wm::book`. A plugin `chora-stack` (D73) is later a
neutral look built from `kit::wm`; the library comes first because every look needs it in its own process.

### 1.2 Cube ownership (question 2)
construct owns the cube and exports a role; `chora_kit::cube` finds the exporter with `methods.list`
(D71). No look names construct, so the cut gate holds.

| method | does |
|---|---|
| `cube.list {}` | faces with output, manager, health; the front face; whether it turns |
| `cube.turn {by, ms}` (exists, methods.rs:44-51) plus `{to}` | turns; new event `construct.cube {front, turning}` at the end |
| `cube.pull {surface}` | lifts a window off its face into the space as a pane (pull.rs, by drag only today); `cube.open`, `cube.close` exist |

Carrying a window to another desk needs no method: the look moves it to desk j and calls `cube.turn {to: j}`.
Onto a face of another look it is `window.give` plus `cube.turn`; I1 must hold across it.

Owner 11-17 10:35: the cube lives in construct, and xp still turns it "like before", by desktop switch
and by mouse, without a full step out. Two ways meet that; reading the code cannot choose:
- **Hosted** (exists: gate step `cube`, one xp on four virtual outputs). The session starts construct with
  xp as its cube manager, stepped in at 1:1 (D76); xp never starts construct. Known costs: no direct
  scanout at rest, a face is not shared through the portal, a manager whose claims are all virtual hears
  its keys only while the pointer is in its claim (input.rs:461-519), and when construct dies its outputs
  end (D69) and the windows come back through the reset.
- **In place**: xp keeps its flat-screen turn but takes geometry and turn from kit (`geometry::Cube`,
  which construct uses) and reports through `cube.*`. One cube code, two hosts; the cube does not "live in
  construct" for an xp without construct.
Mouse turning: construct holds the turn grab itself while it hosts (no motion crosses IPC, no new core
method). If the smallest-claim rule (ipc.rs:441-460) does not give it the press, a core slice "grab
hand-over without a window" is needed (R14: `move_grab` moves a grab only with a given window,
input.rs:1152). A lazy cube (desks bound to outputs only during a turn) is dropped unless measured (R17).
The first cube slice (WM7a) measures the hosted session: frame cost at rest, keys with the pointer outside
the claim, the grab, construct killed mid-turn, the return to 1:1. Then the owner chooses; I recommend
hosted if the numbers hold, because only then does the cube live in construct. Without the role every
look switches desks flat and its key sheet says "Next desktop". tiling on faces drives the cube through
the same `System::Cube` action; mc has one page per output and only calls `cube.turn` (voice, remote).

### 1.3 Where desks live (question 3)
In the look's `kit::wm` model, not in the scene and not in the cube. A virtual output is how a desk shows
on a face; whoever runs on that output is its manager, so a face can run another look (D4) once use and
navigate modes exist (K4, K12; not designed here). virtual-outputs steps 8-10 follow the cube in P3.

### 1.4 Tools, grants, words (question 4)
- **One rule, two sources of tools.** In-session tools are exported methods: `methods.export` already
  carries description, schema and `privileged` (runtime.rs:568-582), MCP makes a tool per method (D34);
  an export gains `tool: { risk: safe | ask | refuse, words: Key }` as the plugin's default. External
  tools are the `Source` list of the voice service (C0). Both resolve "ask first?" in one function,
  `kit::tool::asks(tool, user)`, where the user's `confirm` list of voice (config.rs:273) overrides the
  default. Grants (may it happen at all, design/grants.md) stay in the core and are a separate question;
  `kit::ask` is the only question model and gains the answer `always` for grant questions (grants.md:
  112-120). System services are a plugin with the role `system.*`; `kit::actions::perform` calls the role
  when present, the shared scripts otherwise. The settings core (backlog N2) is `settings::Common` plus
  the `system.*` doors plus an event per changed setting; its applet tree is a later plugin.
- **Words.** The owner chose kit tables on 11-17 04:38 (D92), after his Fluent note of 11-16 (C12); I keep
  that. What both need is the same: `kit::words::Key` (exhaustive enums per module), `Catalog::get(Key,
  &Args)`, a term per look, a gate against literal strings, a pseudo-locale cell, a language switch
  through `settings.set`. `actions`, `keysheet` and `ask` freeze keys, not strings. Fluent files loaded
  at run time are then a change inside `kit::words` only, at a third language or the first plural (owner).

### 1.5 Sequence and rework (question 5)
- Build only on the new modules: priorities rows 5, 11, 12, 13, 17 and all cube work. Safe now: mc-32
  verify (a haiku lane this week), P1, row 19. sh-26 and sh-27 are fixed by KIT4, not in place.
- `book` is the first slice of the WM core; `actions` runs beside it.
- Config promise: every shipped and documented config of head 29e2cd77 loads after C10 and gives the same
  action table. Old spellings load as serde aliases for one release, named in the release notes; xp gains
  `bindings` whose defaults are today's keys. Gate: `tests/configs/29e2cd77/*.yaml` in a unit test per look.
- **Before the simulated user test (P2)**: the ports, focus (WM2) and open surfaces (WM3) merge first.
  P2 findings in areas a later slice replaces (desks, stack, cube, notices, helper) are filed as known-red
  rows tied to that slice and are not patched in place (owner 10:43).

## 2. Target structure
### 2.1 Crates and modules
No new crate; plugins may use chora-scene, chora-kit, chora-draw (D95).

```
chora-scene   + paths (runtime, config, state dirs; one private-dir check)
chora (core)  + scene.boxes in a new boxes.rs (read-only) + layers (D89) + linger (S3) + grab rule (sh-24)
chora-kit     wm/{book, focus, open, desks, stack, give, kept, state}    new (1.1)
              cube (role client), tool (tool data, asks)                  new
              actions, keysheet, ask, launch, notice, settings::Common, log   as ARC section 2
              words (typed keys, catalog), testing (FakeCore, Script; feature, dev only)
              runtime + Plugin::state(), serves state.get; clock in every look (stepped runs)
chora-draw    text drawers return their runs (text, rect, size, colours)
plugins       looks; notify, assistant; later system, effects, D73 blocks (stack, panel, desk)
tests/acts    act contracts, invariants, known-red list (Python, over choractl); tools/cut-check
```

### 2.2 APIs (signatures, not code)
```rust
// chora_kit::wm: the look keeps its own data per window and per desk in maps keyed by id
pub struct Wm { /* book, focus, open, desks, kept */ }
pub trait Policy {                                   // the look implements it; it sees the model
    fn focus_rule(&self, view: &View) -> FocusRule;                 // Click | Pointer | Explicit
    fn focus_after(&self, why: Why, view: &View) -> Option<Id>;     // close, minimize, desk switch, give
    fn order(&self, output: &str, view: &View) -> Vec<Id>;          // bottom first; kit adds the 2 rules
    fn desks(&self) -> DeskRule;                                    // None | One | Tags, Global | PerOutput
}
impl Wm {
    pub fn new(name: &str) -> Self;
    pub fn restore(&mut self, nodes: &[Node]) -> Vec<Change>;       // kept meta, versioned
    pub fn apply(&mut self, event: &Event, policy: &dyn Policy) -> Vec<Change>;
    pub fn act(&mut self, intent: Intent, policy: &dyn Policy) -> Vec<Change>;
    pub fn confirm(&mut self, id: &str) -> Vec<Change>;             // the core took the commands
    pub fn refused(&mut self, id: &str) -> Vec<Change>;             // another manager was first; a give failed
    pub fn state(&self) -> State;
}
pub enum Intent { Focus(Id), Close(Id), Minimize(Id), Restore(Id), View { output, desks },
    MoveTo { id, desks }, Give { id, to }, Hold(Id), Drop(Id), Open(Shell), Shut(ShellId), Escape,
    Bind { desk, output } }
pub enum Change { Adopt { id, origin }, Detached(Id), Forget { id, why }, Resized(Id), Retitled(Id),
    Requested { id, request }, Outputs, Focus { from: Holder, to: Holder }, Restack { output },
    Show(Id), Hide(Id), Desks { output }, Phase { id, phase }, Shut(ShellId), Keep(Id) }
pub enum Origin { New { token: Option<String> }, Given(Handover), Kept }
pub enum Holder { Window(Id), Shell(ShellId), Nobody, Foreign }
pub enum Phase { Opened, Closing, Closed, Minimized, Restored, Given }
pub struct State { windows, views, focus, desks, open, outputs, busy, texts }   // serde; docs/api.md

trait Plugin { fn state(&self) -> Value { Value::Null } }           // kit runtime: state.get, read-only
// chora_kit::cube
impl Cube { fn find(conn) -> Option<Cube>; fn list(..) -> Faces; fn turn(.., to: Face); fn pull(.., surface) }
// chora_kit::testing
impl FakeCore { fn new() -> Self; fn plugin<P: Plugin>(..) -> P; fn send(&mut self, event: Event);
    fn refuse_next(&mut self); fn scene(&self) -> &Scene; fn check(&self, state: &Value) -> Result<(), Vec<Broken>> }
// core, read-only
scene.boxes { output? } -> [{ id, owner, kind, output, rect, corners, opacity, clipped, z,
                              text?, font_px?, fg?, bg? }]
```

`actions`, `keysheet`, `ask`, `launch`, `notice`, `settings::Common`: as ARC section 2. Changes: `System`
gains `Desk(Step)` and `Cube(Turn)`; every `System` value and every tool has a `words::Key`; `ask` gains
`always`.

### 2.3 Effect hook for S3 (part of F3)
`effects` on nodes and the foreign `update` of D58 exist (api.md:56, 291). An effects plugin still needs:
1. Window phases as events: `<look>.window {surface, node, phase, rect}` from `kit::wm`. With WM1.
2. Pass-through: the book applies a foreign `effects` or `opacity` update on a window group and on the
   root groups of the look, in every look alike (playbook beat 9 burns the shells too). With WM1.
3. `linger` in the core: a toplevel whose group carries `linger: ms` keeps its last buffer as a ghost
   without input until the time ends or the owner removes it. Slice FX1, before P4. Until FX1 merges the
   hook is designed, not built. The frame-budget cell of an effect reads `render.stats`.

### 2.4 Cut gate (`tools/cut-check`, in `ci-check` and the merge gate)
ARC's four checks (deps, assets, names, packaging) plus:
- **Plugin to plugin**: no workspace dependency between crates under `plugins/` and `services/`. Blocking
  from its first day; it holds today.
- **Copies** (`tools/copies.toml`): per mechanism that moved to kit a pattern, the paths that may hold it
  and the count allowed elsewhere. A count only falls; at 0 the row blocks. First rows:
  `Event::SurfaceCreated` and `to != NAME` outside kit, `"confirm.answer"`, `fn language`,
  `XDG_RUNTIME_DIR`, `UnixListener::bind` in plugin tests, `Command::Focus` outside `kit::wm`. This is
  the "no copy left" check of F2.
- Assets, names and packaging report until the asset moves (ARC steps 3, 4), then block.

## 3. Safety net: state, act contracts, invariants (F7)
### 3.1 State
Every plugin with a kit runtime answers `state.get`; `choractl state` joins the answers with core facts
(focus, outputs, toplevels with owner). chora-chat has no kit runtime; the assistant reports its state.
Shape per look: `{ windows: [{surface, title, app_id, output, desks, rect, shown, minimized, maximized,
fullscreen, floating, z}], views: [{surface, owner, pane}], focus: {holder, surface?}, desks, open:
[{id, kind, modal, by}], busy: [..], texts: [{node, text, rect, px, fg, bg}] }`; assistant and notify add
`{helper: idle|working|asking|no_helper, shown: [..]}`. `busy` names running animations and pending
proposals, so "quiet" needs no clock. `texts` are the runs a drawer painted into an image (xp, assistant
and notify create no text node: 0 `NodeKind::Text`, 29 `Image`).
Until a look uses `kit::wm`, a `state.rs` adapter fills the shape. A port deletes the adapter; the test is
that the fields the slice declares untouched are equal before and after, with ids normalised by app id
and title. A changed field is a new golden in its own commit, reviewed as a behaviour change.

### 3.2 Act contracts (`tests/acts/`)
```python
with Session(looks=["xp"]) as s:
    s.act("open notepad", cues=[key("Super+r"), text("notepad\n")],
          opens=[], expect=Expect(new_window={"app_id": "notepad", "owner": "xp", "focused": True}))
```
`act` runs the cues, waits until every `busy` is empty (or a timeout, which fails), reads `choractl state`
and `scene.boxes`, compares with `expect` (windows new and gone, owner, focus, open, notice, helper),
then runs every invariant. A mismatch fails with the act, the state diff and a still. An act declares what
it leaves open; deliberate disconnects (restart, `cube.close`) are declared too. Acts run on the wall
clock; stepped runs for recordings need the kit clock in every look (only construct exports `clock.step`
today) and come with NET4.

Known red: `tests/acts/known-red.toml` lists each invariant and act that fails on today's code, with its
finding id and the slice that turns it green. A listed failure does not block; a listed pass fails the
gate (the row must go). The count is reported with F7 and never called met while rows remain.

Where they run: (1) unit level, `FakeCore::check` (I1, I3) after every `send`, I4 per scripted act;
(2) merge gate step `acts` per look, beside `doors` and `matrix` until all ports have merged, then a
slice of its own retires the cells the acts cover; (3) the simulated user test and the new tour (3.4).

### 3.3 First invariants (each from a glitch class of tours v3 to v8 or an open finding)
| # | invariant | read from | class | slice |
|---|---|---|---|---|
| I1 | every toplevel is attached, owned by one look and in that look's `windows`; a `view` is not an owner | state, scene | lost window (co-18, sd-h1) | NET2 |
| I2 | a shown window has a size and lies at least 32 px inside its output, except declared floats | boxes | off-screen window | NET3 |
| I3 | once quiet, the core focus is a shown window of the one look that says it holds the focus | state | focus stays with the other look (sh-23) | NET2 |
| I4 | `open` after the act is `open` before plus what the act declared | state | menu left open (v3-v8, sd-h2) | NET2 |
| I5 | no two text boxes of visible nodes or reported runs overlap on one output without an opaque node between | boxes, texts | overprint | NET3 |
| I6 | a text box lies inside its panel | boxes, texts | text overflow (mc long keys) | NET3 |
| I7 | the work area of an output whose look draws the wallpaper shows more than one colour | screen.capture | black screen, black pane (sh-23) | NET2 |
| I8 | the helper is not `working` after its answer event; one helper surface per output | state | stale chat state | NET2 |
| I9 | a notice chora-notify accepted during the act is in its `shown` and has a box inside the output, not under the chat | state, boxes | notice not drawn (sh-18) | NET3 |
| I10 | no visible text matches a raw entry (`.desktop`, `%U`, a reverse-DNS id, a binary name) | boxes, texts | raw launcher entries | NET3 |
| I11 | a probe client with the focus receives no key of a binding that a look holds | probe log | stray key (sh-24) | WM2 |
| I12 | no bar or panel box over a fullscreen window; the held window on top; a modal above its parent | boxes, state | bar over fullscreen (F29), cover (B5) | NET3 |
| I13 | the projected corners of neighbouring cube faces meet within 1 px, at rest and at a held 45 degrees | boxes (corners) | cube seams (co-21, DB14) | NET3 |
| I14 | during the act: no `effect.failed`, no panic line, no disconnect the act did not declare | events, logs | crash, effects lost | NET2 |

Limits, stated so they are not read as coverage: I5, I6 and I10 see text nodes and reported runs only.
waybar and mako are outside programs (the overprint j1-4 "No helperVolume 40%" was waybar text) and stay
pixel cells until a panel block replaces them. I7 skips looks whose wallpaper is an outside program
(tiling with swaybg). I9 covers chora-notify only. `effects.dropped` is reported, not failed (D94).
A finding seen by eye becomes a new row before it is fixed.

### 3.4 Hooks for what follows (slice NET4, before P2)
- Simulated user test: `Session.observe()` (state, boxes, a still) for the persona model; cues
  `click(label)` through boxes and texts (Erika and Murat use the mouse only), `remote(key)`, `say(text)`;
  `act` without `expect` runs the invariants; a broken one becomes a finding row for triage.py.
- Visual checks against design/looks: boxes and texts give rects, font size and colours; a `guide.yaml`
  per look (grid, margins, minimal contrast, from design/looks/*.md) in its flavor folder; the checker
  measures spacing, alignment, contrast and text fit and flags frames for opus vision.
- Video checker: `choractl record` writes `frames.jsonl` (frame, time, damage hash per output) for black,
  frozen and reverting frames, without decoding. Needs the kit clock in every look.

## 4. Slices

One merge each through tests/merge-gate.sh. A fix or feature is red first; MV, NET and CUT slices cannot
be red first and carry a selftest instead (a planted breach must fail). Pattern for a shared part: kit
module with FakeCore tests and a pilot look in one merge; then the other looks, one lane per look. A port
deletes the look code and its state adapter in the same merge and lowers a row of `copies.toml`. Every
port also carries the port cells: restore from meta written by the 29e2cd77 binary, a snapshot of
`arrange()` per layout, the grab table of the core, the taskbar order (R10, R24). "Rows" are
priorities-sprint1 section 1; "C" are rework-audit clusters. Model: sonnet unless named.

| id | slice | test or gate | look | size, model, cost | removes (unblocks) |
|---|---|---|---|---|---|
| MV1 | size ratchet on production lines: a listed file grows only in a merge that also extracts from it; with CUT1 in one merge | selftest | all | S, 0.5 % | C11 |
| MV2 | moves without logic: `impl` blocks of xp wm.rs (4,273), mc main.rs (3,456), tiling main.rs (2,281) into child modules, each file under 1,500; three lanes | `--color-moved` review, the existing gate | xp, mc, tiling | M each, 4.5 % | C11; (ports stop conflicting) |
| CUT1 | `tools/cut-check`: dependencies blocking; assets, names, packaging, copies in report mode | selftest per check | all | S-M, 1.5 % | (row 14, C15) |
| NET1 | `kit::testing` FakeCore from mc testing.rs with `refuse_next`; `Plugin::state`, `state.get`, `choractl state` | mc tests pass on it, mc testing.rs deleted | mc | M, 2 % | C9 part, row 18 |
| NET2 | `tests/acts`: `act`, `Expect`, I1, I3, I4, I7, I8, I14, known-red list; `state.rs` adapter per look; gate step `acts` with the door acts | a cell per invariant that breaks it on purpose; the door acts run on the head with every failure listed | all four | L, opus (two lanes: harness, adapters and acts), 7 % | C10, P2, P3, P5 |
| NET3 | core `scene.boxes` (boxes.rs); text runs from chora-draw into `texts`; I2, I5, I6, I9, I10, I12, I13 | core unit tests with a font check; I13 red on today's seams, listed | core, draw, construct | M-L, opus, 4.5 % | (co-21 fix, visual checks) |
| NET4 | `observe`, `click`, `remote`, `say`, finding rows; kit clock in every look; `frames.jsonl` and the video checker | selftests; a planted frozen stretch is found | all | M, 3 % | (P2, new tour) |
| WM1 | `kit::wm::{book, give, kept, state}` with proposals, `Phase` events, D58 pass-through; pilot tiling | FakeCore script: create, refused attach, detach, destroy, output removed, give in, refused give out; port cells | tiling | M-L, opus (fixes the types), 4 % | C7 part; (rows 2, 16) |
| WM1b-d | port mc, construct, xp to `book` | same script per look; adapter deleted; copies rows to 0 | one each | M each, 4.5 % | C7; ARC row 1 |
| WM2 | `kit::wm::focus`; core fix of the pointer-gated grab (input.rs:461-519, extracting from input.rs) | I3, I11 cells red first: two panes (sh-23), stray key (sh-24), keyboard after a stale pointer | core, then all | M-L, opus, 4 % | rows 3, 16 part; A17, A18 |
| WM3 | `kit::wm::open` in every look | I4 cell red first: Start menu open through a drag (sd-h2) | all | M, 2.5 % | (rows 10, 16) |
| WM4 | `kit::wm::desks`, `<look>.desks`; tiling tags, then xp desktops | acts: view tag, move to tag, urgent, xp desktop switch on a screen and on faces; port cells | tiling, xp | M-L, opus pilot, 4 % | (rows 1, 15, 16) |
| WM5 | `kit::wm::stack` rules; scene layers (D89) with `held`; mc root reorder (D83) goes | I12 cells; core unit test of the layer order | core, all | M, 3 % | B5; (row 17) |
| WM6 | construct pane flags to enums with a transition table (AR14); co-21 seam fix | I13 goes green; table test | construct | M, 2.5 % | C8; co-21 |
| WM7a | cube role API (`list`, `turn {to}`, event, `pull`), `kit::cube`; the hosted xp session measured (1.2) | acts: turn by key, pager, drag; back at 1:1 with no construct surface; construct killed mid-turn keeps I1 | construct, xp | M, opus, 3 % | (row 1) |
| WM7b | after the owner's choice: xp drives the role; its own turn code goes or moves to kit; flat fallback | the WM7a acts in the chosen form and without construct | construct, xp | M-L, opus, 4 % | row 1 (DB12, co-16, 18, 20) |
| KIT1 | `chora_scene::paths`, `kit::words` typed keys with `Language::of_session` | empty `LC_ALL` does not mask `LANG`; a private dir of another owner is refused | all | S-M, 1.5 % | C14; (row 8) |
| KIT2 | `kit::actions` with `settings::Common` (C10); pilot mc | kit: one YAML binding gives one `System` action; mc configs of 29e2cd77 load to the same table | mc | M, 1.5 % | (rows 4, 10, 12) |
| KIT2b-d | port construct, tiling, xp (xp gains `bindings`; it has no action enum today) | the cross-look test becomes real with the second look; door acts unchanged | one each | M each, 4.5 % | row 4; C2, C12 |
| KIT3 | `kit::keysheet` (needs KIT2b-d) | acts read the drawn rows (`texts`): one spelling and wording in all four | all | M, 2 % | row 10; C3 |
| KIT4 | `kit::ask` the only question model in mc and construct, with `always` reserved | a bidi prompt shows sanitized; a press inside 400 ms is refused; buttons follow the language | mc, construct | S-M, opus review (security), 1.5 % | sh-26, sh-27; C6 |
| KIT5 | `kit::notice`, chora-notify the only server, zbus out of kit; mc slot | I9 in every look; `cargo tree -p chora-kit` without zbus | all | M, design pass for mc, 3 % | row 6; C4 |
| KIT6 | `kit::launch` (D87 revised) | FakeCore: exit before a window gives `Exited` at once | xp, tiling | M, 2 % | C13; (row 12) |
| KIT7 | chora-assistant the only helper presenter; the xp and tiling helper doors on it | I8 in every look; one state machine test | all | L, opus design pass, 5 % | row 5; C5; (rows 9, 11) |
| KIT8 | `kit::tool` and `asks`; MCP reads `risk`; `system.*` skeleton with one service | an `ask` tool called by MCP shows the `kit::ask` card | voice, assistant | M, 3 % | (rows 7, 9) |
| CUT2 | asset moves and `look.theme` event (ARC steps 3, 4); cut gate blocking | no breach; a fifth fake look gets a costume | assistant, tiling | S-M, 1.5 % | C15; (row 14) |
| FX1 | core `linger` (2.3) | a destroyed toplevel with `linger` stays drawn and takes no input | core | M, opus, 2.5 % | (S3, P4) |

Rows no slice removes, said plainly: 7 and 9 (KIT8 is only the base; B4 core protocols, A15 steps 3-8,
C0 sources, V1, V2 need their own design after P2); 12 (after KIT2 and KIT6, in the P2 fix list); 13 and
17 with X10 (they need the panel block of D73, a design pass of its own on WM5, not before P3); 15
(after WM7b, needs K4 and K12); 14's split packages (after CUT2); 19 and 20 (stand alone, any free
lane); AR5 pointer part, AR11 and the files over 1,500 lines that MV2 leaves.

### 4.1 Rank by work unblocked
NET1, NET2 (every slice) > WM1 with its ports (rows 1-4, 13, 16, 17 rest on it) > KIT2 with its ports
(4 to 5 rows) > WM2 > WM3 (the most frequent glitch class) > NET3, NET4 (P2 and the tour) > KIT4 >
WM4, WM6, WM7 (the owner's cube) > KIT5 > KIT3 > WM5 > KIT6 > KIT7 > KIT8 > FX1.

### 4.2 The week (to Sat 03:59; about 33 % left after this lane) and after
| when | merges (at most 12; one gate runs at a time) | cost |
|---|---|---|
| Tue-Wed | MV1 with CUT1; MV2 (three lanes); mc-32 verify (haiku, no merge) | 7 % |
| Wed-Thu | NET1; NET2 | 9 % |
| Thu-Fri | WM1 (tiling); KIT1; KIT2 (mc); KIT4 | 8.5 % |
| Fri | NET3 if NET2 merged by Thursday, else it moves; docs (F8), debt recount, kit coverage | 6 % |
| reserve | 2.5 %; if ahead: WM1b (mc) | |

After the week, before P2 (21 to 26 %, with or without NET3): WM1b-d, KIT2b-d, WM2, WM3, NET3 or its rest, NET4, WM6.
With P3 (cube, settings, tray): WM4, WM5, WM7a, WM7b, KIT5, KIT6, KIT3, KIT8. Before P4: FX1. KIT7 and
CUT2 when a lane is free.

Against the criteria, as they will stand on Saturday (owner 10:59: partly is open):
- F1: this plan, reviewed.
- F2 open: lifecycle in one look of four, actions and config in one of four, the question card in the two
  looks that had a copy; key sheet, notices and launching not started; `copies.toml` rows above 0.
- F3 open: the design is here, with two open points on the cube (1.2) and `linger` not built; the first
  slice (WM1) is lifecycle, not focus, stacking, desks or the cube.
- F4 open: the ratchet and three files. Still over 1,500 production lines: chora-scene scene.rs 2,033,
  mc region.rs 2,272, core render.rs 1,858, input.rs 1,729, ipc.rs 1,720, construct cube.rs 1,642, voice
  chat.rs 1,536. Counted with the command of debt-rc; a test module moved out is not a drop.
- F5 open unless measured: `tests/coverage-summary.sh` on chora-kit, target 80 % of lines; F17 has no
  slice (not reproduced since 11-15) and stays open.
- F6: the dependency check blocks from CUT1; the rest of the cut gate reports.
- F7 open: as redefined it needs every playbook beat and the video checker, which come with P2 and NET4.
  The week delivers the contract, six invariants and the known-red list; foundation.md:20 still says
  tour v9 and should be rewritten.
The PM should ask the owner now whether P1 runs on into next week until the "before P2" list has merged,
or whether F2, F4 and F7 are re-scoped for Saturday.

## 5. Proposed decision records (the PM files them; "owner" marks his choice, not the PM's)

1. **WM core** (owner asked 10:49). Window-management mechanism (lifecycle, focus, open shell surfaces,
   desks, handover and kept state) is the library `chora_kit::wm`, a model without IO whose changes the
   look settles; the look keeps geometry, layout, the stacking order, decoration and motion.
2. **Desks.** A desk is a set of windows in the look's model, shown on its output or bound to an output.
   A cube face is a virtual output; the cube holds no desk membership; `placed` (D84) stays in construct.
3. **Cube role** (owner: hosted or in place, after WM7a). construct exports `cube.*`; looks reach it as a
   role (D71) through `chora_kit::cube`; without it they switch desks flat. Fulfils D73's last sentence.
4. **State contract.** Every plugin with a kit runtime exports `state.get`; `choractl state` joins them;
   act contracts and invariants read state and `scene.boxes`. Visual and video checks stay beside them.
5. **`scene.boxes`.** A read-only core method: rectangle, projected corners and text facts of the visible
   nodes of an output; image drawers report their text runs in `state.get`.
6. **Shared first.** A feature or fix that touches two looks starts as a kit function with a test on
   `kit::testing`, then one look, then the others; moves come before logic; `copies.toml` only falls.
7. **Known red.** A failing invariant is listed with its finding and its slice; a listed pass fails the
   gate; no criterion is met while its rows remain.
8. **Tools.** Exported methods carry `risk` and a word key as defaults; the user's `confirm` list
   overrides; external sources use the same rule; grants stay in the core; `kit::ask` is the only
   question model, with `always` for grant questions; system services are the role `system.*`.
9. **Effect hook.** Window phases as `<look>.window` events, foreign effect updates applied by `kit::wm`
   on windows and look roots (D58), `linger` in the core. No effect API beyond `effects` on nodes.
10. **D92 kept, extended** (owner, if he wants Fluent). Typed keys in `kit::words`, tables as the store, a
    term per look; Fluent files loaded at run time are a later change inside `kit::words`.
11. **D87 revised** (owner: replaces his pick "pidfd + waitid" of 04:38). pidfd on the main pid of the unit
    that `choractl run` started, checked against the unit state; no `waitid` (`systemd-run` is the parent, D42).
12. **D89 kept, extended.** Layers background, normal, held, overlay, lock; inside a layer the look orders.
13. **D93 kept, extended.** Old spellings load as aliases for one release; the configs of 29e2cd77 are a
    test. The one exception to "no compatibility code before 1.0" (D3), for daily driving.
14. **D95 amended.** The cut gate blocks a plugin-to-plugin dependency and counts copies; construct finds
    managers through entries under `/usr/share/chora/managers/`.

## 6. What the reviews changed, what I did not take, limits

Taken, risk review: moves before the net (R1); invariants split by what they read (R2, R22); `boxes.rs`
and a ratchet that allows extraction (R3); at most 12 merges, acts beside doors (R4, R5); proposals with
`confirm` and `refused`, look-owned order, `Detached`, tiling as pilot, versioned kept state, `views`,
desk scope, no generic (R6-R13); the cube measured before a choice, no grab hand-over by default, `by`
kept (R14-R17); text runs and the stated blindness of I5, I6, I10 (R18); wall clock and `busy` (R19);
known-red list (R20); narrowed invariants (R21); goldens by declared fields, port cells, selftests
(R23-R25); the week cut to about 31 % and the criteria stated as open with corrected numbers (R26-R28).
Taken, fit review: the refactor placed before P2 with known-red routing; D92 kept, Fluent left to the
owner; one tool rule with voice `confirm`, `always` and external sources; NET4 (`observe`, mouse, remote
and voice cues, boxes with text and colour, video checker); focus and desk policy as hooks; rows without
a slice named; WM3 ranked up; the owner's records marked; slice ids that no longer clash with the backlog.
Not taken: a lazy cube as the xp default (fit review 1): it moves client windows between outputs in
mid-turn (R17), and neither form is measured. Keeping `xp/cube.rs` for good (R15): only until WM7a and
the owner's choice. A video checker inside this week (fit review 3): it needs the kit clock in every
look; F7 is reported open instead.
Limits: nothing was built or run; sizes and costs are judgement, and the week follows the risk reviewer's
numbers (he puts the first draft at 42 to 45 %). Not read in depth: construct `panes.rs`, voice, the
assistant; KIT5 for mc, KIT7 and KIT8 keep a design pass. The WM1 lane may find a simpler `Policy`.
