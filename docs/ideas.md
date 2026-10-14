# Idea backlog

All ideas that are not decisions yet. An idea moves to
[Decisions](decisions.md) or leaves the list.

## Interaction

- Voice as the main channel to an agent: speech to text in, text to speech
  out, with a robust push-to-talk or wake-word mechanism.
- A morning view: greeting, current tasks, sorted email, and a day plan that
  user and agent prioritize together.
- A view for each task, built on demand: data analysis with data, plots and
  interactive exploration; email; web research; coding.
- Live changes by voice: resize, rearrange, highlight, dim, switch views.
- Views that fit the task and the taste of the user. The agent learns the
  layout preferences of the user.

## Visuals

- Effects like in Hyprland and Compiz: blur, shadows, rounded corners, an
  overview, transitions. Wobbly windows, the magic lamp and the cube are
  part of M3 (D39).
- An infinite zoomable canvas through cameras.
- Surfaces as 3D objects.
- Complex 3D scenes, with models, depth and ray tracing, belong in clients,
  not in the core. A client renders them with Vulkan, for example with
  Bevy or Godot, and chora shows its surface as a node, without a copy
  through dmabuf. The client gets plain keys with press, release and
  repeat, and relative pointer motion for a mouse look, as every Wayland
  client does. Through IPC or MCP it still moves cameras and panels. The
  scene of the core stays 2.5D: planes in perspective, meshes and fragment
  shaders on GLES, without depth test. A 3D node in the core would make it
  an engine and break its freedom of policy. On the Radeon 8060S of Strix
  Halo, raster with ray-traced shadows and reflections at 1080p to 1440p
  is realistic, full path tracing in real time is not.
- Pixel-perfect copies of existing desktops as flavors, for example KDE
  Plasma. The XP showroom of M3 is the first (D39). They prove the
  flexibility. They are not a goal.

## Agents and the ecosystem around chora

- An inventory: a local, git-versioned store of reusable elements. Code for
  one-time use goes away, useful code stays.
- Agent memory in the home directory.
- *Demiurge* as the name of the coordinating agent. In Plato, the demiurge
  shapes the forms in the chora.

## Core extensions

- In-process WASM plugins on wasmtime with the Component Model, for effects
  that need code in every frame, for example wobbly windows. Only after a
  measurement shows the need (D28).
- A portal backend of its own, so that screen sharing can share a group or a
  canvas instead of a whole output or window.
- Restricted IPC sockets for containers: a socket that only offers a subset
  of the API to the clients behind it.

## Services

- A local inference engine (LLM) in a container.
- Speech services (whisper, piper).
- Development environments with special configurations (distrobox).
- A dedicated webview host on `gtk4-layer-shell`, if a browser in app mode is
  too slow, too heavy or cannot be transparent.

## Platform

- Flavors as installable bundles: classic, tiling, agentic.
- A later registry for plugins and flavors, possibly as OCI artifacts. OCI
  fits the container world of the host.
- A Copr repository, so that `dnf` updates chora like every other package.
