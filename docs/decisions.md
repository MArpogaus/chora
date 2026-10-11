# Decisions and open questions

The decision log. *Decided* means agreed in discussion; it can still be
reopened by the critical review. *Proposed* means a recommendation waiting
for confirmation.

## Decided

| # | Decision | Rationale |
|---|----------|-----------|
| D1 | **Rust + Smithay** for the core | Smithay is a policy-free library, proven in niri and cosmic-comp; Rust's safety matters when agents change the system live |
| D2 | **Everything is a plugin** — the guiding principle | maximal flexibility; the core stays lean |
| D3 | **Experimentation platform first**, reaching a stable, feature-complete core quickly, then testing | stability through a small core, not through features |
| D4 | **Fedora / secureblue host**: a derived image offers Void as an additional session | the author's daily system and existing image tooling |
| D5 | **Services in containers** (Podman, distrobox) | isolation, easy add/remove |
| D6 | **Coordinator** is the user first, later an agent with special permissions that can install and write plugins and services | gradual path to agentic control |
| D7 | **Two access levels**: an IPC socket (agents, services, live config) and in-process plugins for per-frame work | latency where it matters, robustness everywhere else |
| D8 | **Scene of few primitives** (Group, Surface, Rect, Image, Shader); text and widgets come from existing Wayland clients | lazy: no text stack in the core |
| D9 | **YAML** for views, flavors and config, with strict schema validation | readability; schema catches YAML's pitfalls |
| D10 | **Vocabulary**: node, view, plugin, service, flavor, coordinator, inventory (see [Glossary](glossary.md)) | one shared language for humans and agents |
| D11 | **Inventory and agent memory** are a separate, opinionated layer, not part of the core | keep the platform neutral |
| D12 | **Reuse before writing**, with a critical alternatives review for every decision | be as lazy as possible, contribute upstream |

## Proposed (needs confirmation)

| # | Proposal | Why |
|---|----------|-----|
| P1 | **Plugin runtime:** wasmtime with the **Component Model** (WIT) for core plugins; Extism or a script runtime for quick snippets | Extism serializes every call; WIT gives typed, cheap calls for the hot path (see [Alternatives](alternatives.md#plugin-runtimes)) |
| P2 | **MCP as a separate bridge** (`void-mcp`) over the IPC socket, built on rmcp | keeps the core lean |
| P3 | **GLES renderer + GLSL shaders**, renderer behind a narrow interface | proven in niri; wgpu has no Smithay precedent |
| P4 | **Build on Smithay, borrow from niri/Pinnacle**; look at Pinnacle's code before ruling out a fork | full control over scene and plugin model |
| P5 | **Hardcoded safety layer** (VT switch, quit, emergency reset to a safe view) as the only built-in behaviour | agents will write broken code live |
| P6 | **Transactions applied at frame boundaries**, with rollback and undo history | no half-applied states, easy recovery |
| P7 | **Capabilities via tokens** for IPC clients and plugin manifests | input injection and screen capture are sensitive |
| P8 | **Xwayland via xwayland-satellite**, outside the core | lean core |
| P9 | **Pin a Smithay git revision** | no Smithay release since 0.7.0 |
| P10 | **English** for code and documentation | the docs become official, public documentation |

## Open questions

1. **Name.** "Void" collides with Void Linux; does that matter (package
   names, searchability)?
2. **License.** MIT/Apache-2.0 (permissive, like Smithay) or GPL (like niri)?
3. **Plugin boundary format.** If Extism stays: JSON, MessagePack or
   FlatBuffers for commands and events?
4. **Scripting tier.** Is a script language (Lua/Luau) worth adding for agents
   that send small snippets, or is "agent writes a WASM plugin" enough?
5. **Webview service.** Which stack (WebKitGTK, Tauri, Servo)? It is central
   for the agentic scenario.
6. **Multi-output and coordinates.** One scene spanning all outputs, or one
   root group per output?
7. **Surface placement rules.** Does a detached new surface stay invisible
   until a plugin places it, or is there a safe fallback?
8. **Hardware scope.** GPU vendors to support first (NVIDIA?).
9. **Screen sharing.** Implement a portal backend or reuse
   xdg-desktop-portal-wlr?
10. **First milestone.** What is the smallest thing that proves the concept —
    e.g. nested winit compositor + scene + IPC + one tiling plugin?
