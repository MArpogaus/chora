# Decisions

The decision log. It lists the decisions that hold. Every entry can come
back for review. The [Roadmap](roadmap.md) names the milestone of a decision
that the code does not implement yet.

## Principles and scope

| # | Decision | Reason |
|---|----------|--------|
| D1 | Rust and Smithay for the core | Smithay is a library without policy, proven in niri and cosmic-comp. Rust is safe when agents change the system live. |
| D2 | Everything is a plugin | Maximum flexibility and a small core. |
| D3 | chora 1.0 is a daily driver with a stable, versioned API; until then, the API changes without notice | A stable API needs real use first. No compatibility code before 1.0. |
| D4 | Reuse before writing; check alternatives for every decision | Little code to maintain. |
| D5 | One vocabulary, defined in the [Glossary](glossary.md) | One language for people and agents. |
| D6 | Inventory and agent memory form a separate layer, not part of the core | The platform stays neutral. |
| D7 | The name is chora, always in lowercase | See [Vision](vision.md#the-name). |
| D8 | The core is GPL-3.0-or-later; code under GPL-3.0-only stays out | The core can then reuse code from Pinnacle and niri. |
| D9 | English for code and documentation | The documentation is public. |

## Platform

| # | Decision | Reason |
|---|----------|--------|
| D10 | Fedora 44 and secureblue as host; a derived image offers chora as an additional session | The system and the image tooling of the author. |
| D11 | The reference hardware is the HP ZBook Ultra G1a (AMD Radeon 8060S); CI adds a VM with `vkms`; Intel works through Mesa without a gate; NVIDIA comes later | Gates need one machine that the author uses every day. The VM covers the tty backend in CI. |
| D12 | Services run as Quadlet units with Podman or in distrobox | Isolation. systemd handles start, restart and order. |
| D13 | The coordinator is the user first, later an agent with special permissions | A gradual path to control by agents. |
| D14 | Releases build an RPM and an Arch package, attached with SHA256 sums to a GitHub release; versions below 1.0 and release candidates are pre-releases | One build for the release assets and the package repositories of D49. |

## Core

| # | Decision | Reason |
|---|----------|--------|
| D15 | Build on Smithay; copy from Pinnacle, niri and anvil; no fork | A fork of Pinnacle keeps less than half of its code. See [Alternatives](alternatives.md#pinnacle-code-review). |
| D16 | Pin a Smithay git revision | No Smithay release since 0.7.0. All major users track git. |
| D17 | A scene renderer of chora on GLES 3, on the GL context of Smithay's `GlesRenderer`; GLSL shaders | Smithay's render elements are axis-aligned rectangles. A renderer of its own draws the full 4x4 model and keeps textures between frames. wgpu has no usable dmabuf path for a compositor. |
| D18 | Six node kinds: group, surface, rect, image, shader and text | Widgets come from Wayland clients. An image node shows a PNG asset, for example a window border, without a client per window. A shader can read another node as a texture. A text node lets plugins and agents label a HUD without a client or a rendered PNG. cosmic-text does the shaping and the font fallback, so the node stays small: text, font family, size, color and maximum width; no editing and no rich text. |
| D19 | Transforms are 4x4 matrices; the scene is 2.5D: planes in perspective, meshes and fragment shaders, without a depth test | 3D effects need no API change. Complex 3D scenes belong in clients. |
| D20 | One logical scene; each output shows it through a camera | Pan, zoom and the infinite canvas need no extra mechanism. |
| D21 | The core places popups, subsurfaces and layer surfaces; toplevels wait for a plugin | The protocols define these positions. Smithay implements them. |
| D22 | A transaction completes when all affected clients commit, or after a deadline; transactions on disjoint surfaces do not wait for each other | A resize needs a client round trip. One slow client must not stop the others. |
| D23 | A hardcoded safety layer: Ctrl+Alt+Backspace quits, Ctrl+Alt+Escape resets, Ctrl+Alt+F1 to F12 switch the VT, SIGUSR1 resets while the main loop runs; a watchdog that blames the client whose visible nodes cost the most for slow frames, removes its effects and then disconnects it, and never resets the session (D57); a watchdog thread on a heartbeat of the main loop for a hung loop (D64); a binding that revokes the capabilities of all clients | A recovery path must not depend on a file that an agent can break, or on a main loop that hangs. One client without grants must not take the window manager, the grants and the effects of the others down with it. |
| D24 | The core renders an output only when it has damage; a layer keeps its texture until its subtree changes; a fullscreen surface without effects uses direct scanout | Battery life and GPU load. |
| D25 | No undo history and no plugin state store in the core | A destroyed client cannot come back. The reset and the views in git cover recovery. The scene holds all state. |
| D26 | Xwayland through `xwayland-satellite` | Keeps the core small. |
| D27 | Output configuration through `wlr-output-management` and through `output.configure`, on the same path | kanshi and wlr-randr exist. An agent has no wlr-randr in every session. |
| D64 | A watchdog thread watches a heartbeat that the main loop bumps after each dispatch; the loop wakes at least once a second. After 3 s without a beat, the thread logs the stall with the action that follows. After `CHORA_HANG_ABORT_S` seconds without a beat, 30 by default, it aborts chora with SIGABRT and a core dump, so that the session can start again; 0 means never abort. The stall and the abort lines go to fd 2 with a raw `write(2)`, without the lock of tracing or of `std::io::stderr`. The SIGABRT handler then prints the backtrace of the watchdog thread, not that of the main loop; the core dump has all threads. The frame watchdog of D57 stays on the main loop | A hung shader or driver blocks the main loop, so the frame watchdog and SIGUSR1 cannot act. Only another thread sees the hang. An abort ends the session, so the user can start it again, and leaves a core dump for the bug. 3 s is above the idle wake-up of 1 s and below the 5 s of the 0.9 gate. A main loop that hangs inside a log write holds the log lock, and the watchdog must not wait for it. A hang of 30 s is longer than any slow frame of llvmpipe, and the user waits no longer than that. |

## Plugins and API

| # | Decision | Reason |
|---|----------|--------|
| D28 | Plugins are IPC clients; the core offers `animate` and frame events with the predicted presentation time; in-process WASM plugins come only after a measurement shows the need | Agents work in seconds, not frames. One mechanism, any language, crash isolation by the OS. `tests/ipc-bench.sh` measures a transaction with a mesh of 8x8 cells at 0.17 ms in the median and 3.6 ms at the 99th percentile, nested and with software rendering. |
| D29 | JSON-RPC 2.0 with one message per line on Unix sockets; `hello` negotiates the protocol version and the features; errors carry a stable code and data | Agents and shell scripts reach it without bindings. A versioned protocol lets plugins in other repositories check what they get. |
| D30 | The core starts one startup command with `WAYLAND_DISPLAY` and `CHORA_SOCKET` set | The startup command starts the plugins, or a systemd target. |
| D31 | The owner of a node is a client name; only the owner changes a node; another client requests a change, and the core sends the request to the owner; plugins export methods with a description and a JSON schema | An agent asks the window manager instead of fighting it. |
| D32 | Queries use JSON filters; a query returns all current matches; subscriptions start with a snapshot, and events have sequence numbers | A schema describes the filters. Agents get predictable results and miss no event. |
| D33 | Each client has a socket directory with an IPC socket and a Wayland socket of its own that hides the privileged protocols; the socket fixes its name and its capabilities; `input.inject`, `screen.capture`, `input.grab-any`, `methods.call`, `methods.privileged`, `activation.token` and `session.quit` are capabilities | A client cannot take another name or give itself capabilities. A container mounts one directory and needs no configuration. |
| D34 | MCP outside the core: `choractl mcp` over stdio on rmcp, with tools for whole tasks and one tool per exported plugin method | The core stays free of MCP. The official SDK carries the protocol. |
| D35 | The core reads no configuration; `choractl` loads YAML views and `~/.config/chora/clients.yaml`; a flavor is a directory with a systemd target; the configuration holds no secrets | Policy and service management do not belong in the core. |
| D36 | The GitHub organization is `chora-desktop` (`chora` is taken). It holds `chora` (the core, `chora-scene`, `choractl`, the reference documentation), the plugins in repositories of their own, `chora-website`, `chora-packaging`, `chora-voice` and `.github`. The organization and this split come after 1.0; until the split, the core repository holds the core, the plugins and the services, and they build with it | Plugins and services get their own owners and releases once the API of `chora-scene` is frozen at 1.0. The core keeps the definition of the API. |
| D58 | A client changes how a node of another client looks, its opacity, effects or meta, only through the owner: `node.requested` has the action `update` (D31). The window manager applies the change and keeps it across layouts. Clients on a confined client socket can send the request `update`, as `move`. An `update` that mixes owner-only props with opacity, effects or meta fails with an error that names the owner-only props. An `update` for an owner that does not subscribe to `node.requested` fails, because nobody would apply it. Requests for `animate` are not part of D58 | The owner stays the only client that changes its nodes. A look that a new layout drops does not hold. |
| D59 | The answer of `scene.apply` carries `warnings` with stable codes, for example for a surface attached under a group with `input: pass`, or for a layer whose subtree (the layer and every layer under it) needs more pixels than the budget of every output; a drop at render time stays the event `layers.dropped` | Some transactions succeed but do not give the result that the client expects. A warning tells the client without a capture. |
| D61 | `choractl mcp` keeps `keep_nodes`; after a restart, an agent removes its own old nodes (#101) | No change in the core or in `choractl`. An agent knows which of its nodes still matter. |
| D62 | Each plugin exports `settings.get` and `settings.set` through the plugin runtime of `chora-kit`, with a schema in `methods.export`. The plugin checks the values, applies them at once and then saves them in its YAML file in `$XDG_CONFIG_HOME/chora/`, for example `chora-tiling.yaml` and `chora-xp.yaml`; a change that cannot be applied is not saved, and a change that cannot be saved stays applied with an error to the caller. The plugin is the only writer of its file. A save renames a new file of mode 0600 over the old one; through a symlink, the save replaces the file that the symlink points to, so the symlink stays. Edits by hand apply with `settings.reload`, which the runtime exports with the other two methods and which reads the file again, or at the next start. `settings.set` takes a JSON merge patch (RFC 7386): structs and maps merge, other values replace, and `null` removes a map entry or resets a setting to its default. A setting that the file leaves out takes its default. A save writes only the settings that differ from the defaults: a changed map whole, and `null` for an option that the user cleared. A file that does not parse or check does not stop the plugin: it starts with the defaults, logs the error and leaves the file as it is, and `settings.set` and every other save refuse until a `settings.reload` succeeds. The file replaces the command-line flags of `chora-xp` except `--theme`, `--config`, `--slow-motion` and `--output`. On a client socket, `settings.set` and `settings.reload` need the capability `methods.privileged` besides `methods.call`, because the settings hold commands that the plugin runs; so does each method that its export marks `"privileged": true`, such as `app.open` and `scene.save` of `chora-mc`, which start programs. `choractl mcp` confines its connection of the main socket with `hello {confined: true}`, so these rules and `activation.token` hold for an MCP agent too until the user grants its name. A generic settings form is not part of 0.8 | One interface serves `choractl`, MCP agents and the dialogs of the plugin. The file keeps the settings. |
| D65 | A client emits its own events with `events.emit {event, data}`. The name is the client name, a dot and a name of the client, for example `voice.state`; the core refuses any other name. A client whose name starts as a namespace of the core events, such as `surface` or `output`, emits nothing. The core sends the event with `client` and `data` to the connections that subscribed to its name or to a pattern `PREFIX.*`, such as `voice.*`. An empty subscription means every core event, not the events of clients. In `events.subscribe`, a name outside the core namespaces is the event of a client; a typo in a core name stays refused. The data has at most 64 KiB. A client name emits at most 32 events per second; the core refuses the rest with code 6 and does not coalesce, so a client sends the last state of a burst again. Any named client may emit, also through a client socket without capabilities, because an event only reaches subscribers. Any connection may subscribe to the events of other clients, by the same rules as core events, also a confined client socket. The voice agent emits `voice.state` (the `agent.set` fields; with `name`, a sub-agent such as `coder`) and `voice.say` (a sentence when its sound starts) and `voice.confirm` (a question with its full command, then its end) on its connection `voice`; `chora-mc` and `chora-xp` subscribe to `voice.*`; `chora-mc` keeps `agent.set` and `hud.say` for other clients | A HUD needs the state of an agent when it changes, without polling and without a method of the HUD in the agent. The prefix makes the emitter visible: no client fakes a core event or the event of another name. A name of the main socket is still taken by whoever connects first; a client socket fixes it (D33). The limits protect the 8 MiB output buffer of a subscriber. A refusal tells the emitter; a silent drop would not. |

## Tools around the core

| # | Decision | Reason |
|---|----------|--------|
| D37 | Screen sharing through `xdg-desktop-portal-wlr` | No code. A portal backend of its own can come later. |
| D38 | HTML content from agents runs in a browser in app mode, as a service | No code. chora places toplevels anywhere. |
| D39 | The XP showroom is a flavor in the style of Windows XP with wobbly windows, a magic lamp minimize and a 3D desktop cube; it lives in the core repository (D36); it is a demo on top of the generic plugins, not a daily driver; it starts installed programs, draws no programs into the scene except the dialogs of D63, and draws its own wallpaper, one per cube face; after its rework in 0.8, it gets fixes and the API ports of the core only, and no release per milestone | It proves that the scene model carries classic desktops and their effects, and it tests every API change. The assets come from `kde-plasma-winxp-launcher` and keep their licenses; the GPL-3.0-only artwork of Exposé and modernXP is data of the flavor, not code of the core (D8). |
| D40 | The user gives capabilities in `~/.config/chora/clients.yaml`; `choractl clients up` creates the client sockets through an admin socket that only `choractl` uses; the emergency reset removes all client sockets; `choractl mcp` has no tool for capabilities | The core draws no text, so a dialog is no option. Only the user changes the capabilities. |
| D41 | `choractl` reads YAML with `serde-saphyr`: strict booleans, errors on unknown and duplicate keys and on unknown tags | `serde_yaml` is archived and `serde_yml` is unsound. `serde-saphyr` is maintained, pure Rust and panic-free, and limits the work of a document. |
| D42 | The core starts no program except the startup command. `choractl run` starts programs through `systemd-run --user` with an activation token, and journald keeps their output. Without a running user manager, `choractl run` starts the program as its own child. systemd units restart plugins, and `choractl restart` restarts such a unit. | systemd already supervises processes. Test sessions and sandboxes have no user manager; the fallback lives in `choractl`, so the core stays free of forks and one binary serves both. The activation token links a program to its window. |
| D43 | Nodes fixed to the screen live under one overlay root per output, `screen:NAME`: in the logical coordinates of the output, without its camera, above the scene and below the top and overlay layers of layer shell | A HUD stays in place while a camera moves. A root per output answers on which output a node shows. Its nodes work as all others: input, effects, ownership. |
| D44 | `output.configure` sets scale and transform of an output through the API, on the same path as `wlr-output-management` | One path keeps both ways in step. |
| D45 | Grabs need Ctrl, Alt or Super, except for a client with the capability `input.grab-any`; `input.ungrab` ends a grab; a key grab tells when the first of its modifiers goes up, except Shift; there is no exclusive keyboard grab | Typing works unless the user gives a capability, for example for push-to-talk. Games and 3D clients get every key through Wayland. |
| D63 | `chora-xp` keeps the dialogs of the XP shell, pixel-perfect: Log Off, Turn Off, Display Properties, Taskbar and Start Menu Properties, Date and Time Properties and the Control Panel menu. Each button of Log Off and Turn Off runs a command from the settings (D62). The defaults are `systemctl poweroff`, `systemctl reboot` and `systemctl suspend`, and Log Off ends the chora session with `choractl quit`, which calls the core method `session.quit`. The dialogs change the settings through `settings.set`. The Explorer, the folder windows, file Properties and the Delete confirmation leave the plugin. My Documents, My Computer, folder icons and the Recycle Bin open the file manager of the settings, by default `thunar`. The tag `archive/xp-scene-programs` keeps the removed code. D63 refines D39 | The dialogs are the showcase and the safeguard before a poweroff. A file manager drawn in the scene cannot match real file operations. |
| D66 | `chora-construct` grows from the loader prototype into a Rust plugin of the core repository, both a flavor and an overlay that mc or the voice agent loads. The space of a construct is a light, foggy space without a grid. The light theme is a pale sky over a warmer ground with a bright horizon; the dark theme is a black void over a dim ground. The user walks on a leash around the start point. Panes face the camera; they hold programs, plugins, nested desktop sessions or prompts in which an ACP agent builds a program live. Constructs are YAML files. Up to 8 panes are live; the rest freeze. Several outputs show one space as a panorama. "Pane" names the floating surfaces, because "view" names the views of the configuration | The loader showed that node textures, meshes, shaders and 4x4 transforms carry a 3D desktop without a change of the core. A plugin with a manifest, tests and settings replaces the untested prototype. |

## Process

| # | Decision | Reason |
|---|----------|--------|
| D46 | Each milestone has a gate: CI jobs and checks on the reference hardware with a log as evidence, tracked in a gate issue; the next milestone of the core starts when the gate holds | Breadth without real use hides faults. A gate makes "done" checkable. |
| D47 | The git flow of the project conventions: `main` holds `chore: init` and merge commits; work happens on `dev`, and each contributor of a shared repository has `dev-NAME`; a release is a merge into `main` and an annotated tag on the merge commit; the tag starts the release, which needs green CI of the same commit; commitizen writes the changelog; a review loop runs until a round finds nothing that matters; `main` accepts pull requests only from the move to the organization after 1.0 (D36) | One flow in every repository. A release never ships a red commit. |
| D48 | The core is a library and a binary; IPC, transactions, focus and grants have unit tests with a fake clock; a headless GLES backend on the software driver of Mesa runs pixel tests in process; each bug fix comes with a test that fails without the fix | Pixel tests in a headless sway cannot reach the internals. Review finds faults that tests should catch. |
| D49 | Packages through COPR with Packit, with a subpackage per plugin; no AUR packages | The image build and updates need a package repository. |
| D50 | The session starts through uwsm; a flavor is a systemd target. While uwsm is not in Fedora 44, the sessions without uwsm (`chora-session`) are the fallback | uwsm handles the environment and the graphical session target. |
| D51 | The daily driver of milestone 0.9 (D60) is `chora-tiling` with waybar, fuzzel, mako, swaybg, swaylock, swayidle and kanshi; panel and launcher plugins of chora come after 1.0 | Existing tools cover the shell. Own plugins are no condition for 0.9 or 1.0. |
| D52 | The milestones run 0.6 to 1.0 as in the [Roadmap](roadmap.md) | A stable API needs real use and a testable core first. |
| D53 | The voice pipeline lives in `chora-voice`, until the split after 1.0 in `services/voice/` of the core repository (D36), and runs in parallel to the core; the core supplies push-to-talk grabs, text injection, the read model and an agent indicator | The pipeline is a product of its own. It needs only the API of the core. |
| D54 | Plugins use the JSON-RPC API with frame events and fd passing; a private Wayland protocol for plugins comes only if a measurement at 120 Hz in milestone 0.10 shows the need | One API to maintain. The measurement decides. |
| D55 | Window managers share the scene through claims. A claim is a scene area of one window manager. A window manager hands a surface to another with `remove` and `to`. The core gives a new toplevel to a window manager (see [below](#d55-window-managers-per-claim)). | The user chose one window manager per claim over a single window manager role. Ownership, `attach` and `surface.detached` stay the mechanism, and a camera switches between claims. |
| D56 | The UI is hybrid. Plugins draw shell chrome and effects as scene nodes. Real applications, such as a file manager or a control panel, run as Wayland clients. Scene nodes take the keyboard through `focus_node`. In 0.8, `chora-xp` stops drawing programs into the scene, except its dialogs, and starts installed programs instead (D39, D63). The git tag `archive/xp-scene-programs` keeps the removed code | Chrome and effects need the scene: transforms, shaders, wobble. Applications need text input, accessibility and toolkits, which clients bring. |
| D57 | The watchdog disconnects a client for slow frames only when its own visible load reaches the owner budget, `CHORA_OWNER_LOAD_BUDGET` with a default of 4000. Below it the slowdown is shared: the watchdog removes the effects of all clients and disconnects none | The W5 battle disconnected a playing snake while a budget probe churned its nodes. The largest single load was not the fault. Two clients that are each half the fault should keep their connections, and the frame still gets cheap; a client alone over its budget is the clear culprit. |
| D60 | The daily driver comes before the renderer: milestone 0.9 is the daily driver and 0.10 the renderer | Daily use needs a stable session more than a faster renderer. 0.9 takes from the renderer only the watchdog thread, which detects a hung main loop. |

## Designs

### D55: window managers per claim

Issue #73 calls the area of a window manager a view. The glossary uses
"view" for a YAML arrangement (D5), so this design says **claim**. The
design has two steps. Step 1 is the handover. Step 2 adds claims and the
routing of new toplevels.

#### Step 1: handover

Window manager A hands surface `s1` to window manager B:

1. A owns `s1`, for example at the end of a drag out of its area. A knows
   B by a fixed name or by a `meta` tag on the root group of B.
2. A sends `remove: {id: s1, to: B}`. `to` must name a connected client
   that holds a claim, because only window managers receive windows, and
   a reservation that B does not attach within 5 seconds ends as in
   item 5. As at every `remove`, the surface becomes detached and loses the
   keyboard focus.
3. The core reserves `s1` for B and sends `surface.detached`, which
   carries the surface alone. An `attach` of another client fails.
4. B attaches `s1`, places it by its own rules, and can give it the
   focus. A plugin that adopts every detached surface needs no change.
5. When the last connection of B closes, the reservation ends. With
   `resume`, it ends after the grace time. The core then sends
   `surface.detached` to all, and any client can attach the surface.

**Focus and grabs.** The keyboard focus does not move with the surface.
The receiving window manager sets it with `focus` after the `attach`.
Grabs belong to connections and do not move. A ends its drag at the
release of the button, and then hands the surface over. The client of the
surface keeps its pointer and keyboard input as before.

#### Step 2: claims

A claim is a rectangle in scene coordinates: the property
`claim: [x, y, w, h]` of a node. The owner of the node holds the claim. A
window manager sets it on its root group, and the claim ends with its
node. Window managers claim the whole work area of their outputs. An
output shows a claim through its camera. So two window managers sit on two
outputs, or at two places in the scene, and a camera move switches one
output between them. With claims, A finds B in the claims of
`scene.query`.

At the first commit of a toplevel, the core picks its window manager.
The first choice is the owner of its parent toplevel, so that a dialog
stays with the manager of its window. The next choice is the owner of the
focused node, or of the focused surface, if it holds a claim. The last
choice is the owner of the smallest claim that contains the pointer. A
client that asked for an activation token (D42) does not win by the token
itself. The core
reserves the surface for that client, as at a handover. Outside of all
claims, any window manager can take the new surface.

**The drop target.** A drag that ends outside the claim of A goes to the
owner of the smallest other claim that holds the drop point; without one
there, the surface stays with A.

**Shared pointer bindings.** Window managers that each hold a claim can
grab the same pointer button binding, and a press goes to the grab of the
smallest claim under the pointer, else to the first grab, with its moves
and release; key bindings keep one owner.

**Out of scope for now.**

- The drop position: B places the surface by its own rules.
- A drag that continues in B after the handover.
- An embedded GNOME session in a claim.
- Claims under `screen:NAME`.

## Rejected proposals

| Proposal | Reason |
|----------|--------|
| Versioned docs with mike are too much | The project conventions use mike. |
| Hooks without matching files | The project conventions use this hook set in every repository. |
| Cairo and Pango, or another 2D engine, in the core | The core composites and draws no vector graphics. Text comes from the text node on cosmic-text (D18), shapes from rects and shader nodes. Plugins draw rich chrome themselves, for example with tiny-skia and resvg. |
| An exclusive keyboard grab | Typing must work. Clients that need every key get it through Wayland (D45). |
| A 3D node with depth buffer in the core | It makes the core an engine. 3D scenes belong in clients (D19). |
| A rewrite from scratch | The scene model, the API and the safety layer hold. The milestones change the rest step by step (D52). |
