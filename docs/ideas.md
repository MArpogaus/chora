# Idea backlog

Everything collected so far that is not (yet) a decision. Ideas move from here
into [Decisions](decisions.md) or get dropped.

## Interaction

- Voice as the primary channel to an agent: speech-to-text in, text-to-speech
  out, with a robust push-to-talk / wake-word mechanism.
- "Morning canvas": greeting, current tasks, pre-sorted email, a day plan
  prioritized together with the agent.
- Task-specific canvases built on demand: data analysis (data, plots,
  interactive exploration), writing email, web research, coding.
- Live adjustment by voice: resize, rearrange, highlight, dim, switch views.
- Views that adapt to task *and* taste: the agent learns layout preferences.

## Visuals

- Effects in the spirit of Hyprland and Compiz: blur, shadows, rounded
  corners, wobbly windows, cube, expo overview, transitions.
- Infinite zoomable canvas (zoomable UI).
- Surfaces as 3D objects.
- Pixel-perfect recreations of existing desktops (Windows XP, KDE Plasma) as
  flavors — a proof of flexibility, not a goal.

## Agents and the ecosystem around Void

- A coordinating agent with dedicated permissions that installs, configures
  and *writes* plugins and services.
- Inventory: a local, git-versioned store of reusable elements; one-off code
  is discarded, useful code kept.
- Agent memory in the home directory — a separate, opinionated workflow on top
  of Void.
- Screenshots as feedback: the agent sees what it built and corrects it.

## Services

- Local inference engine (LLM) as a container.
- Speech services (whisper, piper).
- Development environments with special configurations (distrobox).
- A webview host for agent-generated HTML.

## Platform

- Flavors as installable bundles: classic, tiling, agentic.
- A derived secureblue image offering Void as an additional session.
- A plugin/flavor registry later, maybe as OCI artifacts (fits the container
  world).
