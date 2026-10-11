# Roadmap

!!! note "Draft"
    A first proposal. The order and the scope of the milestones are open for
    discussion.

Each milestone has a goal, a scope and an exit criterion. A milestone is done
when its exit criterion holds on the real setup of the author. After M6 the
core is feature complete, and testing starts (D3).

## M0: Workspace and tooling

**Goal:** the first line of Rust already meets all guards.

- Cargo workspace with `chora-scene`, `chora` and `choractl` (D33).
- Workspace lints: clippy `pedantic`, `cognitive_complexity`,
  `excessive_nesting`; `unsafe_code` denied. `clippy.toml` sets the limits.
- A CI workflow for `cargo test`, and `cargo-deny` for licenses and advisories.
  The license check keeps GPL-3.0-only code out (D8).
- Dependabot for Cargo. A pinned Smithay revision (D15).

**Exit:** CI is green. `chora` opens an empty winit window.

## M1: Nested proof of concept

**Goal:** the smallest proof of the concept, inside an existing session.

- winit backend, `xdg-shell` toplevels and popups.
- Scene with group, surface and rect nodes, 2D transforms (D17, D18).
- Transactions that wait for client commits (D21).
- IPC socket with `scene.query`, `scene.apply`, `events.subscribe` and
  `input.grab`.
- Safety layer: quit and emergency reset (D22).
- `choractl` with basic commands.
- A tiling plugin as IPC client, as reference.

**Exit:** the tiling plugin arranges terminal windows in the nested session.
A crash of the plugin leaves the core running. The reset works.

## M2: Session on real hardware

**Goal:** Chora runs as a login session on AMD and Intel (D11).

- udev, DRM/KMS, libinput and libseat backends, copied from Pinnacle.
- Several outputs with viewports (D19), `wlr-output-management` (D26).
- `wlr-layer-shell`, so waybar, fuzzel and Quickshell work.
- `ext-session-lock`, idle protocols, `xwayland-satellite` (D25).
- `chora.desktop` session file.

**Exit:** a tiling session with bar and launcher runs for a full workday.

## M3: Agent interface

**Goal:** an agent sees and changes the screen.

- `choractl mcp` over stdio, with tools for whole tasks (D31).
- `screen.capture` and `input.inject`, both with approval (D30).
- Node ownership and plugin methods (D28).
- `choractl` loads and saves YAML views (D32).
- `wp-security-context-v1`.

**Exit:** an agent rearranges views over MCP and checks the result with
screenshots.

## M4: Effects

**Goal:** the game-engine half of the idea.

- Shader nodes, effects and shaders that read nodes as textures (D17).
- GLSL hot reload; compile errors as `transaction.rejected`.
- The `animate` command (D27).
- Damage tracking for static and animated shaders, direct scanout (D23).
- The watchdog of the safety layer.

**Exit:** blur, rounded corners and animated transitions run at the full
frame rate of the output.

## M5: Packaging

**Goal:** Chora installs like every other desktop on the host.

- A derived secureblue image with the Chora session (D10).
- Flavors as directories with systemd targets: tiling first (D32).
- Services as Quadlet units (D12).

**Exit:** after an image update, the login screen offers Chora.

## M6: Agentic flavor

**Goal:** the agentic workday scenario works end to end.

- Speech services: whisper.cpp or faster-whisper, piper.
- A browser in app mode as service for HTML content (D35).
- Morning view and task views.

**Exit:** the user plans the day by voice, and the agent builds the views
for the tasks.

## After M6

- Conformance tests with wlcs, as Pinnacle does.
- Hardening and testing on the daily system.
- Items from the [Idea backlog](ideas.md), for example in-process WASM
  plugins or a portal backend of its own.
