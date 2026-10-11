# Glossary

The shared vocabulary of Void. Code, docs and prompts to agents use these
terms and no synonyms.

| Term | Meaning |
|------|---------|
| **Core** | The compositor process. It handles the Wayland protocols, the backends (DRM/KMS, libinput), the scene, the renderer and the IPC socket. It contains no policy. |
| **Scene** | The tree of nodes that the core renders every frame. It is the single source of truth for the screen content. |
| **Node** | One element of the scene. The kinds are **group**, **surface**, **rect**, **image** and **shader**. Every node has an id, a transform, an opacity and optional effects. |
| **Group** | A node that contains other nodes. It applies a transform, a clip, an opacity and optional effects to its children. |
| **Canvas** | A group that serves as a working area: a workspace, a desktop or an infinite zoomable plane. |
| **Surface** | A node that shows a Wayland client: an application window, a layer-shell widget or a webview. For the core, a surface is a texture source and an input target. |
| **Rect** | A colored rectangle with optional rounded corners and border. |
| **Image** | A texture from a file or a buffer. |
| **Shader** | A node that a fragment shader draws completely. A shader can also be an **effect** on a group or a surface. |
| **Effect** | A shader that changes how a node and its children look, for example blur or a color change. |
| **View** | A declarative arrangement: nodes, the plugins that drive them, and their configuration. A view is a YAML file in git. The user or an agent changes it live. |
| **Plugin** | Logic that reacts to events and changes the scene. A window manager, a layout, an animation and a keybinding set are plugins. See [Decisions](decisions.md) for the plugin runtime. |
| **Service** | A process outside the core, usually in a container (Podman, distrobox). Examples are speech recognition, a local LLM, a webview host and a development environment. A service uses the IPC socket, appears as a Wayland client, or both. |
| **Flavor** | A preconfigured bundle of views, plugins and services, for example "classic", "tiling" or "agentic". |
| **Coordinator** | The party that installs and connects plugins, services and flavors. First the user, later an agent with special permissions. |
| **Inventory** | A local, git-versioned store of reusable elements (views, plugins, widgets). It belongs to the opinionated layer above Void, not to the core. |
| **Capability** | A permission of an IPC client or a plugin, for example to read the scene, to inject input or to capture the screen. |
| **Transaction** | A batch of scene commands that the core applies as one unit. If one command fails, the core applies none of them. |
