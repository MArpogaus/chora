# chora

*chora is the space where forms appear.*

chora is a Wayland compositor without policy. It is a programmable scene, not
a desktop.

chora has no window manager, no layout, no keybindings and no look. The core
renders a small scene of primitives and offers one API. Plugins add all other
behavior. A plugin can supply a stacking desktop, a tiling workflow, or a view
that an agent arranges live by voice. Only a small safety layer is built in.

## Status

Early development. chora runs nested inside another Wayland session, and a
reference plugin tiles the windows. A login session on real hardware exists,
but nobody tested it on hardware yet. An agent sees and changes the screen
through `choractl mcp`, after a grant of the user. The
[roadmap](https://marpogaus.github.io/void/roadmap/) shows the next steps.
APIs change without notice before version 1.0.

## Install

Each [release](https://github.com/MArpogaus/void/releases) has an RPM for
Fedora 44, with SHA256 sums:

```bash
sha256sum -c SHA256SUMS
sudo dnf install ./chora-*.rpm
```

## Try it

Start chora with the tiling plugin inside a Wayland session:

```bash
chora -- chora-tiling
```

A window opens. The plugin uses these bindings:

| Binding | Action |
|---------|--------|
| Super+Return | start a terminal (foot) |
| Super+d | start a launcher (fuzzel) |
| Super+q | close the focused window |
| Super+j, Super+k | focus the next or previous window |

GNOME keeps Super for itself. There, start `chora -- chora-tiling --mod alt`
to use Alt instead. `chora-tiling --help` lists all options.

The safety layer has two fixed bindings: Ctrl+Alt+Escape resets the scene,
and Ctrl+Alt+BackSpace quits chora.

To let an agent drive chora, grant it screenshots and input, and add
`choractl --name agent mcp` as an MCP server of the agent:

```bash
choractl grant agent screen.capture input.inject
```

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
flavor. The hardware session is not tested yet.

Programs inside chora find its IPC socket in `CHORA_SOCKET`. For example, a
terminal in chora shows the scene with `choractl query`, and
`choractl events` prints the events.

## Documentation

The documentation lives in
[`docs/`](https://github.com/MArpogaus/void/tree/main/docs) and on
<https://marpogaus.github.io/void/>.

## Contributing

See [CONTRIBUTING.md](https://github.com/MArpogaus/void/blob/main/CONTRIBUTING.md).

## License

GPL-3.0-or-later. See
[LICENSE](https://github.com/MArpogaus/void/blob/main/LICENSE).

## Use of LLM coding tools

This project uses LLM coding tools for design, code and documentation.
