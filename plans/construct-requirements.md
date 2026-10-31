# chora-construct: requirements

Status: draft for review, 2026-10-31. Source: the interview of
2026-10-31 and the loader prototype in `examples/loader/`.

## 1. Goal

A spatial desktop for chora. The user stands in a white, empty space
with a soft, foggy horizon. There is no grid and no floor pattern.
Content floats in the space as **panes**. Each pane faces the camera.
A pane can hold a plugin, a program, a nested desktop session, or a
prompt in which an agent builds a program or game live.

The loader prototype showed that the scene API of the core is enough
for a 3D space with live windows: 4x4 transforms per node, node
textures of windows, meshes and shader nodes. chora-construct turns
the prototype into a real plugin.

## 2. Decisions from the interview

| # | Question | Decision |
|---|---|---|
| C1 | Role | Both a flavor and an overlay. A session can start into an empty or a configured construct. mc and other plugins can load the construct later. |
| C2 | Movement | Free walk on a leash: at most radius R from the start point. The leash ends softly in the fog. |
| C3 | Prompt pane | An ACP agent (as `agent_start` of chora-voice, in the sandbox container) builds a program. The pane shows the running result and the progress. |
| C4 | Leash | A soft border at radius R around the origin. Panes can stand anywhere inside it. |
| C5 | Use of a pane | Both: use it in the space (focus follows the pane under the pointer or gaze), and fly in for focused work (1:1 pixels, full screen, Esc flies out). |
| C6 | Persistence | A construct is a YAML file: panes with kind, content and pose. Mouse, voice and API change it live, and it can be saved. Like the mc scenes. |
| C7 | New panes | They appear in the view direction at a fixed distance and face the camera. They move aside to the left or right of panes in the way. The user can move them later. |
| C8 | Nested sessions | A nested GNOME session as a pane is a must for v1. |
| C9 | Prompt input | By voice and by typing in the pane. |
| C10 | Core work | The plugin builds on the core as it is. The user allows a core change for the keyboard focus of panes (#27) if the existing focus rules leave a gap; work package 0 checks this first. Other missing features are documented limits. |
| C11 | Performance | Up to 8 live panes at 60 fps. Panes that are far away or hidden freeze as still pictures. |
| C12 | Name and plan | `chora-construct`, milestone 0.10. |
| C13 | Language | Rust with chora-kit, like mc, tiling and xp. The Python prototype is a reference only. |
| C14 | Outputs | Panorama: one space; each output shows its own view direction, side by side as one wide field of view. |
| C15 | Term | Pane. "View" stays the name of the configuration views. |
| C16 | Dark theme | A black void with dark fog; the light theme is white. |
| C17 | Donut | Not in v1; system values come from the mission scene of mc as a pane. |

C8 and C10: a nested GNOME session runs today (`gnome-shell --devkit`,
see the loader README). The core already lets a client ask the owner of
a surface for the focus (`node.requested`, D31) and give the focus back
once after it cleared it (#27, closed). As a flavor, the construct owns
its windows and focuses them itself; as an overlay, it asks their owner.
A core change comes only if work package 0 finds a gap.
The host keymap (#12) stays a limit of v1.

## 3. Terms

- **Construct**: the space and its panes; one YAML file.
- **Pane**: one flat, rectangular object in the space that faces the
  camera. "View" already names a chora concept (configuration.md,
  Views), so the plugin says pane (C15).
- **Fly-in**: the camera moves in front of a pane until the pane fills
  the output at 1:1 pixels.
- **Leash**: the radius R around the origin that limits the walk.
- **LOD**: level of detail. A pane is live, reduced or frozen.

## 4. Scope

### v1 (0.10)

- The space: white, empty, foggy horizon, soft leash border.
- Walking and looking (keyboard, mouse; later voice).
- Panes of four kinds: plugin node trees, Wayland programs, nested
  GNOME session, prompt pane.
- Fly-in and fly-out; use of a pane in the space.
- Constructs as files: load, save, live change.
- Start as a flavor, and load from a running session (mc, voice,
  choractl).
- Voice tools: create, move, remove, fly to and name panes, and give
  a prompt.
- LOD for 8 live panes at 60 fps.

### Not in v1

- Core changes, unless work package 0 finds a focus gap (C10). See section 9.
- Several users or remote panes.
- VR or head tracking.
- Physics, gravity, collisions beyond the "move aside" rule of C7.
- KDE or other nested sessions: the same mechanism as GNOME, but not
  tested in v1.

## 5. Functional requirements

### 5.1 The space

- **F1** The background is white with a foggy horizon: a gradient with
  slow, soft noise. No grid, no floor lines. The dark theme is a black
  void with dark fog (C16).
- **F2** Panes far away fade into the fog with distance (depth cue
  without lines).
- **F3** The leash: walking slows down near R and stops at R. The fog
  gets denser towards the border.

### 5.2 Movement

- **F4** Keys: walk forward, back and sideways, turn, and look up and
  down. The bindings are in the settings. Defaults avoid Super, which
  GNOME takes on the host.
- **F5** Mouse: drag to look around; wheel to walk forward and back.
- **F6** A "home" key flies back to the origin, facing the first pane.
- **F7** All camera moves are animated with eased motion, at most
  about 400 ms for a fly-in.

### 5.3 Panes

- **F8** Each pane faces the camera. Its surface normal points to the
  camera centre. Billboard rotation follows the camera continuously.
- **F9** A pane has a pose (position, size) and a kind. The size is in
  logical pixels of its content at 1:1.
- **F10** Kinds:
  - **plugin**: a node tree of another plugin, for example an mc scene.
  - **program**: a Wayland client. Its surface texture shows on the
    pane.
  - **session**: a nested desktop session (GNOME in v1) as a Wayland
    client.
  - **prompt**: section 5.6.
- **F11** New panes appear as in C7. If the user looks at an existing
  pane, the new pane goes beside it.
- **F12** Move and resize a pane with the mouse while a modifier key is
  held, and by voice ("move the terminal to the left").
- **F13** Remove a pane: it closes its program or session after a
  confirmation if the program has unsaved state. The rule is the same as
  for windows elsewhere.

### 5.4 Use of panes

- **F14** In the space: the pane under the pointer gets the pointer
  events in its own coordinates. The keyboard goes to the pane that was
  clicked or flown into last (`focus`; as an overlay through
  `node.requested` to the owner, D31).
- **F15** Fly-in: a click with a modifier, a key or voice flies the
  camera in front of the pane. At the end, the pane covers the output at
  1:1 pixels (no blur) and gets the keyboard. Esc or a key flies back
  to the last pose in the space.
- **F16** While flown in, the walk keys go to the program, not to the
  camera.

### 5.5 Constructs

- **F17** A construct file lists the panes with kind, content (command,
  plugin and scene, or prompt) and pose. It also has the camera start
  pose and R.
- **F18** `construct.load NAME`, `construct.save NAME` and
  `construct.list` as plugin methods. Voice and mc use them.
- **F19** A flavor `construct` starts chora into a construct (empty or
  named in the settings).
- **F20** Loaded from mc or voice in a running session, the construct
  takes the outputs as an overlay. Leaving it gives the outputs back to
  the previous flavor's camera. A camera belongs to one client, so
  hand-over goes through the core reset (D40).

### 5.6 Prompt panes

- **F21** A prompt pane has two parts: the live result (the program
  that the agent builds, as a program pane) and a strip with the
  prompt, the agent's state and its last message.
- **F22** The agent is an ACP sub-agent in the sandbox container
  (`agent-sandbox.sh`), one per prompt pane, with its own project
  directory under a construct folder.
- **F23** Input: voice ("build me a snake game in a new pane") through
  chora-voice, and typing in the strip of the pane.
- **F24** When the agent changes the program, the pane restarts the
  program and keeps its pose. The previous run stays visible until the
  new one draws its first frame.
- **F25** The agent's permission questions show in the strip and can be
  answered by voice or click (as `agent_permission_s` today).

### 5.7 Voice

- **F26** Tools for chora-voice: `construct.pane_open {kind, content}`,
  `pane_move`, `pane_remove`, `fly_to`, `walk_home`, `construct.load`,
  `construct.save`. Names in the space are short labels, so that
  "fly to the terminal" works.
- **F27** Voice knows which pane is in front of the user, for
  "this pane".
- **F28** The construct draws its own voice HUD from the `voice.*`
  events (D65): a holographic HUD in the style of a science fiction
  game, with a waveform, a state ring and the spoken text in the
  space. It belongs to step K3.

## 6. Non-functional requirements

- **N1 Performance:** 8 live panes at 60 fps on the reference hardware
  at output scale 1 and 1.5. Panes beyond distance D or behind the camera
  freeze (the last frame stays as a still picture). Panes in the far fog
  render at reduced size.
- **N2 Layer budget:** the core limit of 64 Mi pixels (loader README)
  holds. The plugin sizes its panes and their render scale so that the
  sum stays under it. It reacts to `layers.dropped`.
- **N3 Latency:** input to a pane in the space or flown in adds no
  visible delay compared to a normal window.
- **N4 Crash safety:** a crashing pane program does not end the
  construct. A crashing construct gives the outputs back (keep_nodes off).
- **N5 Security:** prompt agents run only in the sandbox container,
  with no host home and no host network beyond `ai-net`. Panes from a
  construct file start programs only after the user confirms a file they
  did not save themselves.
- **N6 Style:** the colours and type follow the style guide of mc
  (`docs/mc-style.md`) where text shows, such as labels and the strip of
  prompt panes.

## 7. Interfaces with the core (as it is)

| Need | Core feature |
|---|---|
| 3D poses of panes | 4x4 `transform` per node, divided by w |
| Live window content on a pane | node textures of surfaces |
| Fog, horizon | shader node over the whole output |
| Pointer on a pane | pointer events of nodes; the plugin maps them to pane coordinates |
| Keyboard to a pane | `focus_node` for node panes; `focus` for own windows; `node.requested` to the owner (D31) and the focus return after a clear (#27) in overlay mode |
| Camera per output | `camera` command, one owner per output (D40) |
| Start programs | `run` / activation tokens, like mc's app contents |
| Agents | chora-voice ACP agents (sandbox) |
| Budget | `layers.dropped` event |

## 8. Acceptance (gates for 0.10)

- **G1** Start the flavor into a construct with 4 panes (terminal,
  mc scene, GNOME session, prompt pane): all four show live within 10 s.
- **G2** Walk, look, fly in and out of each pane. While flown in, text
  typed in the terminal arrives; the screenshot is pixel-sharp (1:1).
- **G3** 8 live panes at 60 fps for 60 s, with frame times from the core
  (no dropped layers).
- **G4** "Build me a snake game in a new pane" by voice: a prompt pane
  appears in the view direction, the agent builds the game, and the
  pane shows it running.
- **G5** Save, quit, start again: the construct comes back with the
  same poses.
- **G6** Load the construct from mc in a running tiling session and
  leave it again: the tiling cameras come back.

## 9. Known limits (C10)

| Limit | Effect in v1 | Core fix later |
|---|---|---|
| #12 keymap | Nested sessions use the host keymap. | keymap per nested session |
| Layer budget 64 Mi px | Limits live panes at scale 3. | budget per output, mipmaps |
| One camera owner (D40) | Overlay mode must reset cameras on exit. | camera hand-over |
| Shader limits (#16) | No `fwidth`; `mediump` breaks hashes. | none needed |

## 10. Risks

- **R1** GNOME nested with the host keymap may be awkward with a
  different layout. Mitigation: use the same layout on host and nested
  session until the core fix of #12.
- **R2** 8 live textures at 60 fps may exceed the budget at scale 1.5.
  Mitigation: aggressive LOD (N1).
- **R3** Prompt agents can start programs that grab the output. They run
  in the sandbox, and their window lives only in their pane.

## 11. Open questions

- **Q4** Value of R and of the pane distance. Start with values that
  the first prototype run tunes by eye, and keep both in the settings.

## 12. Work packages (draft)

0. Focus check: walk, fly in and type into a terminal and a nested
   GNOME pane, as flavor and as overlay over tiling. A core change, with
   a decision record, only for a gap that this test shows.
1. Space and camera: fog shader, walk with leash, home, camera
   ownership and reset.
2. Panes: billboard poses, program panes with node textures, pointer
   mapping, move and resize.
3. Fly-in and fly-out, 1:1 snapping, focus by click.
4. Constructs: file format, load, save, live change, flavor and overlay.
5. LOD and budget.
6. Plugin and session panes (mc scene as a pane, GNOME).
7. Prompt panes with ACP agents and voice tools.
8. Gates G1 to G6 as tests (headless where possible).
