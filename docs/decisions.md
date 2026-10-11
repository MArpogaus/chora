# Decisions and open questions

The decision log. Nothing here is final. Every entry can come back for review.

- **Decided:** agreed in discussion.
- **Accepted after review:** corrections from the first review. They follow
  from the decided principles or from Wayland facts.
- **Proposed:** recommendations from the review. They wait for a decision.

## Decided

| # | Decision | Reason |
|---|----------|--------|
| D1 | Rust and Smithay for the core | Smithay is a library without policy, proven in niri and cosmic-comp. Rust is safe when agents change the system live. |
| D2 | Everything is a plugin | Maximum flexibility and a small core. |
| D3 | An experimentation platform first, then a stable, feature-complete core, then testing | A small core gives stability. |
| D4 | Fedora and secureblue as host; a derived image offers Void as an additional session | The system and the image tooling of the author. |
| D5 | Services in containers (Podman, distrobox) | Isolation, easy to add and remove. |
| D6 | The coordinator is the user first, later an agent with special permissions | A gradual path to control by agents. |
| D7 | Two access levels: the IPC socket, and in-process plugins for work in every frame | Low latency where it matters, robustness everywhere else. R1 challenges the second level. |
| D8 | A scene of five primitives; text and widgets come from existing Wayland clients | No text stack in the core. |
| D9 | YAML for views and flavors, with strict schema validation | Readability. The schema catches the YAML traps. R2 makes a flavor a directory with a systemd target. |
| D10 | One vocabulary, defined in the [Glossary](glossary.md) | One language for people and agents. |
| D11 | Inventory and agent memory form a separate layer, not part of the core | The platform stays neutral. |
| D12 | Reuse before writing; check alternatives for every decision | Little code to maintain. |

## Proposed before the review

| # | Proposal | Reason |
|---|----------|--------|
| P2 | MCP outside the core | Keeps the core small. Now `voidctl mcp` over stdio. |
| P3 | GLES renderer and GLSL shaders | Proven in niri. No Smithay compositor uses wgpu. |
| P4 | Build on Smithay and copy from niri and Pinnacle | Full control of the scene and the plugin model. |
| P8 | Xwayland through `xwayland-satellite` | Keeps the core small. |
| P10 | English for code and documentation | The docs become public documentation. |

P1 (plugin runtime), P5 (safety), P6 (transactions), P7 (capabilities) and P9
(Smithay pin) moved into the lists below.

## Accepted after review

| # | Change | Reason |
|---|--------|--------|
| A1 | A transaction completes when all affected clients commit, or when a deadline passes | A resize needs a client round trip. Sway, niri and River do the same. |
| A2 | The core places popups, subsurfaces and layer surfaces; only toplevels wait for a plugin | The protocols define these positions. Smithay implements them. |
| A3 | A `configure` command for size, toplevel states and decoration mode; events with size limits and activation requests | A window manager needs them. |
| A4 | Hardcoded safety layer: fixed bindings that ignore injected input, a reset to a grid, session lock above all plugins, a frame watchdog | A recovery path must not depend on a file that an agent can break. |
| A5 | No undo history in the core | A destroyed client cannot come back. The reset and the views in git cover recovery. |
| A6 | A plugin owns its nodes; the core removes them when the plugin stops | No orphaned nodes. |
| A7 | No `z` property, no query inside transactions, no plugin state store, no `shader.compile`, no `view.diff` in the core | Each duplicated another mechanism. |
| A8 | Smithay's renderer traits, without an extra abstraction layer | Smithay already has the interface. |
| A9 | Three crates to start: `void-scene`, `void`, `voidctl` | A crate comes when a second consumer needs it. |
| A10 | Output configuration through `wlr-output-management` | kanshi and wlr-randr exist. |
| A11 | The socket path in `VOID_SOCKET`, no token store in the config | The config lives in git. |
| A12 | Pin a Smithay git revision | No release since 0.7.0. |
| A13 | Shaders declare static or animated; fullscreen surfaces without effects use direct scanout | Damage tracking and battery life. |

## Proposed after review

| # | Proposal | Reason |
|---|----------|--------|
| R1 | Plugins are IPC clients first, like the window managers of River; the core offers `animate`; wasmtime only when a measurement demands it | Agents work in seconds, not frames. One runtime less. Typed WIT only helps without JSON. |
| R2 | Views, flavors and services leave the core: `voidctl` loads views, a flavor is a systemd target, a service is a Quadlet unit; the core reads no config | Policy and service management do not belong in the core. systemd exists. |
| R3 | Write ownership: one writer at a time changes a node; plugins export methods, for example `tiling.set_layout` | A6 settles who removes a node. R3 settles who changes it. Otherwise an agent and the window manager fight over the same nodes. |
| R4 | Version 1 trusts the same user, as sway does; `wp-security-context-v1` hides privileged protocols from sandboxed clients; containers get restricted sockets; input injection and screen capture need approval | Simple first, safe for containers. |
| R5 | One logical scene; each output is a viewport onto it | Coordinates, scale and multiple outputs depend on this. It also gives the infinite canvas. |
| R6 | A 3D matrix transform instead of a 2D transform | Cube, wobbly windows and 3D surfaces need it. |
| R7 | Defer the image node, or support PNG only | Image decoding in the core costs code. |
| R8 | Define selector behavior for several matches, no match and changed titles | Agents depend on predictable results. |
| R9 | A short, timeboxed review of the Pinnacle code before P4 is final | A fork could save work. |
| R10 | First milestone: nested winit core, scene, transactions, IPC, one tiling plugin as IPC client; no WASM, MCP, shaders or flavors | The smallest proof of the concept. |

## Rejected review findings

| Finding | Reason |
|---------|--------|
| Versioned docs with mike are too much | The project conventions use mike. |
| Hooks without matching files, and `commitizen-branch` next to `commitizen` | The project conventions use this hook set in every repository. |
| Cargo hooks before the first crate | Requested from the start. They stay inactive until the first Rust file. |

## Open questions

1. **Name.** "Void" collides with Void Linux. Does that matter for package
   names and search?
2. **License.** MIT or Apache-2.0 like Smithay, or GPL like niri?
3. **Webview host.** WebKitGTK, Tauri or Servo? The agentic scenario depends
   on it.
4. **Hardware.** Which GPU vendors come first? NVIDIA?
5. **Screen sharing.** A portal backend of its own, or
   `xdg-desktop-portal-wlr`?
