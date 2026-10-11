# Vision and principles

## The idea

Today's desktops decide for us how content is arranged: windows, panels,
workspaces. Void removes that decision from the compositor. It is a
**programmable canvas** that behaves more like a game engine than a window
manager: a scene of nodes, rendered every frame, changed live through one API.

The motivating use case is **agentic interaction**. A user talks to an agent
(ideally by voice), and the agent arranges, renders and rearranges what is on
screen for the task at hand and the user's taste — a morning briefing, a data
exploration workspace, a coding view — and changes it live while the user
works.

Agents are the motivation, not a requirement. The same canvas must be able to
host a pixel-perfect Windows XP desktop, a keyboard-driven tiling setup, a
KDE-like workflow, or a kind of human–machine interaction nobody has built
yet. Void ships none of them; it makes all of them possible.

## Principles

1. **Everything is a plugin.** The core has no policy: no layout, no focus
   model, no keybindings, no decorations, no defaults. Even the window
   manager is a plugin. A core without plugins shows a black screen.
2. **Safety is the only built-in behaviour.** The one exception to "no
   defaults": the core always keeps an escape hatch (VT switch, emergency
   reset to a safe state, quit) that no plugin can override, because agents
   will write broken code live.
3. **Lean core, fast stability.** The core stays as small as possible so it
   reaches feature completeness quickly and can then be hardened. Features
   grow in plugins, not in the core.
4. **Be lazy: reuse before writing.** Wayland protocol handling, rendering,
   plugin runtimes, widgets, shells, services — take what exists, glue the
   pieces together, contribute upstream. Write only what is missing. Every
   decision gets a critical check for existing alternatives.
5. **One API, many callers.** Plugins, scripts, the CLI, external services
   and agents all speak the same scene language. What a human can configure,
   an agent can change — live.
6. **Live and real-time.** Changes apply while the session runs, without
   restart and without noticeable latency. Per-frame logic runs in-process.
7. **Declarative and versionable.** Views and flavors are plain YAML files:
   readable, diffable, kept in git, easy for an agent to read and write.
8. **Isolation by default.** Plugins are sandboxed; services run in
   containers; capabilities (read scene, inject input, install plugins) are
   granted explicitly.

## Goals

- An experimentation platform first, quickly reaching a stable, feature-complete
  core that can enter testing.
- Runs as an additional session on a Fedora-based host — concretely a derived
  [secureblue](https://secureblue.dev/) image — next to the existing desktops.
- Functionality is added as plugins and as containerized services (Podman,
  distrobox).
- A coordinating instance installs and wires plugins and services: at first
  the user, later an agent with dedicated permissions that can also *write*
  new plugins and services.
- Any existing workflow can be replicated (with effort); new ones can be
  invented.

## Non-goals (for now)

- Shipping a usable desktop out of the box. Flavors may do that; the core never.
- Agent memory and a local inventory of reusable elements are a **separate,
  opinionated layer** on top of Void, not part of the platform (see
  [Idea backlog](ideas.md)).
- Distributions other than Fedora/secureblue, and hardware other than what the
  author uses, until the core is stable.
