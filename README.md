# chora

*chora is the space where forms appear.*

chora is a Wayland compositor without policy. It is a programmable scene, not
a desktop.

chora has no window manager, no layout, no keybindings and no look. The core
renders a small scene of primitives and offers one API. Plugins add all other
behavior. A plugin can supply a stacking desktop, a tiling workflow, or a view
that an agent arranges live by voice. Only a small safety layer is built in.

## Status

Pre-release. chora runs nested inside another Wayland session and as a
login session, and a reference plugin tiles the windows. An agent sees and
changes the screen through `choractl mcp`, after a grant of the user.
chora is no daily driver yet. The
[roadmap](https://marpogaus.github.io/chora/roadmap/) leads to 1.0: a daily
driver with a stable API. The API changes without notice before 1.0.

## Install

Each [release](https://github.com/MArpogaus/chora/releases) has an RPM for
Fedora 44 and a package for Arch Linux, with SHA256 sums:

```bash
sha256sum -c --ignore-missing SHA256SUMS
sudo dnf install ./chora-*.rpm          # Fedora
sudo pacman -U ./chora-*.pkg.tar.zst    # Arch Linux
```

## Try it

Start chora with the tiling plugin inside a Wayland session:

```bash
chora -- chora-tiling
```

A window opens. The plugin follows the default keymap of the awesome
window manager:

| Binding | Action |
|---------|--------|
| Super+Return | start a terminal (foot) |
| Super+r | start a launcher (fuzzel) |
| Super+Shift+c | close the focused window |
| Super+j, Super+k | focus the next or previous window |
| Super+1 to Super+9 | show a tag |
| Super+Shift+1 to Super+Shift+9 | move the focused window to a tag |
| Super+space | switch to the next layout |

`chora-tiling --print-config` prints all options and bindings. Copy the
output to `~/.config/chora/chora-tiling.yaml` and change it there. GNOME keeps
Super for itself; there, set `modifier: Alt` in that file.

Nested, chora takes the keymap of the host session. `XKB_DEFAULT_LAYOUT`
sets another layout, for example `XKB_DEFAULT_LAYOUT=de chora --
chora-tiling`. The log names the layouts at the start.

The safety layer has fixed bindings: Ctrl+Alt+Escape resets the scene,
Ctrl+Alt+BackSpace quits chora, and Ctrl+Alt+F1 to F12 switch the VT.

To let an agent drive chora, grant it screenshots and input, and add
`choractl --name agent mcp` as an MCP server of the agent:

```bash
choractl grant agent screen.capture input.inject
```

The agent can then put its own shaders and images, reserve a panel at the
edge of the output, wait for clicks and keys, and move the camera. With
`choractl run` it starts programs outside the core, and `choractl
supervise` with `choractl restart` keeps a plugin running (D42).

To try the XP flavor, a stacking desktop in the style of Windows XP, start
`chora -- chora-xp`. From a source tree, point it at the theme:
`target/debug/chora -- target/debug/chora-xp --theme flavors/xp/theme`.
Windows wobble while you drag them and minimize through a magic lamp.
Ctrl+Alt+Left and Ctrl+Alt+Right, or the desktop buttons 1 to 4 in the
taskbar, turn a cube of four desktops. Inside GNOME, which keeps
Ctrl+Alt+arrows and Super for itself, use the desktop buttons and the
start menu.

To log in to chora on real hardware, choose the session "chora" on the
login screen. It starts the tiling flavor with waybar and X11 support
through xwayland-satellite. The session "chora (Windows XP)" starts the XP
flavor.

Programs inside chora find its IPC socket in `CHORA_SOCKET`. For example, a
terminal in chora shows the scene with `choractl query`, and
`choractl events` prints the events.

Under [`examples/`](https://github.com/MArpogaus/chora/tree/main/examples)
live programs that drive the scene through that socket: the loader
prototype (`examples/loader`) flies a construct of views, HUDs and
shaders through the camera, and `examples/showcase` collects six small
apps: snake, pong, two text renderers, a shader card UI and a slide
deck.

## Documentation

The documentation lives in
[`docs/`](https://github.com/MArpogaus/chora/tree/main/docs) and on
<https://marpogaus.github.io/chora/>.

## Contributing

See [CONTRIBUTING.md](https://github.com/MArpogaus/chora/blob/main/CONTRIBUTING.md).

## License

GPL-3.0-or-later. See
[LICENSE](https://github.com/MArpogaus/chora/blob/main/LICENSE).

## Use of LLM coding tools

This project uses LLM coding tools for design, code and documentation.
