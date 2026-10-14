# Roadmap

Each milestone has a goal, a scope and an exit criterion. A milestone is done
when its exit criterion holds. After M4 the core is feature complete, and
testing starts (D3). M5 and M6 build on the core.

## M0: Workspace and tooling

**Goal:** the first line of Rust meets all guards, and a tag produces a
package.

- Cargo workspace with `chora-scene`, `chora`, `choractl` and the reference
  plugins in `plugins/` (D36).
- Workspace lints: clippy `pedantic`, `cognitive_complexity` and
  `excessive_nesting`; `unsafe_code` denied. `clippy.toml` sets the limits.
- CI: build and `cargo test` in a Fedora 44 container; `cargo-deny` for the
  licenses and advisories of the dependencies.
- A pre-commit hook that rejects files with a GPL-3.0-only header (D8).
- Dependabot for Cargo.
- commitizen for version bumps and the changelog.
- A release workflow: a `v*` tag builds the RPM in Fedora 44 and attaches it
  with its SHA256 sum to a GitHub pre-release (D14).

**Exit:** CI is green. The RPM of a test build installs in a Fedora 44
container, and `chora --version` runs there.

## M1: Nested proof of concept

**Goal:** the smallest proof of the concept, inside an existing session.

- winit backend on a pinned Smithay revision (D16).
- Protocols: `xdg-shell` with popups, subsurfaces, `xdg-decoration`,
  `linux-dmabuf`, `viewporter`, `xdg-output`, data device and
  `primary-selection`.
- Scene with group, surface and rect nodes, 2D transforms (D18, D19).
- Transactions that wait for client commits (D22).
- IPC socket (D29) with `hello`, `scene.query`, `scene.apply`,
  `events.subscribe`, `input.grab` and `output.list`.
- Ownership (D31).
- The startup command (D30).
- Safety layer: quit and emergency reset (D23).
- `choractl` with `query`, `apply`, `events` and `outputs`.
- `chora-tiling`: a reference plugin with a master and stack layout and
  keybindings to start a terminal, close a window and move the focus.

**Exit:** an automated test in a headless sway passes:

1. `chora -- chora-tiling` starts nested.
2. Three foot windows appear without overlap. `scene.query` and a screenshot
   confirm the layout.
3. After `kill` of `chora-tiling`, `chora` still runs, and the three surfaces
   are detached.
4. The emergency reset puts the three surfaces in the grid. The test sends
   SIGUSR1, because synthetic keys do not reach a nested compositor.

The test, `tests/nested.sh`, also checks fixed regressions.

## M2: Session on real hardware

**Goal:** chora runs as a login session on AMD and Intel (D11).

- udev, DRM/KMS, libinput and libseat backends; the VT switch bindings.
- Several outputs, cameras (D20) and the `camera` command;
  `wlr-output-management` (D27).
- Commit blockers, as in niri and Pinnacle, so that the new buffers of a
  transaction appear in the same frame.
- Protocols: `cursor-shape`, `presentation-time`, `wlr-layer-shell`,
  `ext-session-lock`, `ext-idle-notify`,
  `idle-inhibit`, `text-input`, `input-method`, `pointer-constraints`,
  `relative-pointer`, `ext-foreign-toplevel-list`,
  `wlr-foreign-toplevel-management`, `xdg-activation`, `fractional-scale`.
- `xwayland-satellite` (D26).
- `chora.desktop`, `chora-flavor` and a tiling flavor with a systemd target.

**Exit:**

1. Automated, nested: waybar, fuzzel and swaylock work.
2. On the hardware of the author: a tiling session with bar and launcher runs
   for 8 hours on an AMD and on an Intel machine, without a crash or a
   restart of the core.

## M3: Effects and the XP showroom

**Goal:** the game-engine half of the idea, shown in a flavor in the style of
Windows XP (D39). The flavor is a demo, not a daily driver.

- Image nodes for assets, and shader nodes, effects and shaders that read
  nodes as textures (D18).
- Rendering of a node into a texture, and a mesh that deforms that texture.
  Wobbly windows and the magic lamp minimize use the mesh.
- Perspective for the 4x4 transforms, for a 3D desktop cube with flying
  windows. Layers have no depth test: the order of the siblings decides
  what is in front.
- The `animate` command (D28).
- GLSL hot reload; compile errors as `transaction.rejected`.
- Damage tracking for static and animated shaders, direct scanout (D24).
- The watchdog of the safety layer.
- A measurement of the IPC round trip for effects in every frame. The result
  decides on in-process WASM plugins (D28).
- An XP flavor: a stacking window manager with title bars, a taskbar and a
  start menu, with the assets of `kde-plasma-winxp-launcher`.

**Exit:**

1. Automated, nested: the XP flavor starts. A screenshot shows the taskbar,
   the start button and two windows with title bars. A drag of a window
   deforms its mesh. A minimize runs the magic lamp. A key binding turns
   the desktop cube with four desktops.
2. On the Intel machine at 60 Hz, four windows with blur and rounded
   corners and a 300 ms transition: `presentation-time` shows at least 99
   percent of the frames on time over 60 seconds.

## M4: Agent interface

**Goal:** an agent sees and changes the screen.

- Grants for `input.inject` and `screen.capture` (D33, D40).
- `ext-image-copy-capture` and `wlr-screencopy`; `xdg-desktop-portal-wlr`
  for screen sharing (D37).
- Plugin methods: `methods.export` and `methods.call` (D31).
- `choractl view load` and `choractl view save` with YAML views (D35).
- `choractl mcp` over stdio, with tools for whole tasks (D34).
- `wp-security-context-v1`.

**Exit:** an automated test passes: an MCP client asks for "describe the
screen", calls `tiling.set_layout`, and the next screenshot shows the new
layout. The last step of `tests/agent.sh` checks it with
`tests/mcp-client.py` in CI.

## M4.1: Agent feedback

**Goal:** an agent builds an interactive view in a few calls, from the
feedback of the first real MCP sessions (issues #3 to #10).

- Docs and code agree (#3): `clip` on all nodes, `radius` and `border` on
  rects, and the `shader` node kind with a uniform that names a node as
  texture (D18). A test parses every example of `docs/api.md`.
- A camera takes a uniform scale of the short form (#4), and animates in
  the core with `duration` and `easing` (#10).
- `choractl mcp`: item schemas for `apply_commands` and `inject_input`
  from the types of `chora-scene` (#5), and tools to subscribe to and wait
  for events (#7).
- `assets.put` stores an image or a shader of a client in another
  namespace in a directory of the core, removed with the nodes of the
  client (#6).
- `output.reserve` reduces the work area for the life of a connection
  (#8); the `reserve` of `chora-tiling` goes.
- `process.spawn` starts a program with the environment of the startup
  command, behind its own grant (D42), and `plugins.restart` restarts a
  plugin of a view (#9).
- Option: harden the grant with an allowlist, for example regular
  expressions over the argv. `choractl grant` reads the list from a file in
  `~/.config/chora/` and hands it to the core with the grant, since the
  core reads no configuration (D35); the core then refuses every other
  program.

**Exit:** an automated test passes: an MCP client uploads a shader,
reserves a panel at the edge of the output, draws it as a rounded shader
node with two buttons, and moves the camera with an animation when the
user clicks a button, which the client learns from `wait_events`. The last
step of `tests/agent.sh` checks it with `tests/mcp-client.py panel` in CI.
The allowlist of the grant `process.spawn` stays an option.

## M4.2: Test round

**Goal:** the API covers what the first test round on real hardware
missed (issues #12 to #27), without new policy in the core.

- Grabs: `input.ungrab(binding)`, and `input.key` tells the release of a
  grabbed key with its state (#18, D45).
- One overlay root per output, `screen:NAME`, for nodes fixed to the
  screen (#19, D43).
- `process.spawn` with `capture`: the core keeps stdout, stderr and the
  exit status, at most 64 KiB, and `process.wait(pid, timeout)` returns
  them (#22).
- `output.configure` for scale and transform (#22, D44).
- Meshes: an optional `depth` per point, which the transform projects, so
  the texture maps perspective-correct, and a point behind the eye drops
  only its triangles, not the layer (#25, #26).
- Nested, the keymap of the host session: a second Wayland connection to
  the host reads its `wl_keyboard.keymap`; `XKB_DEFAULT_LAYOUT` still wins
  (#12).
- #14 waits for a `WAYLAND_DEBUG=1` log of foot.

**Exit:** `tests/agent.sh` and `tests/nested.sh` check each item, and an
MCP client runs a probe with `capture`, reads its output, sets the scale
of the output, and draws a HUD under `screen:NAME` that stays in place
during an animated camera move. The last step of `tests/agent.sh` checks it
with `tests/mcp-client.py hud` in CI. #14 still waits for a log.

## M5: Packaging

**Goal:** chora installs like every other desktop on the host.

- A derived secureblue image with the chora package (D10, D14).
- The tiling flavor in the image, and a classic flavor.
- Services as Quadlet units (D12).

**Exit:** after an image update, the login screen offers chora. The tiling
flavor starts, and one Quadlet service connects to the IPC socket.

## M6: Agentic flavor

**Goal:** the agentic workday scenario works end to end.

- A local voice pipeline on AMD Strix Halo, for example the HP ZBook Ultra
  G1a with 128 GB. [Gufo](https://github.com/gufo-org/gufo) serves the
  models over an OpenAI-compatible API, one model per process, and runs as
  Quadlet units (D12):
  - Qwen3-ASR 1.7B for speech to text, streamed over a WebSocket.
  - Qwen3-TTS 1.7B for text to speech, streamed as PCM. German is still
    to check.
  - Qwen3.8 Flash-Next as the main model, for tasks that need reasoning.
  - A small model, for example a Qwen with 1.7B to 4B parameters, in
    llama.cpp with Vulkan. It routes each turn, answers short commands
    with the tools of `choractl mcp` itself, and gives a first answer
    while the main model works. With Vulkan it takes none of the four
    HIP processes that a GPU allows.
- A voice agent on the CPU: voice activity detection, turns, sentence
  chunks for the TTS, and barge-in, which stops the audio and cancels the
  model request. Pipecat for a first prototype; a Rust service later, if
  the prototype holds. The goal is about one second from the end of the
  speech to the first audio.
- A browser in app mode as service for HTML content (D38).
- A morning view and task views.
- Task and email sources are MCP servers of the agent, outside chora.

**Exit:** the user says "show my tasks for today", and the agent opens a
morning view with the tasks from an MCP source. Then the user says "open a
view for the first task", and the agent opens a view with at least two
surfaces.

## After M6

- Conformance tests with wlcs, as Pinnacle does.
- Hardening and testing on the daily system.
- Items from the [Idea backlog](ideas.md).
