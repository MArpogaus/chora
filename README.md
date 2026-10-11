# Void

A policy-free, programmable Wayland compositor: a canvas, not a desktop.

Void ships no window manager, no layout, no keybindings, no look. It provides
a small scene of primitives, a render loop, Wayland protocol handling and one
API. Everything else — a stacking desktop, a tiling workflow, a futuristic
voice-driven workspace arranged live by an agent — is a plugin.

> **Status:** concept phase. This repository holds the working documents
> that will become the official documentation. No code yet.

## Guiding principle

**Everything is a plugin.** Without plugins, Void shows a black screen and
does nothing but keep the session safe.

## Documentation

| Page | What it covers |
|------|----------------|
| [Vision and principles](docs/vision.md) | why Void exists, goals, non-goals |
| [Glossary](docs/glossary.md) | the shared vocabulary: node, view, plugin, service, flavor, inventory |
| [Scenarios](docs/scenarios.md) | classic desktop, tiling, agentic workday — and what each costs |
| [Architecture](docs/architecture.md) | core, scene, plugin host, IPC, MCP bridge, frame and input flow |
| [API](docs/api.md) | scene commands, events, IPC methods, plugin interface, MCP tools |
| [Configuration](docs/configuration.md) | YAML views and flavors |
| [Project structure](docs/project-structure.md) | planned crates and repository layout |
| [Dependencies](docs/dependencies.md) | what we reuse and why |
| [Alternatives](docs/alternatives.md) | existing compositors and runtimes, critically compared |
| [Decisions and open questions](docs/decisions.md) | the decision log |
| [Idea backlog](docs/ideas.md) | everything collected, not yet decided |

The rendered site: <https://marpogaus.github.io/void/>.

## Building the docs locally

```bash
uvx --with-requirements docs/requirements.txt mkdocs serve
```
