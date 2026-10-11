# Dependencies

The parts that chora reuses. [Alternatives](alternatives.md) records the
check of each choice. [Decisions](decisions.md) records the result.

## Core

| Dependency | Purpose | Notes |
|------------|---------|-------|
| [Smithay](https://github.com/Smithay/smithay) | Wayland protocols, backends, GLES renderer, input | a library without policy; used by niri, cosmic-comp and Pinnacle; the latest release is 0.7.0 (June 2025), so chora pins a git revision |
| libinput, libseat, udev, GBM, Mesa | hardware access | through Smithay |
| calloop | event loop | Smithay uses it; the IPC socket runs in the same loop |
| serde, serde_json, thiserror | API types, JSON-RPC, error types | |

## choractl

| Dependency | Purpose | Notes |
|------------|---------|-------|
| a maintained YAML crate | views | `serde_yaml` is unmaintained |
| [rmcp](https://github.com/modelcontextprotocol/rust-sdk) | `choractl mcp` | the official Rust MCP SDK, version 3.x |

## Build and release

| Tool | Purpose |
|------|---------|
| [cargo-generate-rpm](https://github.com/cat-in-136/cargo-generate-rpm) | builds the RPM from the Cargo metadata, without a spec file |
| cargo-deny | license and advisory checks of the dependencies |
| commitizen | version bump, tag and changelog |

## Code to copy

The core is GPL-3.0-or-later, so it can reuse GPL code. Code under
GPL-3.0-only stays out, for example `src/protocol/drm.rs` of Pinnacle.
[CONTRIBUTING](https://github.com/MArpogaus/void/blob/main/CONTRIBUTING.md)
says how to mark copied code.

| Source | Parts |
|--------|-------|
| [Pinnacle](https://github.com/pinnacle-comp/pinnacle) | transactions (`util/transaction.rs`, `hook.rs`), wlr protocol implementations (`protocol/`), udev and winit backends, the wlcs test adapter |
| [niri](https://github.com/YaLTeR/niri) | custom shaders, blur, offscreen rendering, animations |
| anvil and smallvil (Smithay) | minimal compositor structure |

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
- **Capture:** `ext-image-copy-capture` and `wlr-screencopy`.
- **Isolation:** `wp-security-context-v1`.

[Roadmap](roadmap.md) assigns each protocol to a milestone.

## Outside the core

| Tool | Role |
|------|------|
| `xwayland-satellite` | Xwayland as a separate process |
| `xdg-desktop-portal-wlr` | screen sharing |
| Quickshell, waybar, eww, AGS/Astal | bars, panels, widgets |
| fuzzel, wofi | launchers |
| swaybg, swww | wallpapers |
| Chromium or Firefox in app mode | HTML content from agents |
| whisper.cpp or faster-whisper, piper | speech in and out |
| Podman with Quadlet, distrobox, systemd user units | services and flavors |
| kanshi, wlr-randr | output configuration |
