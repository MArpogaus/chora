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

- Effects like in Hyprland and Compiz: blur, shadows, rounded corners, wobbly
  windows, cube, overview, transitions.
- An infinite zoomable canvas through viewports.
- Surfaces as 3D objects.
- Pixel-perfect copies of existing desktops (Windows XP, KDE Plasma) as
  flavors. They prove the flexibility. They are not a goal.

## Agents and the ecosystem around Chora

- An inventory: a local, git-versioned store of reusable elements. Code for
  one-time use goes away, useful code stays.
- Agent memory in the home directory.
- *Demiurge* as the name of the coordinating agent. In Plato, the demiurge
  shapes the forms in the chora.

## Core extensions

- In-process WASM plugins on wasmtime with the Component Model, for effects
  that need code in every frame, for example wobbly windows. Only after a
  measurement shows the need (D27).
- A portal backend of its own, so that screen sharing can share a group or a
  canvas instead of a whole output or window.

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
