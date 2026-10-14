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

A view is a part of the scene and the plugins that drive it (D35):

<!-- parses: view -->
```yaml
# ~/.config/chora/views/work.yaml
name: work
plugins:
- name: tiling
  config: {layout: columns, gap: 12}
scene:
- id: work-corner
  kind: group
  props:
    transform: {translate: [1800, 20]}
- id: work-badge
  kind: rect
  parent: work-corner
  props:
    size: [100, 30]
    color: [0.2, 0.5, 1.0, 1.0]
```

- `choractl view load work` starts each plugin of the view that does not
  run yet, with the `exec` of its manifest, and waits up to 10 s until it
  connects. It passes each `config` to the method that the manifest names
  in `configure`. Then it applies the nodes of the view as one transaction.
- The client `view-work` owns the nodes. A second load replaces them,
  `choractl view unload work` removes them.
- `choractl view save work` writes the nodes of `view-work` as they are
  now, with the plugins of the file that was there, to
  `~/.config/chora/views/work.yaml`, or to `--file PATH`. A plugin or an
  agent changes the nodes as `view-work`, for example
  `choractl --name view-work apply`.
- A node has an `id`, a `kind` (`group`, `rect` or `image`), a `parent`
  (the root by default) and its `props` as in a `create` command.
- A name has letters, digits, `_`, `-` and `.`, but no `.` first, so that
  its file stays in its directory. The file `NAME.yaml` must name the same
  view.
- `choractl view load` also takes the path of a YAML file; the MCP tools
  take names only.
- A plugin that a view starts gets no stdin and no stdout of `choractl`,
  which carry MCP under `choractl mcp`.

`choractl` looks for `NAME.yaml` in `$XDG_CONFIG_HOME/chora/views/` first
(`~/.config/chora/views/` by default), and then in
`/usr/share/chora/views/`.

### Strict YAML

`choractl` reads views and manifests with `serde-saphyr` (D41). Unknown
keys, duplicate keys and unknown tags are errors. Only `true` and `false`
are booleans, so `no` stays the string `no`.

### Plugin manifests

A manifest tells `choractl` how to start and how to configure a plugin. It
lives in `plugins/NAME.yaml` of the same directories:

<!-- parses: manifest -->
```yaml
# /usr/share/chora/plugins/tiling.yaml
name: tiling
exec: chora-tiling
configure: set_layout
description: Tiles the windows as master and stack, columns or rows.
```

| Key | Meaning |
|-----|---------|
| `name` | the client name of the plugin |
| `exec` | the shell command that starts it |
| `configure` | the exported method that takes the `config` of a view; a view without `config` needs none |
| `description` | what the plugin does |

The package installs the manifest of `chora-tiling`.

## chora-tiling

`chora-tiling` reads `~/.config/chora/tiling.yaml`, or the file of
`--config`. Without a file, it uses its default, which
`chora-tiling --print-config` prints. The file writes out every option:

| Option | Meaning |
|--------|---------|
| `modifier` | what `MOD` means in the bindings: Super, Alt, Ctrl or Shift |
| `gap` | the gap around each window, in logical pixels |
| `tags` | the tag names of each output, at most 16 |
| `layouts` | the layouts in the order of `next_layout`; each tag starts with the first |
| `master_width`, `masters`, `columns` | the start values of each tag for the tile layouts |
| `new_window` | `master` or `stack`: where a new window goes |
| `focus` | `click` or `follow_mouse` |
| `bindings` | a binding with `MOD`, and its action |

The layouts and the default bindings follow the awesome window manager.
Each output has its own tags, and each tag its own layout and parameters.
A window can be on several tags, and an output can show several tags. A
floating, maximized or fullscreen window leaves the tiles. The binding
`MOD+Ctrl+r` reads the file again. Mouse moves and resizes of floating
windows do not exist.

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
├── chora-tiling.target              # wants the two units below
├── chora-tiling-wm.service          # chora-tiling
└── chora-tiling-xwayland.service    # xwayland-satellite, if installed
```

The target also wants `waybar.service`, which the waybar package installs.
A copy of the directory in `~/.config/chora/flavors/tiling/` replaces it.

### The XP flavor

The XP flavor is the showroom (D39): a stacking desktop in
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
  and close buttons; a window that draws its own frame, as GTK programs
  do, gets none;
- a taskbar at the bottom of the work area of each output, with the start
  button, one button per window of the current desktop, four desktop
  buttons and a clock;
- the start menu with the user name, programs, places, Log Off and Turn
  Off Computer;
- the wallpaper Bløff below everything.

A click on a window raises and focuses it. A drag of a title bar moves the
window, and a double click maximizes it. A maximized window fills the work
area above the taskbar. A click on a task button shows or minimizes its
window. Buttons light up under the pointer, and so do the entries of the
start menu. Log Off asks every window to close. Turn Off Computer ends
`chora-xp`; it shuts nothing down.

The effects run in the plugin, which computes their meshes and
transforms at 60 Hz:

- **Wobbly windows.** A dragged window bends like jelly: the point under
  the pointer follows it, the far side lags behind, and after the release
  the window shakes until it settles.
- **Magic lamp.** Minimize pours the window into its task button, and a
  click on the button pours it out again.
- **Desktop cube.** Four desktops sit on the faces of a cube. A turn shows
  the cube in perspective, and the windows fly off their faces and land
  again. The taskbar lists the windows of the current desktop.

If frames get too slow, the core drops the effects; `chora-xp` then goes
without them for 15 seconds.

| Binding | Action |
|---------|--------|
| Alt+F4 | close the focused window |
| Super+e | start the terminal |
| Ctrl+Alt+Left, Ctrl+Alt+Right | turn the cube to the previous or next desktop |
| Ctrl+Alt+Shift+Left, Ctrl+Alt+Shift+Right | turn the cube and take the focused window along |
| a desktop button 1 to 4 | turn the cube to that desktop |

The safety layer keeps only Ctrl+Alt with BackSpace, Escape and F1 to F12,
so the cube takes Ctrl+Alt with the arrows. When another client holds
these bindings, Super+Ctrl with the arrows stands in.

| Option | Default | Meaning |
|--------|---------|---------|
| `--theme DIR` | `/usr/share/chora/flavors/xp/theme`, or `CHORA_XP_THEME` | the theme directory |
| `--terminal CMD` | `foot` | the terminal of the start menu and of Super+e |
| `--launcher CMD` | `fuzzel` | All Programs and Run in the start menu |
| `--program TITLE=CMD` | | another program in the start menu; repeat for more |
| `--clock FORMAT` | `%-I:%M %p` | the clock, in the format of strftime |
| `--wallpaper FILE` | Bløff | a PNG or SVG file |
| `--slow-motion FACTOR` | `1` | stretch the magic lamp and the cube, to show them slowly |

To try the flavor from a source tree, start it nested:

```bash
cargo build --workspace
target/debug/chora -- target/debug/chora-xp --theme flavors/xp/theme
```

## Services

A service runs as a Quadlet unit with Podman, or in distrobox. It connects
to the IPC socket, appears as a Wayland client, or both. A container mounts
`$XDG_RUNTIME_DIR/chora/` (in Quadlet: `%t/chora`) and gets `CHORA_SOCKET`
from the environment of the unit.
