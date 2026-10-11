# Dependencies

The parts that Void plans to reuse. Each entry is a candidate until
[Alternatives](alternatives.md) and [Decisions](decisions.md) confirm it.

## Core

| Dependency | Purpose | Notes |
|------------|---------|-------|
| [Smithay](https://github.com/Smithay/smithay) | Wayland protocols, backends, GLES renderer, input | a library without policy; used by niri, cosmic-comp and Pinnacle; the latest release is 0.7.0 (June 2025), so Void pins a git revision |
| libinput, libseat, udev, GBM, Mesa | hardware access | through Smithay |
| calloop | event loop | Smithay is built on it; the IPC socket shares it |
| serde, schemars | API types, generated JSON Schemas | |
| a maintained YAML crate | views and flavors | `serde_yaml` is unmaintained |
| [rmcp](https://github.com/modelcontextprotocol/rust-sdk) | `voidctl mcp` | the official Rust MCP SDK, version 3.x |
| [wasmtime](https://wasmtime.dev/) | in-process plugins, if R1 keeps them | Component Model with typed WIT interfaces |

## Wayland protocols

The core speaks these protocols, so existing tools work unchanged:

- **Base:** `xdg-shell`, `xdg-decoration`, `viewporter`, `fractional-scale`,
  `linux-dmabuf`, `presentation-time`, `xdg-output`, `cursor-shape`, data
  device, `primary-selection`.
- **Shells and widgets:** `wlr-layer-shell`, `ext-foreign-toplevel-list`,
  `wlr-foreign-toplevel-management`, `xdg-activation`.
- **Desktop integration:** `ext-session-lock`, `ext-idle-notify`,
  `idle-inhibit`, `text-input`, `input-method`, `pointer-constraints`,
  `relative-pointer`, `wlr-output-management`.
- **Capture:** `ext-image-copy-capture` and `wlr-screencopy`, and a portal for
  screen sharing.
- **Isolation:** `wp-security-context-v1`, to hide privileged protocols from
  sandboxed clients.
- **Xwayland:** through `xwayland-satellite`, a separate process outside the
  core.

## Outside the core

| Tool | Role |
|------|------|
| Quickshell, waybar, eww, AGS/Astal | bars, panels, widgets |
| fuzzel, wofi | launchers |
| a webview host on `gtk4-layer-shell` and WebKitGTK, or Tauri | HTML from agents as surfaces |
| whisper.cpp or faster-whisper, piper | speech in and out |
| Podman with Quadlet, distrobox, systemd user units | services and flavors |
| kanshi, wlr-randr | output configuration |
