# Dependencies

The parts that chora reuses. [Alternatives](alternatives.md) records the
check of each choice. [Decisions](decisions.md) records the result.

## Core

| Dependency | Purpose | Notes |
|------------|---------|-------|
| [Smithay](https://github.com/Smithay/smithay) | Wayland protocols, backends, GLES renderer, input | a library without policy; used by niri, cosmic-comp and Pinnacle; the latest release is 0.7.0 (June 2025), so chora pins a git revision |
| libinput, libseat, udev, GBM, Mesa | hardware access | through Smithay |
| calloop | event loop | Smithay uses it; the IPC socket runs in the same loop |
| [imbl](https://github.com/jneem/imbl) | the node map of the scene | a persistent map, so each transaction copies the scene in O(1) |
| serde, serde_json, thiserror | API types, JSON-RPC, error types | |
| tracing, tracing-subscriber | logs | Smithay logs through tracing too |

## choractl

| Dependency | Purpose | Notes |
|------------|---------|-------|
| clap | command line arguments | also for the plugins |
| serde-saphyr | views | strict YAML (D41) |
| schemars | the JSON schemas of the MCP tools, from the types of `chora-scene` | |

`choractl mcp` implements the stdio transport of MCP itself. Milestone 0.8
replaces it with [rmcp](https://github.com/modelcontextprotocol/rust-sdk),
the official Rust MCP SDK (D34).

## Plugins

| Dependency | Purpose | Notes |
|------------|---------|-------|
| [resvg](https://github.com/linebender/resvg) | `chora-xp` rasterizes the SVG files of its theme into PNG files for image nodes | Apache-2.0 OR MIT; brings usvg (Apache-2.0 OR MIT) and tiny-skia (BSD-3-Clause) |
| fontdb, harfrust, skrifa | text of `chora-xp` through usvg: the bundled Tahoma and the system fonts | fontdb and harfrust MIT; skrifa and read-fonts MIT OR Apache-2.0 |
| zune-jpeg | `chora-xp` decodes JPEG wallpapers, such as XP's Bliss, into its PNG cache | MIT OR Apache-2.0 OR Zlib; Rust 1.75; with zune-core; resvg has it only behind `raster-images`, which also brings GIF and WebP |
| chrono | the clock of the taskbar of `chora-xp`, in local time | MIT OR Apache-2.0; with iana-time-zone (MIT OR Apache-2.0) |
| [zbus](https://github.com/z-galaxy/zbus) | the notification server of `chora-xp` on the session bus, through its blocking API | MIT; 5.19, because 6.0 has no blocking API; brings zvariant (MIT) and the smol crates (Apache-2.0 OR MIT) |

The theme of the XP flavor is artwork, not code: SVG files of Exposé and of
the Windows XP pack of `kde-plasma-winxp-launcher`, two fonts, a wallpaper
and cursors. `flavors/xp/theme/README.md` lists each file with its license.
Debug builds compile the image, font and SVG crates optimized, so that
`chora-xp` starts in a second.

## Build and release

| Tool | Purpose |
|------|---------|
| [cargo-generate-rpm](https://github.com/cat-in-136/cargo-generate-rpm) | builds the RPM from the Cargo metadata, without a spec file |
| cargo-deny | license and advisory checks of the dependencies |
| commitizen | version bump, tag and changelog |

## Code to copy

The core is GPL-3.0-or-later, so it can reuse GPL code. Code under
GPL-3.0-only stays out, for example `src/protocol/drm.rs` of Pinnacle.
[CONTRIBUTING](https://github.com/MArpogaus/chora/blob/main/CONTRIBUTING.md)
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

## Outside the core

| Tool | Role |
|------|------|
| `xwayland-satellite` | Xwayland as a separate process |
| `xdg-desktop-portal-wlr` | screen sharing |
| Quickshell, waybar, eww, AGS/Astal | bars, panels, widgets |
| fuzzel, wofi | launchers |
| swaybg, swww | wallpapers |
| Chromium or Firefox in app mode | HTML content from agents |
| [Gufo](https://github.com/gufo-org/gufo) with speech, router and main models | the voice pipeline of `chora-voice` (D53) |
| Podman with Quadlet, distrobox, systemd user units | services and flavors |
| kanshi, wlr-randr | output configuration |
