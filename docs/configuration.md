# Configuration

Views are YAML files in git, and an agent can read and write them. A strict
schema validates each file. It rejects unknown keys and catches the YAML
traps, for example `no` read as `false`.

The core reads no configuration file. It runs one startup command (D30), for
example `chora-flavor`, which starts the systemd target of a flavor:

```bash
chora -- chora-flavor tiling
```

## Locations

```text
~/.config/chora/
├── views/      # one file per view
├── flavors/    # bundles of views, plugins and services
├── plugins/    # plugin manifests
└── shaders/    # GLSL ES shaders
```

The directory is a git repository. It contains no secrets. The package
installs the reference flavors to `/usr/share/chora/`.

## Views

A view is a part of the scene and the plugins that drive it.
`choractl view load <name>` starts the plugins of the view that do not run
yet. Then it applies the nodes of the view as one transaction.

```yaml
# ~/.config/chora/views/analysis.yaml
name: analysis
plugins:
- name: tiling
  config: {layout: columns, gap: 12}
- name: keybindings
  config:
    bindings:
      Super+Return: {spawn: foot}
      Super+q: close
scene:
- id: workspace
  kind: group
  effects: [{shader: vignette.frag, uniforms: {strength: 0.3}}]
```

The tiling plugin places the surfaces in `workspace`. The view only names the
plugin and its configuration.

## Flavors

A flavor is a directory with views, plugin manifests and services, plus a
systemd user target `chora-<name>.target`. `chora-flavor <name>` looks for
the directory in `~/.config/chora/flavors/` first and then in
`/usr/share/chora/flavors/`. It imports `WAYLAND_DISPLAY`, `CHORA_SOCKET` and
the X11 `DISPLAY` of `xwayland-satellite` into the systemd user manager, so
that every unit of the flavor starts with them. It links the units of the
directory, and starts
the target. The target starts the services and the plugins. systemd handles
start, restart and order, so chora needs no service manager.

The package installs the tiling flavor:

```text
/usr/share/chora/flavors/tiling/
├── chora-tiling.target              # wants the three units below
├── chora-tiling-wm.service          # chora-tiling
└── chora-tiling-xwayland.service    # xwayland-satellite, if installed
```

The target also wants `waybar.service`, which the waybar package installs.
A copy of the directory in `~/.config/chora/flavors/tiling/` replaces it.

### The XP flavor

The XP flavor is the showroom of milestone M3 (D39): a stacking desktop in
the style of Windows XP. It is a demo, not a daily driver.

```text
/usr/share/chora/flavors/xp/
├── chora-xp.target              # wants the two units below
├── chora-xp-wm.service          # chora-xp
├── chora-xp-xwayland.service    # xwayland-satellite, if installed
└── theme/                       # SVG files, fonts, wallpaper and cursors
```

The session "chora (Windows XP)" on the login screen starts it with the XP
cursors, which the package installs to `/usr/share/icons/ModernXP`.
`theme/README.md` lists the source and the license of every file of the
theme.

`chora-xp` draws everything as image nodes. It rasterizes the SVG files of
the theme with resvg into PNG files in `$XDG_RUNTIME_DIR/chora-xp/`, and
renders each picture once:

- a frame around each window, with its title and the minimize, maximize
  and close buttons;
- a taskbar at the bottom of the work area of each output, with the start
  button, one button per window and a clock;
- the start menu with the user name, programs, places, Log Off and Turn
  Off Computer;
- the wallpaper Bløff below everything.

A click on a window raises and focuses it. A drag of a title bar moves the
window, and a double click maximizes it. A maximized window fills the work
area above the taskbar. Minimize shrinks the window into its task button. A
click on a task button shows or minimizes its window. Log Off asks every
window to close. Turn Off Computer ends `chora-xp`; it shuts nothing down.
The plugin grabs two key bindings: Alt+F4 closes the focused window, and
Super+e starts the terminal.

| Option | Default | Meaning |
|--------|---------|---------|
| `--theme DIR` | `/usr/share/chora/flavors/xp/theme`, or `CHORA_XP_THEME` | the theme directory |
| `--terminal CMD` | `foot` | the terminal of the start menu and of Super+e |
| `--launcher CMD` | `fuzzel` | All Programs and Run in the start menu |
| `--program TITLE=CMD` | | another program in the start menu; repeat for more |
| `--clock FORMAT` | `%-I:%M %p` | the clock, in the format of strftime |
| `--wallpaper FILE` | Bløff | a PNG or SVG file |

To try the flavor from a source tree, start it nested:

```bash
cargo build --workspace
target/debug/chora -- target/debug/chora-xp --theme flavors/xp/theme
```

```text
flavors/agentic/
├── chora-agentic.target
├── views/       # morning, analysis, writing, coding
├── plugins/     # canvas layout, keybindings
└── services/    # Quadlet units: speech, browser in app mode
```

## Services

A service runs as a Quadlet unit with Podman, or in distrobox. It connects
to the IPC socket, appears as a Wayland client, or both. A container mounts
`$XDG_RUNTIME_DIR/chora/` (in Quadlet: `%t/chora`) and gets `CHORA_SOCKET`
from the environment of the unit.
