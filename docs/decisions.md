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
| D18 | Six node kinds: group, surface, rect, image, shader and text; the text node comes in milestone 0.8 | Widgets come from Wayland clients. An image node shows a PNG asset, for example a window border, without a client per window. A shader can read another node as a texture. A text node lets plugins and agents label a HUD without a client or a rendered PNG. cosmic-text does the shaping and the font fallback, so the node stays small: text, font family, size, color and maximum width; no editing and no rich text. |
| D19 | Transforms are 4x4 matrices; the scene is 2.5D: planes in perspective, meshes and fragment shaders, without a depth test | 3D effects need no API change. Complex 3D scenes belong in clients (see [Idea backlog](ideas.md#visuals)). |
| D20 | One logical scene; each output shows it through a camera | Pan, zoom and the infinite canvas need no extra mechanism. |
| D21 | The core places popups, subsurfaces and layer surfaces; toplevels wait for a plugin | The protocols define these positions. Smithay implements them. |
| D22 | A transaction completes when all affected clients commit, or after a deadline; transactions on disjoint surfaces do not wait for each other | A resize needs a client round trip. One slow client must not stop the others. |
| D23 | A hardcoded safety layer: Ctrl+Alt+Backspace quits, Ctrl+Alt+Escape resets, Ctrl+Alt+F1 to F12 switch the VT, SIGUSR1 resets; a watchdog in its own thread; a binding that revokes the capabilities of all agents | A recovery path must not depend on a file that an agent can break, or on a main loop that hangs. |
| D24 | The core renders an output only when it has damage; a layer keeps its texture until its subtree changes; a fullscreen surface without effects uses direct scanout | Battery life and GPU load. |
| D25 | No undo history and no plugin state store in the core | A destroyed client cannot come back. The reset and the views in git cover recovery. The scene holds all state. |
| D26 | Xwayland through `xwayland-satellite` | Keeps the core small. |
| D27 | Output configuration through `wlr-output-management` and through `output.configure`, on the same path | kanshi and wlr-randr exist. An agent has no wlr-randr in every session. |

## Plugins and API

| # | Decision | Reason |
|---|----------|--------|
| D28 | Plugins are IPC clients; the core offers `animate` and frame events with the predicted presentation time; in-process WASM plugins come only after a measurement shows the need | Agents work in seconds, not frames. One mechanism, any language, crash isolation by the OS. `tests/ipc-bench.sh` measures a transaction with a mesh of 8x8 cells at 0.17 ms in the median and 3.6 ms at the 99th percentile, nested and with software rendering. |
| D29 | JSON-RPC 2.0 with one message per line on Unix sockets; `hello` negotiates the protocol version and the features; errors carry a stable code and data | Agents and shell scripts reach it without bindings. A versioned protocol lets plugins in other repositories check what they get. |
| D30 | The core starts one startup command with `WAYLAND_DISPLAY` and `CHORA_SOCKET` set | The startup command starts the plugins, or a systemd target. |
| D31 | The owner of a node is a client name; only the owner changes a node; another client requests a change, and the core sends the request to the owner; plugins export methods with a description and a JSON schema | An agent asks the window manager instead of fighting it. |
| D32 | Queries use JSON filters; a query returns all current matches; subscriptions start with a snapshot, and events have sequence numbers | A schema describes the filters. Agents get predictable results and miss no event. |
| D33 | Each client has a socket directory with an IPC socket and a Wayland socket of `wp-security-context-v1`; the socket fixes its name and its capabilities; `input.inject`, `screen.capture` and `input.grab-any` are capabilities | A client cannot take another name or give itself capabilities. A container mounts one directory and needs no configuration. |
| D34 | MCP outside the core: `choractl mcp` over stdio on rmcp, with tools for whole tasks and one tool per exported plugin method | The core stays free of MCP. The official SDK carries the protocol. |
| D35 | The core reads no configuration; `choractl` loads YAML views and `~/.config/chora/clients.yaml`; a flavor is a directory with a systemd target; the configuration holds no secrets | Policy and service management do not belong in the core. |
| D36 | The GitHub organization `chora` holds the repositories `chora` (the core, `chora-scene`, `choractl`, the reference documentation), one repository per plugin, `chora-website`, `chora-packaging`, `chora-voice` and `.github` | Plugins and services get their own owners and releases. The core keeps the definition of the API. |

## Tools around the core

| # | Decision | Reason |
|---|----------|--------|
| D37 | Screen sharing through `xdg-desktop-portal-wlr` | No code. A portal backend of its own is in the [Idea backlog](ideas.md). |
| D38 | HTML content from agents runs in a browser in app mode, as a service | No code. chora places toplevels anywhere. |
| D39 | The XP showroom is a flavor in the style of Windows XP with wobbly windows, a magic lamp minimize and a 3D desktop cube, in its own repository, in active development; it is a demo, not a daily driver | It proves that the scene model carries classic desktops and their effects, and it tests every API change. The assets come from `kde-plasma-winxp-launcher` and keep their licenses; the GPL-3.0-only artwork of Exposé and modernXP is data of the flavor, not code of the core (D8). |
| D40 | The user gives capabilities in `~/.config/chora/clients.yaml`; `choractl clients up` creates the client sockets through an admin socket that only `choractl` uses; the emergency reset removes all client sockets; `choractl mcp` has no tool for capabilities | The core draws no text, so a dialog is no option. Only the user changes the capabilities. |
| D41 | `choractl` reads YAML with `serde-saphyr`: strict booleans, errors on unknown and duplicate keys and on unknown tags | `serde_yaml` is archived and `serde_yml` is unsound. `serde-saphyr` is maintained, pure Rust and panic-free, and limits the work of a document. |
| D42 | The core starts no program except the startup command; `choractl run` starts programs through `systemd-run --user` with an activation token; `systemctl --user restart` restarts a plugin; journald keeps the output | systemd already supervises processes. The activation token links a program to its window. |
| D43 | Nodes fixed to the screen live under one overlay root per output, `screen:NAME`: in the logical coordinates of the output, without its camera, above the scene and below the top and overlay layers of layer shell | A HUD stays in place while a camera moves. A root per output answers on which output a node shows. Its nodes work as all others: input, effects, ownership. |
| D44 | `output.configure` sets scale and transform of an output through the API, on the same path as `wlr-output-management` | One path keeps both ways in step. |
| D45 | Grabs need Ctrl, Alt or Super, except for a client with the capability `input.grab-any`; `input.ungrab` ends a grab; a grab tells the release of its key; there is no exclusive keyboard grab | Typing works unless the user gives a capability, for example for push-to-talk. Games and 3D clients get every key through Wayland. |

## Process

| # | Decision | Reason |
|---|----------|--------|
| D46 | Each milestone has a gate: CI jobs and checks on the reference hardware with a log as evidence, tracked in a gate issue; the next milestone of the core starts when the gate holds | Breadth without real use hides faults. A gate makes "done" checkable. |
| D47 | The git flow of the project conventions: `main` holds `chore: init` and merge commits; work happens on `dev`, and each contributor of a shared repository has `dev-NAME`; a release is a merge into `main` and an annotated tag on the merge commit; the tag starts the release, which needs green CI of the same commit; commitizen writes the changelog; a review loop runs until a round finds nothing that matters | One flow in every repository. A release never ships a red commit. |
| D48 | The core is a library and a binary; IPC, transactions, focus and grants have unit tests with a fake clock; a pixman backend runs pixel tests in process; each bug fix comes with a test that fails without the fix | Pixel tests in a headless sway cannot reach the internals. Review finds faults that tests should catch. |
| D49 | Packages through COPR with Packit, with a subpackage per plugin, and PKGBUILDs in the AUR | The image build and updates need a package repository. |
| D50 | The session starts through uwsm; a flavor is a systemd target | uwsm handles the environment and the graphical session target. |
| D51 | The daily driver of 1.0 is `chora-tiling` with waybar, fuzzel, mako, swaylock, swayidle and kanshi; panel and launcher plugins of chora grow in parallel in their own repositories | Existing tools cover the shell. Own plugins are no condition for 1.0. |
| D52 | The milestones run 0.6 to 1.0 as in the [Roadmap](roadmap.md) | A stable API needs real use and a testable core first. |
| D53 | The voice pipeline lives in `chora-voice` and runs in parallel to the core; the core supplies push-to-talk grabs, text injection, the read model and an agent indicator | The pipeline is a product of its own. It needs only the API of the core. |
| D54 | Plugins use the JSON-RPC API with frame events and fd passing; a private Wayland protocol for plugins comes only if a measurement at 120 Hz in milestone 0.9 shows the need | One API to maintain. The measurement decides. |

## Rejected proposals

| Proposal | Reason |
|----------|--------|
| Versioned docs with mike are too much | The project conventions use mike. |
| Hooks without matching files | The project conventions use this hook set in every repository. |
| An exclusive keyboard grab | Typing must work. Clients that need every key get it through Wayland (D45). |
| A 3D node with depth buffer in the core | It makes the core an engine. 3D scenes belong in clients (D19). |
| A rewrite from scratch | The scene model, the API and the safety layer hold. The milestones change the rest step by step (D52). |
