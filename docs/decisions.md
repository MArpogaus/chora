# Decisions

The decision log. Nothing here is final. Every entry can come back for review.

## Principles and scope

| # | Decision | Reason |
|---|----------|--------|
| D1 | Rust and Smithay for the core | Smithay is a library without policy, proven in niri and cosmic-comp. Rust is safe when agents change the system live. |
| D2 | Everything is a plugin | Maximum flexibility and a small core. |
| D3 | An experimentation platform first, then a stable, feature-complete core, then testing | A small core gives stability. |
| D4 | Reuse before writing; check alternatives for every decision | Little code to maintain. |
| D5 | One vocabulary, defined in the [Glossary](glossary.md) | One language for people and agents. |
| D6 | Inventory and agent memory form a separate layer, not part of the core | The platform stays neutral. |
| D7 | The name is chora, always in lowercase | See [Vision](vision.md#the-name). |
| D8 | The core is GPL-3.0-or-later; code under GPL-3.0-only stays out | The core can then reuse code from Pinnacle and niri. |
| D9 | English for code and documentation | The documentation is public. |

## Platform

| # | Decision | Reason |
|---|----------|--------|
| D10 | Fedora 43 and secureblue as host; a derived image offers chora as an additional session | The system and the image tooling of the author. |
| D11 | AMD and Intel GPUs first, NVIDIA later | The hardware of the author. Mesa drivers need no special paths. |
| D12 | Services run as Quadlet units with Podman or in distrobox | Isolation. systemd handles start, restart and order. |
| D13 | The coordinator is the user first, later an agent with special permissions | A gradual path to control by agents. |
| D14 | Releases are RPM packages for Fedora 43, attached with SHA256 sums to a GitHub release; a `v*` tag starts the release; versions below 1.0 are pre-releases | `dnf5 install <url>` installs them by hand and in the image build. |

## Core

| # | Decision | Reason |
|---|----------|--------|
| D15 | Build on Smithay; copy from Pinnacle, niri and anvil; no fork | A fork of Pinnacle keeps less than half of its code. See [Alternatives](alternatives.md#pinnacle-code-review). |
| D16 | Pin a Smithay git revision | No Smithay release since 0.7.0. All major users track git. |
| D17 | GLES renderer and GLSL shaders through Smithay's renderer traits | Proven in niri. No Smithay compositor uses wgpu. |
| D18 | Four node kinds: group, surface, rect, shader | Text, images and widgets come from Wayland clients. A shader can read another node as a texture. |
| D19 | Transforms are 4x4 matrices; the first milestones test 2D cases only | 3D effects need no API change later. |
| D20 | One logical scene; each output shows it through a camera | Pan, zoom and the infinite canvas need no extra mechanism. |
| D21 | The core places popups, subsurfaces and layer surfaces; toplevels wait for a plugin | The protocols define these positions. Smithay implements them. |
| D22 | A transaction completes when all affected clients commit, or after a deadline | A resize needs a client round trip. |
| D23 | A hardcoded safety layer with fixed bindings: Ctrl+Alt+Backspace quits, Ctrl+Alt+Escape resets, Ctrl+Alt+F1 to F12 switch the VT | A recovery path must not depend on a file that an agent can break. |
| D24 | Shaders declare static or animated; fullscreen surfaces without effects use direct scanout | Damage tracking and battery life. |
| D25 | No undo history and no plugin state store in the core | A destroyed client cannot come back. The reset and the views in git cover recovery. The scene holds all state. |
| D26 | Xwayland through `xwayland-satellite` | Keeps the core small. |
| D27 | Output configuration through `wlr-output-management` | kanshi and wlr-randr exist. |

## Plugins and API

| # | Decision | Reason |
|---|----------|--------|
| D28 | Plugins are IPC clients; the core offers `animate`; in-process WASM plugins come only after a measurement shows the need | Agents work in seconds, not frames. One mechanism, any language, crash isolation by the OS. |
| D29 | JSON-RPC 2.0 with one message per line on a Unix socket in `$XDG_RUNTIME_DIR/chora/`; the core serves it from the calloop event loop | No extra runtime and no thread handoff. A container mounts the directory. |
| D30 | The core starts one startup command with `WAYLAND_DISPLAY` and `CHORA_SOCKET` set | The startup command starts the plugins, or a systemd target. |
| D31 | A client names itself when it connects; the owner of a node is a client name; only the owner changes a node; plugins export methods | An agent asks the window manager instead of fighting it. |
| D32 | Selectors work in queries only; a query returns all current matches | No window rules in the core. Predictable results for agents. |
| D33 | Version 1 trusts the clients of the same user; `wp-security-context-v1` hides privileged protocols from sandboxed clients; the IPC methods `input.inject` and `screen.capture` need a grant of the user | The grant protects against mistakes of agents. It is no barrier against hostile code of the same user. |
| D34 | MCP outside the core: `choractl mcp` over stdio, with tools for whole tasks | The core stays free of MCP. |
| D35 | The core reads no configuration; `choractl` loads YAML views; a flavor is a directory with a systemd target; the configuration holds no secrets | Policy and service management do not belong in the core. |
| D36 | Three crates to start: `chora-scene`, `chora`, `choractl`; reference plugins in `plugins/` | A crate comes when a second consumer needs it. |

## Tools around the core

| # | Decision | Reason |
|---|----------|--------|
| D37 | Screen sharing through `xdg-desktop-portal-wlr` first | No code. A portal backend of its own is in the [Idea backlog](ideas.md). |
| D38 | HTML content from agents runs in a browser in app mode, as a service | No code. chora places toplevels anywhere. |

## Rejected review findings

| Finding | Reason |
|---------|--------|
| Versioned docs with mike are too much | The project conventions use mike. |
| Hooks without matching files | The project conventions use this hook set in every repository. |
| Cargo hooks before the first crate | Requested from the start. They stay inactive until the first Rust file. |

## Open questions

1. **Grants.** How does the user grant `input.inject` and `screen.capture`?
   A `choractl grant` command is the simplest option, because the core draws
   no text. Decide before M3.
2. **YAML crate.** Which maintained crate replaces `serde_yaml`? Decide in M3.
3. **Repository name.** The GitHub repository still has the name `void`.
