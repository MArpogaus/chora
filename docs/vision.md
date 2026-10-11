# Vision and principles

## The idea

A desktop decides how content is arranged: windows, panels, workspaces. Void
takes this decision out of the compositor. Void is a programmable scene. It
works more like a game engine than like a window manager. The core renders a
scene of nodes every frame, and one API changes the scene live.

The motivating use case is agentic interaction. The user talks to an agent,
ideally by voice. The agent arranges the screen for the current task and for
the taste of the user. Examples are a morning briefing, a data exploration
workspace and a coding view. The agent changes the arrangement while the user
works.

Agents are the motivation, not a requirement. The same scene can host a
pixel-perfect Windows XP desktop, a keyboard-driven tiling setup or a
KDE-like workflow. It can also host a kind of human-machine interaction that
does not exist yet. Void ships none of them. Void makes all of them possible.

## Principles

1. **Everything is a plugin.** The core has no policy: no layout, no focus
   model, no keybindings, no decorations, no defaults. The window manager is a
   plugin too. A core without plugins shows a black screen.
2. **Safety is the only built-in behavior.** The core keeps an escape hatch
   that no plugin can override: VT switch, quit, and an emergency reset to a
   safe state. Agents will write broken code live, so this exception is
   necessary.
3. **Lean core, fast stability.** A small core reaches feature completeness
   fast and can then be hardened. Features grow in plugins, not in the core.
4. **Reuse before writing.** Wayland protocol handling, rendering, plugin
   runtimes, widgets, shells and services already exist. Void connects these
   parts and writes only what is missing. Fixes go upstream. Every decision
   gets a check for existing alternatives.
5. **One API for every caller.** Plugins, scripts, the CLI, services and
   agents use the same scene language. An agent can change everything that a
   user can configure, and it can change it live.
6. **Live and real time.** Changes apply while the session runs. They need no
   restart and cause no noticeable latency.
7. **Declarative and versioned.** Views and flavors are plain YAML files. They
   are readable, can be compared line by line and live in git. An agent can
   read and write them easily.
8. **Isolation by default.** Plugins run in a sandbox. Services run in
   containers. The coordinator grants each capability explicitly, for example
   to read the scene, to inject input or to install plugins.

## Goals

- Start as an experimentation platform. Reach a stable, feature-complete core
  fast, then start testing.
- Run as an additional session on a Fedora host, next to the existing
  desktops. The concrete host is a derived [secureblue](https://secureblue.dev/)
  image.
- Add features as plugins and as services in containers (Podman, distrobox).
- Let a coordinator install and connect plugins and services. The first
  coordinator is the user. A later coordinator is an agent with special
  permissions that can also write new plugins and services.
- Make it possible to rebuild any existing workflow, with effort, and to
  invent new ones.

## Non-goals for now

- A usable desktop out of the box. A flavor can supply one. The core never
  does.
- Agent memory and a local inventory of reusable elements. These form a
  separate, opinionated layer on top of Void (see [Idea backlog](ideas.md)).
- Distributions other than Fedora and secureblue, and hardware other than the
  hardware of the author, until the core is stable.
