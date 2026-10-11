# Alternatives

Following the principle *be lazy*, every building block gets checked against
what already exists. This page records the first review (October 2026).
Versions and status were checked at that time; entries marked *(unverified)*
need a second look.

## Compositors: adopt, fork, or build?

| Project | Stack | Extension model | How close to Void | What is missing |
|---------|-------|-----------------|-------------------|-----------------|
| [River 0.4](https://isaacfreund.com/blog/river-window-management/) | Zig, wlroots | the WM is a **separate process** speaking the `river-window-management-v1` protocol; atomic, frame-perfect batches | closest to "the WM is a plugin" | no scene graph, no per-frame plugins; custom shaders/effects explicitly far off; early, breaking changes expected |
| [Pinnacle](https://github.com/pinnacle-comp/pinnacle) | Rust, Smithay | out-of-process **gRPC API** with Lua and Rust clients; Snowcap widget layer | best Rust fork base: policy already lives outside the compositor | no per-frame plugins, no scene primitives or shader nodes, no MCP; small team |
| [niri](https://github.com/YaLTeR/niri) | Rust, Smithay | KDL config, `niri msg` IPC; no plugins | excellent Smithay reference (rendering, shaders, blur, animations) | scrolling-tiling policy hard-wired |
| [cosmic-comp](https://github.com/pop-os/cosmic-comp) | Rust, Smithay | none (COSMIC shell) | production-grade Smithay reference | tied to COSMIC |
| [Jay](https://github.com/mahkoh/jay) | Rust, own stack | config as a Rust shared library *(unverified)* | Vulkan renderer as a reference | not Smithay; GPLv3 |
| [Wayfire](https://github.com/WayfireWM/wayfire) | C++, wlroots | in-process C++ plugins + IPC | plugin philosophy and 3D effects close | unsafe C++ ABI, slow release cadence |
| [Hyprland](https://hypr.land/) | C++, Aquamarine | C++ plugins (unstable ABI), Lua config since 0.55 | the effects we want | opinionated; plugin ABI breaks every release |
| Mir / MirAL | C++ | libraries to build shells | mature | C++, MirAL's policy model |
| Louvre | C++ | library with overridable defaults | nice scene API | not Rust |
| scenefx | C, wlroots | drop-in wlroots scene with blur, shadows, rounded corners | "scene graph + effects" | C/wlroots only |
| CwC, SomeWM, Strata | C/Lua, Rust+Lua | scriptable configs | scriptable | none is policy-free + WASM + scene graph |

**Agent tooling today** exists only as external MCP servers on top of
existing compositors (e.g. kwin-mcp, hypruse for Hyprland). No compositor
with WASM plugins or built-in MCP was found. Void's combination — policy-free
core, scene of primitives with shaders, sandboxed per-frame plugins, one API
for plugins/IPC/MCP — appears to be new.

### Options

1. **Build on Smithay, borrow heavily** (from anvil/smallvil examples, niri,
   cosmic-comp, Pinnacle). Full control over scene and plugin model.
   *Recommended.*
2. **Fork Pinnacle.** Gets a working Smithay compositor with an external API
   on day one; we would rip out its policy, add the scene graph, shaders and
   in-process plugins. Worth a closer look at its code before deciding against
   it.
3. **River 0.4 + a Rust WM client.** Zero compositor code, but gives up
   shaders, effects and the per-frame scene — the "game engine" half of the
   idea. Good fallback for a quick *tiling-only* prototype.
4. **External MCP on an existing compositor** (like kwin-mcp). Fastest way to
   try agentic workflows, but no canvas, no effects: useful as an experiment,
   not as Void.

## Plugin runtimes

| Runtime | Strengths | Weaknesses |
|---------|-----------|------------|
| [Extism](https://extism.org/) (wasmtime) | 13+ guest languages, simple, sandboxed | bytes in / bytes out: every call serializes and copies; fine for small messages, poor for large scene diffs |
| wasmtime + **Component Model** (WIT) | typed interfaces, resource handles to scene nodes, fuel/epoch limits, fast re-instantiation | fewer mature guest languages (Rust first-class, JS/Python heavier) |
| wasmer | viable | less Component Model momentum |
| mlua (LuaJIT / Luau) | lowest friction, trivial hot reload, Luau sandbox | no memory isolation from the host |
| Rhai | pure Rust, easy | tree-walking interpreter, too slow per frame |
| Rune | Rust-native | pre-1.0 |
| deno_core (V8) | JS/TS, huge ecosystem | heavy, GC pauses, API churn |
| abi_stable dylibs | native speed | stale, unsafe unloading, no sandbox |

**Finding:** for the per-frame hot path, wasmtime with the Component Model
(WIT interfaces) beats Extism; Extism shines for "any language, simple
messages". Both run on wasmtime, so a **two-tier** approach is possible:
Component Model for core plugins (WM, layout, animation), Extism-style or
script plugins for quick user/agent snippets. See [Decisions](decisions.md).

## Renderer

- Smithay's GLES renderer is what niri, cosmic-comp and Pinnacle use; custom
  GLSL shaders via texture/pixel shader elements are proven (niri's blur).
- No Smithay compositor renders through **wgpu**; Smithay maintainers
  consider it too abstract for compositor needs. No merged Smithay Vulkan
  renderer either.
- **Recommendation:** GLES + GLSL now, keep the renderer behind a narrow
  interface.

## MCP

The official Rust SDK [rmcp](https://github.com/modelcontextprotocol/rust-sdk)
(3.x) supports stdio and Streamable HTTP. Plan: stdio for local agents,
Streamable HTTP for agents in containers, both bridged to the Unix-socket IPC.

## Widgets and shells

[Quickshell](https://quickshell.org/) (Qt6/QML, declarative, live reload) is
the strongest choice and closest to "declarative views"; waybar is mature;
eww and AGS/Astal need an activity check. For agent-generated content, a
webview on gtk4-layer-shell + WebKitGTK, or a Tauri window mapped as a
layer-shell surface. All of them need `wlr-layer-shell`.

## Smithay itself

Latest crate release 0.7.0 (June 2025); all major users track git master.
Mature in production, API unstable between releases → **pin a git revision**.
