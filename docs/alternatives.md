# Alternatives

Every part of Chora gets a check against existing projects. This page records
the first check from October 2026. An entry with the mark *unverified* needs
a second look.

## Compositors

| Project | Stack | Extension model | Fit for Chora | Missing |
|---------|-------|-----------------|--------------|---------|
| [River 0.4](https://isaacfreund.com/blog/river-window-management/) | Zig, wlroots | the window manager is a separate process; it speaks the stable `river-window-management-v1` protocol with atomic, frame-perfect updates | closest to "the window manager is a plugin" | no scene graph, no shaders or effects |
| [Pinnacle](https://github.com/pinnacle-comp/pinnacle) | Rust, Smithay | gRPC API with Lua and Rust clients; Snowcap widgets | best Rust base for a fork: the policy already lives outside the compositor | no scene primitives, no shader nodes, no MCP; small team |
| [niri](https://github.com/YaLTeR/niri) | Rust, Smithay | KDL config, `niri msg` IPC, no plugins | best Smithay reference for rendering, shaders, blur and animations | scrolling tiling is built in |
| [cosmic-comp](https://github.com/pop-os/cosmic-comp) | Rust, Smithay | none | production-grade Smithay reference | bound to COSMIC |
| [Jay](https://github.com/mahkoh/jay) | Rust, own stack | config as a Rust library *(unverified)* | reference for a Vulkan renderer | not Smithay; GPLv3 |
| [Wayfire](https://github.com/WayfireWM/wayfire) | C++, wlroots | in-process C++ plugins, IPC | close plugin idea, 3D effects | unsafe C++ ABI, slow releases |
| [Hyprland](https://hypr.land/) | C++, Aquamarine | C++ plugins with an unstable ABI, Lua config | the target effects | opinionated; the plugin ABI breaks with every release |
| Mir and MirAL | C++ | libraries for shells | mature | C++; the policy model of MirAL |
| Louvre | C++ | library with defaults that can be replaced | good scene API | not Rust |
| scenefx | C, wlroots | wlroots scene with blur, shadows and rounded corners | scene graph and effects | C and wlroots only |
| CwC, SomeWM, Strata | C and Lua, Rust and Lua | scripted configs | scriptable | no policy-free core with a scene graph |

Agent tools exist today only as external MCP servers on top of existing
compositors, for example kwin-mcp, and hypruse for Hyprland. The check found
no compositor with WASM plugins or built-in MCP. The combination of Chora
appears to be new: a policy-free core, a scene of primitives with shaders,
and one API for plugins, IPC and MCP.

### Options

1. **Build on Smithay and copy from others.** Sources: the anvil and smallvil
   examples, niri, cosmic-comp, Pinnacle. This option gives full control of
   the scene and the plugin model. This is the decision (D14).
2. **Fork Pinnacle.** A working Smithay compositor with an external API on
   day one. The code review below rejects this option.
3. **River 0.4 with a Rust window manager.** No compositor code, but no
   shaders, no effects and no scene. This option drops the game-engine half of
   the idea. It is a fallback for a quick tiling prototype.
4. **An MCP server on an existing compositor**, like kwin-mcp. This is the
   fastest way to try agentic workflows. It gives no canvas and no effects.

River shows a strong design for the window manager interface. Chora can copy
its semantics without its code.

### Pinnacle code review

The review measured a clone of Pinnacle 0.2.2 (last commit 2026-06-27).

- **Size:** 34,138 lines of Rust in `src/`. The workspace adds the client
  API (Rust and Lua), the Snowcap widget toolkit and a wlcs test adapter.
- **Policy in the core:** tags touch 20 files, layout modes 20 files, window
  rules 8 files. Click-to-focus is hardcoded.
- **Rendering:** a flat list of Smithay render elements on top of Smithay's
  `Space`. No scene graph and no custom shaders. `Space` appears in 23 files.
- **API:** tonic gRPC with 115 methods around tags and layout modes. No
  batching and no transactions for clients.
- **Transactions:** yes, for window positions per output. A transaction
  completes when every window commits, or after 150 ms. The design comes from
  niri.
- **License:** GPL-3.0-or-later. `src/protocol/drm.rs` is GPL-3.0-only. The
  API crates and Snowcap are MPL-2.0.
- **Activity:** 57 commits in February 2026, then 2, 3 and 8 per month.

A fork keeps about 40 to 50 percent of `src/`. It deletes or rewrites the
spine that Chora replaces: `Space`, the window state, tags and the API. Only
the generic parts survive, and these are easy to copy. [Dependencies](dependencies.md#code-to-copy)
lists them.

## Plugin runtimes

| Runtime | Strengths | Weaknesses |
|---------|-----------|------------|
| [Extism](https://extism.org/) on wasmtime | more than 13 guest languages, simple, sandboxed | bytes in, bytes out: every call serializes and copies |
| wasmtime with the Component Model | typed WIT interfaces, handles to scene nodes, fuel and epoch limits, fast new instances | fewer mature guest languages; Rust is first-class |
| wasmer | works | less momentum for the Component Model |
| mlua with LuaJIT or Luau | low friction, easy reload, Luau sandbox | no memory isolation from the host |
| Rhai | pure Rust, easy | slow tree-walking interpreter |
| Rune | native to Rust | not at 1.0 |
| deno_core (V8) | JS and TS ecosystem | heavy, GC pauses, API churn |
| abi_stable dylibs | native speed | stale, unsafe unloading, no sandbox |
| **IPC clients** (separate processes) | any language, crash isolation from the OS, no runtime in the core | a round trip per event; no code in every frame |

For code in every frame, the Component Model is better than Extism. Typed
calls only help when the boundary stays typed: JSON over WIT gives back the
cost of Extism. Chora starts with IPC clients only (D27). wasmtime comes when
a measurement demands it.

## Renderer

- niri, cosmic-comp and Pinnacle use the GLES renderer of Smithay. Custom
  GLSL shaders through texture and pixel shader elements work in practice,
  for example the blur of niri.
- No Smithay compositor renders through wgpu. The Smithay maintainers
  consider wgpu too abstract for a compositor. Smithay has no merged Vulkan
  renderer either.
- Decision: GLES and GLSL (D16).

## MCP

[rmcp](https://github.com/modelcontextprotocol/rust-sdk) 3.x supports stdio
and Streamable HTTP. `choractl mcp` uses stdio. A container gets the socket
through a bind mount, so Chora needs no network transport for now.

## Widgets and shells

[Quickshell](https://quickshell.org/) is the strongest choice: Qt6 and QML,
declarative, with live reload. waybar is mature. eww and AGS/Astal need a
check of their activity. All of these need `wlr-layer-shell`.

For HTML from agents, a browser in app mode (`chromium --app=...`) is enough
(D35). Chora places toplevels anywhere. A dedicated host on
`gtk4-layer-shell` with WebKitGTK, or a Tauri window as a layer surface, is
an option for later.
