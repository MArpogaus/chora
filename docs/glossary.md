# Glossary

The shared vocabulary of chora. Code, docs and prompts to agents use these
terms and no synonyms.

| Term | Meaning |
|------|---------|
| **Core** | The compositor process `chora`. It handles the Wayland protocols, the backends (DRM/KMS, libinput), the scene, the renderer, the input routing, the IPC socket and the safety layer. It contains no policy. |
| **Scene** | The tree of nodes that the core renders. It is the single source of truth for the screen content. |
| **Node** | One element of the scene. The kinds are **group**, **surface**, **rect**, **image**, **shader** and **text**. Every node has an id, an owner, a transform, an opacity and optional effects. |
| **Group** | A node that contains other nodes. It applies a transform, a clip, an opacity and optional effects to its children. |
| **Plane** | A node with a size whose transform, relative to its nearest layer, is not a 2D translation with a positive scale. The size is its `size` or its content: the geometry of a window, the file of an image, the layout of a text. The core draws it through a texture and maps pointer input through the inverse of its transform. |
| **Floating window** | A window of a window manager on a virtual output that shows on a plane of its own in front of the shape that shows the output, such as a face of the cube of the construct. The window manager hides it in the output with opacity 0 for that time (`windows.float`), and the plane reads its node as a node texture. |
| **3D context** | A group with a `size` and a `perspective`. Its descendants share one space with an eye in front of its centre: the core sorts and splits their planes, cuts them at the near plane and gives pointer input to the nearest plane on the ray from the eye. |
| **Canvas** | A group that serves as a working area: a workspace, a desktop or an infinite zoomable plane. |
| **Surface** | A node that shows a Wayland client: an application window, a layer-shell widget or a browser window. For the core, a surface is a texture source and an input target. |
| **Rect** | A colored rectangle with optional rounded corners and border. |
| **Shader** | A node that a fragment shader draws completely. A shader can read other nodes as textures. A shader can also be an **effect**. |
| **Effect** | A shader that changes how a node and its children look, for example blur or a color change. Every node can have effects. |
| **Backdrop** | The frame behind a rect, blurred, which the rect draws under its own color, the tint: the glass of a panel. The watchdog removes it like an effect. |
| **Camera** | The part of the scene that one output shows: a transform from scene coordinates to the logical coordinates of the output. |
| **Claim** | A scene area of one window manager (D55). |
| **Virtual output** | An output that a client adds and that no screen shows. Its frames go into a texture that a shader node of its owner reads; a window manager runs on it as on a screen. |
| **Work area** | The part of the scene that an output shows, without the zones that panels and IPC clients reserve. Plugins place windows in it. |
| **Client** | A process on the IPC socket: a plugin, `choractl` or a service. A client names itself when it connects. |
| **Owner** | The client name that created or attached a node. Only the owner changes the node. |
| **Toplevel** | A surface with the role of an application window (`xdg_toplevel`). Other roles are layer surfaces and popups. |
| **Damage-less attach** | A commit that attaches an shm buffer of the same size and format as the buffer before, with no damage inside the buffer. The core keeps the texture of the surface as it is, a leniency that foot relies on. |
| **Detached** | The state of a surface node without a parent. A detached surface is not visible. |
| **View** | A declarative arrangement: nodes, the plugins that drive them, and their configuration. A view is a YAML file in git. The user or an agent changes it live. |
| **Plugin** | A client that reacts to events, changes the scene and can export methods. A window manager, a layout, an animation set and a keybinding set are plugins. |
| **Service** | A process in a container, outside the core. Examples are speech recognition, a local LLM and a development environment. |
| **Recording** | Frames of a headless session with a stepped scene clock: each frame shows the scene exactly one frame time later, whatever the speed of the renderer. `choractl record` makes one through the admin socket. |
| **Construct** | A YAML file of `chora-construct` with the start pose of the camera, the leash radius and the panes of one space (D66). |
| **Pane** | A flat area that stands in the space of `chora-construct` with a fixed turn, set to face the camera where it is placed or moved. It holds a program, a plugin, a nested desktop session or a prompt. "Pane" is not "view": a view is a declarative arrangement of the configuration (D66). |
| **Focus border** | The 2 px border that `chora-tiling` draws outside each window: a gradient from the accent to the second accent on the focused window, the line colour on the others, the urgent colour on a window that asks for attention. |
| **Key list** | The glass panel that `chora-tiling` shows on `MOD+slash` and on the Keys button of the panel: every binding with its words, built from the file. |
| **Layout chip** | The small glass chip that `chora-tiling` shows for 700 ms and that names the new layout. |
| **Flavor** | A preconfigured bundle of views, plugins and services, for example "classic", "tiling" or "agentic". |
| **Block** | A neutral plugin of the XP split that does one job for any flavor: notifications, assistant, desktop, panel or stacking window manager (D73). |
| **Asset folder** | The folder from which a block loads its pictures and a few values. The block ships a neutral one, and a flavor ships another (D73). |
| **Card** | The rounded rectangle in which `chora-notify` shows a notification, in a corner of an output. Its colors, corners and optional tail are values of the asset folder. |
| **Bubble** | The rounded rectangle with an optional tail that `chora-assistant` draws around a sentence of the voice agent. `chora-draw` draws its outline for the card and the bubble. |
| **Figure** | What `chora-assistant` shows for the voice agent: the dot of the neutral HUD folder, or the paperclip of the XP flavor. |
| **Question card** | What `chora-assistant` draws for `voice.confirm`: prompt, detail, key line and the buttons Yes and No. The host draws it with its own fonts, on top of every panel; no HUD folder styles, moves or hides it. See `hud.md`. |
| **Muted badge** | The label "Microphone off. Click to turn it on." that `chora-assistant` shows while the mic is muted. A click of a person on it turns the mic on. |
| **HUD folder** | A folder with `hud.yaml` (version 1), SVG pictures and optional fonts that `chora-assistant` draws as the look of the voice agent and of the chat. It holds data only and runs no code. See `hud.md`. |
| **HUD context** | The fields that decide which HUD folder and which panels show: `wm`, `space`, `app`, `state`, `agents`, `chat`, `output` and `theme`. The setting `huds` maps it to folders; it stays while a question waits or the chat field has the keyboard. See `hud.md`. |
| **Helper row** | A row that `chora-assistant` draws for a helper of the voice agent: its name, its state word and its task, as `Atlas · working · Write file (2/2)`. A click opens the chat with the helper. See `hud.md`. |
| **Chat slot** | The box of a window manager that holds the chat view: a HUD folder names the client with `chat.slot`, and the client announces the box with its event `chat.slot`. mc uses its drawer. See `hud.md`. |
| **Chat field** | The text field of the chat: `chora-chat --input NAME`, a layer surface that the host starts. Enter sends, Esc closes it and keeps the draft. It answers no question. See `voice.md`. |
| **Coordinator** | The party that installs and connects plugins, services and flavors. First the user, later an agent with special permissions. |
| **Inventory** | A local, git-versioned store of reusable elements (views, plugins, widgets). It belongs to the opinionated layer above chora, not to the core. |
| **Command** | A request to change the scene, for example `create` or `update`. |
| **Event** | A message from the core about a change, for example `surface.created`. |
| **Selector** | An expression that finds nodes by id, attribute or meta data, for example `app_id:firefox`. |
| **Transaction** | A batch of commands that the core applies as one unit. If one command fails, the core applies none of them. |
| **Capability** | A right of a client beyond the scene API: `input.inject`, `screen.capture`, `input.grab-any`, `methods.call` and `activation.token`. A client socket fixes the capabilities of its name (D33). |
| **Grant** | A capability that a client name holds: from `choractl grant` on the main socket, or from `clients.yaml` for a client socket. |
| **Milestone** | A minor release on the way to 1.0, with a goal, a scope and a gate. See [Roadmap](roadmap.md). |
| **Gate** | The checks that end a milestone; see [Roadmap](roadmap.md). |
| **Safety layer** | The only built-in behavior of the core: fixed bindings, emergency reset, session lock, watchdog. |
