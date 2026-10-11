# Glossary

The shared vocabulary of chora. Code, docs and prompts to agents use these
terms and no synonyms.

| Term | Meaning |
|------|---------|
| **Core** | The compositor process `chora`. It handles the Wayland protocols, the backends (DRM/KMS, libinput), the scene, the renderer, the input routing, the IPC socket and the safety layer. It contains no policy. |
| **Scene** | The tree of nodes that the core renders every frame. It is the single source of truth for the screen content. |
| **Node** | One element of the scene. The kinds are **group**, **surface**, **rect** and **shader**. Every node has an id, an owner, a transform, an opacity and optional effects. |
| **Group** | A node that contains other nodes. It applies a transform, a clip, an opacity and optional effects to its children. |
| **Canvas** | A group that serves as a working area: a workspace, a desktop or an infinite zoomable plane. |
| **Surface** | A node that shows a Wayland client: an application window, a layer-shell widget or a browser window. For the core, a surface is a texture source and an input target. |
| **Rect** | A colored rectangle with optional rounded corners and border. |
| **Shader** | A node that a fragment shader draws completely. A shader can read other nodes as textures. A shader can also be an **effect**. |
| **Effect** | A shader that changes how a node and its children look, for example blur or a color change. Every node can have effects. |
| **Camera** | The part of the scene that one output shows: a transform from scene coordinates to the logical coordinates of the output. |
| **Work area** | The part of the scene that an output shows, without the zones that panels reserve. Plugins place windows in it. |
| **Client** | A process on the IPC socket: a plugin, `choractl` or a service. A client names itself when it connects. |
| **Owner** | The client name that created or attached a node. Only the owner changes the node. |
| **Toplevel** | A surface with the role of an application window (`xdg_toplevel`). Other roles are layer surfaces and popups. |
| **Detached** | The state of a surface node without a parent. A detached surface is not visible. |
| **View** | A declarative arrangement: nodes, the plugins that drive them, and their configuration. A view is a YAML file in git. The user or an agent changes it live. |
| **Plugin** | A client that reacts to events, changes the scene and can export methods. A window manager, a layout, an animation set and a keybinding set are plugins. |
| **Service** | A process in a container, outside the core. Examples are speech recognition, a local LLM and a development environment. |
| **Flavor** | A preconfigured bundle of views, plugins and services, for example "classic", "tiling" or "agentic". |
| **Coordinator** | The party that installs and connects plugins, services and flavors. First the user, later an agent with special permissions. |
| **Inventory** | A local, git-versioned store of reusable elements (views, plugins, widgets). It belongs to the opinionated layer above chora, not to the core. |
| **Command** | A request to change the scene, for example `create` or `update`. |
| **Event** | A message from the core about a change, for example `surface.created`. |
| **Selector** | An expression that finds nodes by id, attribute or meta data, for example `app_id:firefox`. |
| **Transaction** | A batch of commands that the core applies as one unit. If one command fails, the core applies none of them. |
| **Grant** | A permission of the user for one client name to use `input.inject` or `screen.capture`. |
| **Safety layer** | The only built-in behavior of the core: fixed bindings, emergency reset, session lock, watchdog. |
