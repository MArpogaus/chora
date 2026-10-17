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
├── views/              # one file per view
├── flavors/            # bundles of views, plugins and services
├── plugins/            # plugin manifests
├── shaders/            # GLSL ES shaders
└── chora-tiling.yaml   # the options and bindings of chora-tiling
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
  config: {layout: fair, gap: 12}
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
description: Tiles the windows with the tags and layouts of awesome.
```

| Key | Meaning |
|-----|---------|
| `name` | the client name of the plugin |
| `exec` | the shell command that starts it |
| `configure` | the exported method that takes the `config` of a view; a view without `config` needs none |
| `description` | what the plugin does |

The package installs the manifest of `chora-tiling`.

## chora-tiling

`chora-tiling` reads `chora/chora-tiling.yaml` in `$XDG_CONFIG_HOME`, by
default `~/.config`, or the file of `--config`. Without a file, it uses its
default, which `chora-tiling --print-config` prints. The file lists every
option:

| Option | Meaning |
|--------|---------|
| `modifier` | what `MOD` means in the bindings: Super or Alt |
| `gap` | the gap around each window, in logical pixels |
| `tags` | the number of tags of each output, at most 16 |
| `layouts` | the layouts in the order of `next_layout`; each tag starts with the first |
| `master_width`, `masters`, `columns` | the start values of each tag for the tile layouts |
| `new_window` | `master` or `stack`: where a new window goes |
| `focus` | `click` or `follow_mouse` |
| `bindings` | a binding with `MOD`, and its action; the default file uses every action at least once |

The layouts and the default bindings follow the awesome window manager.
Each output has its own tags, and each tag its own layout and parameters.
A window can be on several tags, and an output can show several tags. A
floating, maximized or fullscreen window leaves the tiles. A click or a
binding raises the window that it focuses; the focus that follows the
pointer raises nothing. The binding
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

When the files of Windows XP are installed in the XDG data directory, as
`get-microsoft-files.sh` of `kde-plasma-winxp-launcher` does, `chora-xp`
uses them instead of the free files of the theme:

- `fonts/WindowsXP/`: XP's Tahoma for the text, Trebuchet MS Bold for the
  titles and Franklin Gothic Medium Italic for the start button. These
  fonts go before the system fonts, so XP's Tahoma wins over the Tahoma
  of newer Windows.
- `wallpapers/WindowsXP/*.jpg`: Bliss (`bliss.jpg`) is the wallpaper
  instead of Bløff, and Control Panel > Desktop lists all of them.
- `winxp-kde/start_flag.png`: XP's flag on the start button, at its own
  size of 20 pixels.
- `sounds/WindowsXP/stereo/`: XP's sounds. `login.wav` plays when
  `chora-xp` starts, `logout.wav` on Log Off, `shutdown.wav` on Turn Off
  and `notify.wav` when a balloon opens. The first of `pw-play`, `paplay`
  and `canberra-gtk-play` on the `PATH` plays them in the background.

`chora-xp` draws everything as image nodes. It rasterizes the SVG files of
the theme with resvg into PNG files in `$XDG_RUNTIME_DIR/chora-xp/`, and
renders each picture once:

- a frame around each window, with its title and the minimize, maximize
  and close buttons; a window that draws its own frame, as GTK programs
  do, gets none;
- a taskbar at the bottom of the work area of each output, with the start
  button, one button per window of the current desktop, four desktop
  buttons and a clock;
- the start menu with the user name, programs, places, the Control Panel,
  Log Off and Turn Off Computer, and All Programs with the applications of
  the desktop files in `$XDG_DATA_HOME/applications` and `$XDG_DATA_DIRS`,
  in program groups by their main category;
- the icons of the desktop, between the wallpaper and the windows;
- the wallpaper below everything: Bliss when it is installed, else Bløff.

A click on a window raises and focuses it. A drag of a title bar moves the
window, and a double click maximizes it. A maximized window fills the work
area above the taskbar. A click on a task button shows or minimizes its
window. Buttons light up under the pointer, and so do the entries of the
start menu. All Programs and its groups open their cascading menus when
the pointer rests on them for 350 ms, or on a click. A click on a program
starts it and closes the menu; a click outside closes the menu. Escape
closes nothing yet: the core takes no grab of a key without Ctrl, Alt or
Super (D45).

The desktop of the first output shows XP's icons in its grid of 75 by 75
pixels: My Documents (`XDG_DOCUMENTS_DIR`, else the home directory) and My
Computer at the top left, then the folders and files of the Desktop folder
(`XDG_DESKTOP_DIR`, else `~/Desktop`) by name, folders first, in columns
from the top. The Recycle Bin sits at the bottom right and shows whether
`$XDG_DATA_HOME/Trash/files` has files. A desktop file shows its name and
icon. Each label is Tahoma in white with a black shadow, in at most two
lines. `chora-xp` looks at the Desktop folder and the trash once a second.
My Network Places is left out, because without a network browser it opens
nothing.

A click selects an icon: its label turns white on dark blue, with a dotted
focus rectangle while no window has the focus. A click on the empty
desktop takes the selection away. A double click opens the icon: a folder,
My Documents and My Computer (`/`) with `xdg-open`, until XP's Explorer
comes; the Recycle Bin with `gio open trash:///`, else its folder; a
desktop file runs its program; any other file opens with `xdg-open`. A drag
moves an icon to the nearest free cell of the grid, until `chora-xp` ends.
XP has one desktop, so the icons stay in place while the cube turns: each
face of the cube shows them on the wallpaper.

The right button opens XP's context menus: white, with a gray border, a
soft shadow and the selection in XP's blue. A click on a row does what it
says and closes the menu; a click outside closes it.

- **The taskbar**: Cascade Windows and Tile Windows Horizontally (one
  above the other) or Vertically (side by side) arrange the windows of
  the current desktop on the output of the menu; Show the Desktop
  minimizes them. Lock the Taskbar shows a check mark and changes
  nothing else, because `chora-xp` neither moves nor sizes the taskbar.
  Toolbars and Task Manager are gray: `chora-xp` has neither.
- **The clock**: Adjust Date/Time, then the rows of the taskbar.
  Customize Notifications is gray.
- **The desktop**: Arrange Icons By Name or Type puts the icons back
  into their columns, in that order; Refresh shows them again; Properties
  opens the start menu with Control Panel > Desktop. Paste and Paste
  Shortcut are gray.

The Control Panel opens as a menu, as with XP's "Display as a menu". A
check mark shows the current choice of each entry, and a click on an
entry applies it and closes the menu:

- **Desktop**: the wallpaper of every output, from the PNG, JPEG and SVG
  files in `theme/wallpaper/`, `/usr/share/backgrounds` and
  `wallpapers/WindowsXP` of the XDG data directories, without their
  subdirectories and at most 20 of each directory;
- **Display**: the scale of the output of the menu, 100 %, 125 %, 150 % or
  200 %, through `output.configure`;
- **Effects**: wobbly windows, the magic lamp, the desktop cube and XP's
  sounds, on or off until `chora-xp` ends.
- **Taskbar and Start Menu**: opens Taskbar and Start Menu Properties.

Properties of the taskbar menu, and Control Panel > Taskbar and Start
Menu, open XP's Taskbar and Start Menu Properties. `chora-xp` draws this
dialog itself, in XP's Luna style: the frame of the theme with the
title and the close button, tabs, group boxes, check boxes, radio
buttons and push buttons that light up under the pointer. A drag of the
title bar moves it. The close button and Cancel close it without a
change, OK takes the changes and closes it, and Apply takes them and
keeps it open. The dialog stays above the windows, and the keyboard
does not work it (D45). The settings last until `chora-xp` ends:

- **Taskbar** shows a preview of the taskbar above its check boxes. Lock
  the taskbar only changes the check mark, as in the context menu.
  Auto-hide the taskbar lets the taskbars slide away half a second after
  the pointer left them, down to a strip of 2 pixels at the bottom
  edge; the pointer on the strip brings a taskbar back. The work area
  then reaches down to the strip, and maximized windows grow into it.
  Keep the taskbar on top of other windows puts the taskbars above the
  windows, or below them when it is off. Show the clock shows the clock
  in the notification area, or shrinks the area without it. Group
  similar taskbar buttons, Show Quick Launch, Hide inactive icons and
  Customize are gray: `chora-xp` has neither grouping, Quick Launch nor
  hidden icons.
- **Start Menu** shows a preview of the start menu. The Classic Start
  menu and the Customize buttons are gray: `chora-xp` has only XP's
  start menu.

Log Off and Turn Off Computer open XP's dialogs in the middle of the
output, and the desktop behind them fades to gray in 1.5 seconds. In Log
Off Windows, Log Off asks every window to close; Switch User is always
disabled, because chora has no second session to switch to. In Turn off
computer, Stand By runs `systemctl suspend`, and is disabled without
`systemctl`; Turn Off ends `chora-xp` and shuts nothing down; Restart
runs `chora-xp` again in the same process, with the same arguments, and
the new instance adopts the windows. Cancel or a click outside the dialog
closes it. The buttons light up under the pointer; the keyboard does not
work the dialogs, for the same reason as Escape.

`chora-xp` serves notifications (`org.freedesktop.Notifications` on the
session bus), so that `notify-send` and programs show XP's balloon tips:
a pale yellow balloon with a black border above the notification area of
the first output, with a tail towards the tray, the icon of the
notification, a bold title, the text wrapped at 300 pixels and a close
button. While a balloon shows, the notification area grows for its icon,
or XP's information icon, left of the clock, as on XP, and the tail
points at it. One balloon shows at a time, and the others wait their turn. A
balloon closes after its time, which is 10 seconds by default and, as on
XP, at least 10 and at most 30 seconds; with a time of 0 it stays until a
click. A click on the close button dismisses it; a click on the balloon
invokes the action `default` of the notification, if it has one, and
dismisses it too. If another server, such as GNOME Shell, owns the name
on the bus, `chora-xp` logs one line and shows no balloons; it never
takes the name from its owner.

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
| `--terminal CMD` | `foot` | the terminal of the start menu and of Super+e; `CMD -e` runs the programs with `Terminal=true` |
| `--launcher CMD` | `fuzzel` | Run in the start menu, and All Programs when no desktop file lists a program |
| `--program TITLE=CMD` | | another program in the start menu; repeat for more |
| `--clock FORMAT` | `%-I:%M %p` | the clock, in the format of strftime |
| `--wallpaper FILE` | Bliss if installed, else Bløff | a PNG, JPEG or SVG file |
| `--slow-motion FACTOR` | `1` | stretch the magic lamp and the cube, to show them slowly |
| `--no-sounds` | | start without XP's sounds; Control Panel > Effects > Sounds turns them on |

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
