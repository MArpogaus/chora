# Dependencies

What Void plans to reuse, following the principle *reuse before writing*.
Each entry is a candidate until the review in [Alternatives](alternatives.md)
confirms it.

## Core

| Dependency | For | Notes |
|------------|-----|-------|
| [Smithay](https://github.com/Smithay/smithay) | Wayland protocols, backends, GLES renderer, input | a library, not a framework: no policy, which fits Void; used by niri, cosmic-comp, Pinnacle; pin a git revision (no release since 0.7.0) |
| libinput, libseat, udev, GBM, Mesa | hardware access | via Smithay |
| [wasmtime](https://wasmtime.dev/) (Component Model) | sandboxed, hot-reloadable core plugins with typed WIT interfaces | proposed, see [Decisions](decisions.md) P1 |
| [Extism](https://extism.org/) | simple plugins in many languages | candidate for the quick-snippet tier |
| serde, serde_yaml (or a maintained fork), schemars | config, API types, JSON Schemas | `serde_yaml` is unmaintained; pick a successor |
| tokio or calloop | event loop | Smithay is built on calloop; IPC can share it |
| [rmcp](https://github.com/modelcontextprotocol/rust-sdk) | the MCP bridge | official Rust MCP SDK |

## Wayland protocols the core must speak

So that existing tools work unchanged:

- core: `xdg-shell`, `xdg-decoration`, `viewporter`, `fractional-scale`,
  `linux-dmabuf`, `presentation-time`, `xdg-output`, `cursor-shape`,
  data device / `primary-selection`
- shells and widgets: `wlr-layer-shell`, `ext-foreign-toplevel-list`,
  `wlr-foreign-toplevel-management`, `xdg-activation`
- desktop integration: `ext-session-lock`, `ext-idle-notify`, `idle-inhibit`,
  `text-input` / `input-method`, `pointer-constraints`, `relative-pointer`
- capture: `ext-image-copy-capture` / `wlr-screencopy`, plus
  xdg-desktop-portal (wlr or a Void backend) for screen sharing
- Xwayland via `xwayland-satellite`, outside the core

## Outside the core (examples)

| Tool | Role |
|------|------|
| Quickshell, eww, waybar, AGS/Astal | bars, panels, widgets as Wayland clients |
| fuzzel, wofi | launchers |
| a webview host (WebKitGTK or a CEF/Servo-based layer-shell client) | agent-generated HTML content as surfaces |
| whisper.cpp / faster-whisper, piper | speech in / out services |
| Podman (Quadlet), distrobox | running services |
| secureblue + image-builder-action | host image |
