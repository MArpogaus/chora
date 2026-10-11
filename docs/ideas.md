# Idea backlog

All ideas that are not decisions yet. An idea moves to
[Decisions](decisions.md) or leaves the list.

## Interaction

- Voice as the main channel to an agent: speech to text in, text to speech
  out, with a robust push-to-talk or wake-word mechanism.
- A morning canvas: greeting, current tasks, sorted email, and a day plan that
  user and agent prioritize together.
- A canvas for each task, built on demand: data analysis with data, plots and
  interactive exploration; email; web research; coding.
- Live changes by voice: resize, rearrange, highlight, dim, switch views.
- Views that fit the task and the taste of the user. The agent learns the
  layout preferences of the user.

## Visuals

- Effects like in Hyprland and Compiz: blur, shadows, rounded corners, wobbly
  windows, cube, overview, transitions.
- An infinite zoomable canvas.
- Surfaces as 3D objects.
- Pixel-perfect copies of existing desktops (Windows XP, KDE Plasma) as
  flavors. They prove the flexibility. They are not a goal.

## Agents and the ecosystem around Void

- A coordinating agent with special permissions. It installs, configures and
  writes plugins and services.
- An inventory: a local, git-versioned store of reusable elements. Code for
  one-time use goes away, useful code stays.
- Agent memory in the home directory, as a separate, opinionated workflow on
  top of Void.
- Screenshots as feedback: the agent sees its result and corrects it.

## Services

- A local inference engine (LLM) in a container.
- Speech services (whisper, piper).
- Development environments with special configurations (distrobox).
- A webview host for HTML that agents generate.

## Platform

- Flavors as installable bundles: classic, tiling, agentic.
- A derived secureblue image that offers Void as an additional session.
- A later registry for plugins and flavors, possibly as OCI artifacts. OCI
  fits the container world of the host.
