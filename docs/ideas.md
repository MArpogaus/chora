# Idea backlog

All ideas that are not decisions yet. An idea moves to
[Decisions](decisions.md) or leaves the list.

## Interaction

- A wake word in addition to push-to-talk for the voice pipeline (D53).
- A morning view: greeting, current tasks, sorted email, and a day plan that
  user and agent prioritize together.
- A view for each task, built on demand: data analysis with data, plots and
  interactive exploration; email; web research; coding.
- Live changes by voice: resize, rearrange, highlight, dim, switch views.
- Views that fit the task and the taste of the user. The agent learns the
  layout preferences of the user.

## Visuals

- Effects like in Hyprland and Compiz: blur behind windows, shadows, an
  overview, transitions. The XP showroom has wobbly windows, the magic
  lamp and the cube (D39).
- An infinite zoomable canvas through cameras.
- Surfaces as 3D objects.
- An example client for complex 3D scenes, with models, depth and ray
  tracing (D19). It renders with Vulkan, for example with Bevy or Godot,
  and chora shows its surface as a node, without a copy through dmabuf.
  The client gets plain keys with press, release and repeat, and relative
  pointer motion for a mouse look, as every Wayland client does. Through
  IPC or MCP it moves cameras and panels.
- Pixel-perfect copies of existing desktops as flavors, for example KDE
  Plasma, next to the XP showroom (D39). They prove the flexibility. They
  are not a goal.
- The XP window frames from the xfwm4 theme of
  [xfce-winxp-tc](https://github.com/rozniak/xfce-winxp-tc) instead of the
  Aurorae theme Exposé, so that the frames and its Luna GTK theme come from
  one theme family.
- XP programs as GTK applications of their own, for example an Explorer,
  in place of the programs that left `chora-xp` (D63).
- The KDE server-decoration protocol, so that GTK3 applications such as
  Thunar get the frame of the window manager.

## Agents and the ecosystem around chora

- Managed sub-agents of the voice agent, in place of the single coding
  task: the coordinating agent starts, lists and stops them. Speech is a
  service that the sub-agents use too, possibly through MCP. The
  coordinating agent routes speech: the user addresses one or more agents
  by name, and each agent answers in a voice of its own.
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
- A private Wayland protocol for plugins, if the measurement of milestone
  0.10 shows the need (D54).

## Services

- A local inference engine (LLM) in a container.
- Development environments with special configurations (distrobox).
- A generic settings form that draws its fields from the settings schema
  of any plugin (D62). It uses GTK4 without libadwaita, so that a GTK
  theme applies.
- A dedicated webview host on `gtk4-layer-shell`, if a browser in app mode is
  too slow, too heavy or cannot be transparent.

## Platform

- Flavors as installable bundles: classic, tiling, agentic.
- A later registry for plugins and flavors, possibly as OCI artifacts. OCI
  fits the container world of the host.
