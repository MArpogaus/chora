# Glossary

The shared vocabulary of Void. Use these terms consistently in code, docs and
when talking to agents.

| Term | Meaning |
|------|---------|
| **Core** | The compositor process: Wayland protocols, backends (DRM/KMS, libinput), the scene, the renderer, the plugin host and the IPC socket. Contains no policy. |
| **Scene** | The tree of nodes the core renders every frame. The single source of truth for what is on screen. |
| **Node** | One element of the scene. Kinds: **Group**, **Surface**, **Rect**, **Image**, **Shader**. Every node has an id, a transform, opacity and optional effects. |
| **Group** | A node that contains other nodes and applies transform, clipping, opacity and an optional shader effect to them. A *canvas*, a workspace, a desktop or an infinite zoomable plane is a group. |
| **Surface** | A node that shows a Wayland client (an app window, a layer-shell widget, a webview). For the core, a surface is a texture source plus an input target. |
| **Rect / Image** | A colored rectangle (rounded corners, border) / a texture from a file or buffer. |
| **Shader** | A node drawn entirely by a user-supplied fragment shader with uniforms. Also usable as an *effect* on a group or surface. |
| **Canvas** | Informal name for a group used as a working area. |
| **View** | A concrete, declarative arrangement: nodes, the plugins that drive them, and rules. Stored as YAML, versioned in git, changed live by the user or an agent. |
| **Plugin** | Logic loaded into the core, sandboxed (WASM on wasmtime, see [Decisions](decisions.md)). Reacts to events, runs per frame, changes the scene. Window managers, layouts, animations and effects are plugins. |
| **Service** | A process outside the core, usually in a container (Podman, distrobox): speech recognition, a local LLM, a webview host, a dev environment. Talks to the core over IPC or simply shows up as a Wayland client. |
| **Flavor** | A preconfigured bundle of views, plugins and services — e.g. "classic", "tiling", "agentic". |
| **Coordinator** | Whoever installs and wires plugins, services and flavors. First the user; later an agent with dedicated permissions. |
| **Inventory** | A local, git-versioned store of reusable elements (views, plugins, widgets) created during work. Part of the opinionated layer above Void, not the core. |
| **Capability** | A permission granted to an IPC client or plugin: read the scene, write the scene, inject input, capture the screen, install plugins. |
| **Transaction** | A batch of scene commands applied atomically, and rolled back as a whole if it fails. |
