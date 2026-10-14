# Vision and principles

*chora is the space where forms appear.*

## The name

Plato calls the space in which things come into being *chora* (χώρα). chora
is neither form nor matter. It only gives the forms a place. The compositor
works the same way: it has no content and no policy of its own. Plugins,
clients and agents bring the forms.

## The idea

A desktop decides the arrangement of the content: windows, panels,
workspaces. chora takes this decision out of the compositor. chora is a
programmable scene. It works more like a game engine than like a window
manager. The core renders a scene of nodes every frame. One API changes the
scene live.

The motivating use case is agentic interaction. The user talks to an agent,
ideally by voice. The agent arranges the screen for the current task and for
the taste of the user. Examples are a morning view, a data exploration view
and a coding view. The agent changes the arrangement while the user works.

Agents are the motivation, not a requirement. The same scene can host a
pixel-perfect Windows XP desktop, a keyboard-driven tiling setup or a
KDE-like workflow. It can also host a kind of human-machine interaction that
does not exist yet. chora ships none of them. chora makes all of them
possible.

## Principles

1. **Everything is a plugin.** The core has no policy: no layout, no focus
   model, no keybindings, no decorations, no defaults. The window manager is a
   plugin too. A core without plugins shows a black screen.
2. **Safety is the only built-in behavior.** The core keeps an escape hatch
   that no plugin can override: VT switch, quit, and an emergency reset.
   Agents will write broken code live, so this exception is necessary.
3. **Lean core, fast stability.** A small core reaches feature completeness
   fast. Then the project hardens it. Features grow in plugins, not in the
   core.
4. **Reuse before writing.** Wayland protocol handling, rendering, widgets,
   shells and services already exist. chora connects these parts and writes
   only what is missing. Fixes go upstream. Every decision gets a check for
   existing alternatives.
5. **One API for every caller.** Plugins, scripts, the CLI, services and
   agents use the same scene language. An agent can change everything that a
   user can configure, and it can change it live.
6. **Live and real time.** Changes apply while the session runs. They need no
   restart and cause no noticeable latency.
7. **Declarative and versioned.** The configuration is plain text in git:
   views in YAML, and flavors as bundles of files. A diff shows each change.
   An agent can read and write the files easily.
8. **Isolation by default.** Plugins run as separate processes, services in
   containers. Input injection and screen capture through the API need a
   grant of the user (D33).

## Goals

- Start as an experimentation platform. Reach a stable, feature-complete core
  fast, then start testing.
- Run as an additional session on a Fedora host, next to the existing
  desktops. The concrete host is a derived [secureblue](https://secureblue.dev/)
  image.
- Add features as plugins and as services in containers.
- Let a coordinator install and connect plugins and services. The first
  coordinator is the user. A later coordinator is an agent with special
  permissions that can also write new plugins and services.
- Make it possible to rebuild any existing workflow, with effort, and to
  invent new ones.

## Non-goals for now

- A usable desktop out of the box. A flavor can supply one. The core never
  does.
- Agent memory and a local inventory of reusable elements. These form a
  separate, opinionated layer on top of chora (see [Idea backlog](ideas.md)).
- Distributions other than Fedora and secureblue, and GPUs other than AMD and
  Intel, until the core is stable.
