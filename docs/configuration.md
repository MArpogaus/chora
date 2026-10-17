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

`chora-tiling` connects with `resume` (see [API](api.md#ipc-methods)). It
keeps the tags and the layouts of each output, and the tags, the states
and the stacking of each window, in the `meta` of its nodes. A restart
within the grace time rebuilds them from there, for example `choractl
restart tiling` after a rebuild.

`chora-tiling` claims the box around the work areas of its outputs
(D55). `--output NAME`, repeatable, limits it to some outputs, as for
`chora-xp`, so that each window manager takes its own outputs. It leaves
a new window that the core reserves for another client to that client.
When the last of its `--output` outputs goes, it gives its windows away,
and another window manager adopts them; it takes new windows again when
one of its outputs comes back.

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
the theme with resvg into PNG files in `$XDG_RUNTIME_DIR/chora-xp/`. It
renders each picture once, at the largest scale of the outputs, but at
most 4. Thus text and icons stay sharp on a scaled output. After a change
of the scale, it renders the pictures again. The borders of windows and
dialogs and the background of the taskbar stay at scale 1. The core
reads the insets of a slice in pixels of the file. The pictures:

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
window, and a double click maximizes it. `chora-xp` claims the box around the work areas
of its outputs (D55). `--output NAME` limits it to some outputs. A window
dragged out of these work areas goes to the window manager `tiling` with
`remove` and `to`. Without `tiling`, the window stays. A drag of the border of the frame
resizes the window: the 4 pixel border and 6 pixels of the shadow outside
it take the press, and the corners reach 16 pixels along both edges. The
client gets the new size once per frame while the button is down, and the
edges that the drag does not move stay in place. A window is at least 112
by 35 pixels with its frame, so that the buttons of the title bar fit. A
window that draws its own frame resizes from a strip of 6 pixels around
it. A maximized window does not resize, and a window does not wobble while
it resizes. The pointer keeps its arrow over the border: plugins cannot
set the cursor. The core reports no size limits of a client yet, so a
client that wants another size than it gets simply takes its own, and the
frame follows it.

A right click on a title bar, a click on the icon of a title bar and a
right click on a task button open XP's system menu, in the look of the
context menus below: Restore, Move, Size, Minimize, Maximize and Close in
bold with `Alt+F4`, with XP's glyphs. Restore is gray unless the window is
maximized or minimized, Minimize is gray for a minimized window and
Maximize for a maximized one. Move and Size are always gray, as XP works
them with the arrow keys, and the keyboard does not reach `chora-xp`
(D45). The click on the icon opens the menu below the icon, the others at
the pointer. A double click on the icon closes the window, as on XP. A maximized window fills the work
area above the taskbar. When an output changes, for example its mode or
its scale, its shown windows move back into its work area. A window that
does not fit shrinks to it. A click on a task button shows or minimizes its
window. Buttons light up under the pointer, and so do the entries of the
start menu. All Programs and its groups open their cascading menus when
the pointer rests on them for 350 ms, or on a click. A click on a program
starts it and closes the menu; a click outside closes the menu. The open
menu takes the keyboard. Up and Down move in a column or a cascade. Left
and Right go between the columns and into and out of the cascades.
Return starts the entry. Escape closes the innermost cascade, then the
menu.

The desktop of the first output shows XP's icons in its grid of 75 by 75
pixels: My Documents (`XDG_DOCUMENTS_DIR`, else the home directory) and My
Computer at the top left, then the folders and files of the Desktop folder
(`XDG_DESKTOP_DIR`, else `~/Desktop`) by name, folders first, in columns
from the top. The Recycle Bin sits at the bottom right and shows whether
`$XDG_DATA_HOME/Trash/files` has files. A desktop file shows its name and
icon. A program, a desktop file or a window without an icon gets XP's
icon of a program, `application-x-executable` of the icon theme. Each
label is Tahoma in white with a black shadow, in at most two
lines. `chora-xp` looks at the Desktop folder and the trash once a second.
The icons that stay keep their cells. A new icon takes the first free
cell of the columns.
My Network Places is left out, because without a network browser it opens
nothing.

A click selects an icon: its label turns white on dark blue, with a dotted
focus rectangle while no window has the focus. A click on the empty
desktop takes the selection away. A double click opens the icon: a
folder, My Documents, My Computer and the Recycle Bin in XP's Explorer
(see below); a desktop file runs its program; any other file opens with
`xdg-open`. A drag
moves an icon to the nearest free cell of the grid, until `chora-xp` ends.
A drag on the empty desktop draws XP's selection rectangle, in XP's
selection blue with a translucent fill and a line of 1 pixel, and selects
every icon whose picture or first line of its label it touches, while it
grows and shrinks. A drag of one of the selected icons moves all of them,
each to the cell at the same offset; if one of them would land on an icon
that stays, or two in one cell, they all go back. A click on one of the
selected icons selects it alone, and a click on the empty desktop selects
nothing.
XP has one desktop, so the icons stay in place while the cube turns: each
face of the cube shows them on the wallpaper.

The right button opens XP's context menus: white, with a gray border, a
soft shadow and the selection in XP's blue. A click on a row does what it
says and closes the menu; a click outside closes it. An open menu takes
the keyboard. Up and Down move the highlight over the enabled rows.
Right opens a submenu, Left closes it. Return does what the row says.
Escape closes the submenu, then the menu. Home and End go to the first and
the last row. Once a key works the menu, it underlines XP's access keys.
A letter acts on the row of its access key, or goes to the next one of
several. A drop-down list opens on its current value.
While no window has the focus, the desktop takes the keyboard: Shift+F10
and the Menu key open the menu of the selected icon, or of the desktop.

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
  opens Display Properties (see below). Paste and Paste Shortcut are
  gray.
- **An icon of the desktop**: the right button selects the icon and
  opens XP's menu of its kind. When the icon is part of the selection,
  the selection stays. The rows act on the icon under the pointer. Open, in bold, does what a double click does.
  A folder and My Documents have Explore, which opens the folder in
  Explorer, as My Computer and the Recycle Bin do. Delete of a file or a
  folder of the Desktop folder asks in XP's Confirm File Delete or
  Confirm Folder Delete, and Yes moves it to the trash of the
  freedesktop.org specification: `$XDG_DATA_HOME/Trash/files`, with its
  `.trashinfo` file in `Trash/info` that holds its percent-encoded path
  and the date. A name that the trash holds already gets a number, as
  `notes.2.txt`. A file on another file system than the trash stays
  where it is, and `chora-xp` logs why. Properties shows the type, the
  folder, the size or the number of items in a folder, and the times it
  was created, changed and opened. The Recycle Bin has Empty Recycle
  Bin while it holds files, and its Properties count them. Send To,
  Open With, Search, Cut, Copy, Create Shortcut and Rename are gray, as
  are Delete of My Documents and all rows of My Computer but Open and
  Explore: its Properties would be XP's System Properties.
- **A program or a place of the start menu, or a program of All
  Programs**: Open in bold, Explore for the folders, Remove from This
  List for the programs of the left column, which leave the menu until
  `chora-xp` ends, and Properties with the name, the icon and the
  command or the folder. Run as, Pin to Start menu, Send To and Rename
  are gray.

`chora-xp` draws XP's Windows Explorer itself, as the client of a
window of its own: the window has XP's frame and a task button with the
icon and the name of the place, and it raises on a click, minimizes
through the magic lamp, maximizes and wobbles like the windows of
clients. It opens at 785 by 559 pixels, the size of XP's My Computer, or
smaller on a small output, and resizes from its edges down to 428 by 294
pixels: the task pane keeps its width, and the view and its scroll bar
take the rest. From the top:

- the menu bar with File, Edit, View, Favorites, Tools and Help, and
  XP's flag when it is installed. A click opens a menu: File > Close
  closes the window; View has Tiles, Go To (Back, Forward, Up One Level,
  My Documents, My Computer, Recycle Bin) and Refresh. The other rows
  are gray;
- the Standard Buttons: Back, Forward and Up, as the history of the
  window allows, and Views with Tiles. Search and Folders are gray. Up
  goes from My Computer and the Recycle Bin to the Desktop folder, where
  they sit on XP;
- the address bar with the path and the icon of the place, its list of
  places, and Go, which shows the place again;
- the blue task pane with XP's boxes: the tasks of the place, Other
  Places and Details. A click on the title of a box closes or opens it.
  The links of Other Places go to the parent folder, My Documents and My
  Computer, and Shared Documents when it exists, in the same window;
  Control Panel and Change a setting open the Control Panel of the start
  menu. The tasks that change files and My Network Places are gray.

The view of a folder is XP's Tiles view: a 48 pixel icon for each folder
and file, its name, and in gray its type and size as XP writes them,
such as `Text Document` and `12.3 KB`. Folders come first, then the files
by name without case and with numbers by their value (`file2` before
`file10`); hidden files, whose name starts with a dot, are left out. The
icon comes from the extension: folder, text, image, audio, video,
archive, program (also a file that may run) or other. A click selects
an item, and Details shows its type, date and size; a click on the empty
view selects nothing. A double click opens a folder in the same window,
with Back, Forward and Up, and a file with `xdg-open`. A view longer
than the window gets XP's blue scroll bar: its arrows scroll by 40
pixels, a click on the track by a page, and the thumb follows a drag.
A notch of the mouse wheel scrolls by 120 pixels, three lines as on XP.
A touchpad scrolls as far as the fingers move.

My Computer shows XP's groups under their headings with the blue line:
Files Stored on This Computer with Shared Documents (`XDG_PUBLICSHARE_DIR`,
when it exists) and "USER's Documents" (My Documents); Hard Disk Drives;
and Devices with Removable Storage. The drives come from
`/proc/self/mounts`: the mounts of disk file systems (btrfs, ext2 to
ext4, xfs, f2fs, vfat, exfat, NTFS and others) and of discs, each device
once at its shortest mount point, without the bind mounts of a container
below `/dev`, `/sys`, `/proc`, `/etc`, `/var/lib` and `/var/log`. A disk
is "Local Disk (/)"; a medium below `/run/media` or `/media`, or a
disc, is removable and goes by the name of its directory. Details shows
the file system, the free space and the size, from `df`. A double click
opens a drive or a folder in the same window.

The Recycle Bin lists the trash of the freedesktop.org specification,
`$XDG_DATA_HOME/Trash/files`, with the folder each item came from and
the date it went, from its file in `Trash/info`. Recycle Bin Tasks has
Empty the Recycle Bin, gray while the bin is empty. It asks first, in XP's
Confirm File Delete or Confirm Multiple File Delete, and Yes deletes the
files and folders of the trash and their info files for good. Restore
all items is gray.

My Documents, My Pictures and My Music of the start menu
(`XDG_DOCUMENTS_DIR`, `XDG_PICTURES_DIR` and `XDG_MUSIC_DIR`, else the
home directory and its folders `Pictures` and `Music`) and My Computer
open in Explorer too, and the window calls these folders by their names
in XP.

Desktop icons dropped on the window of a folder go into the folder, or
into the folder item under the pointer: a file or a folder moves on the
same file system and is copied to another one, as XP moves on one drive
and copies to another. When the folder has a file of the name, XP's
Confirm File Replace asks first, and Yes replaces it; a folder of the
name stays, and `chora-xp` logs it. A drop on the window of the Recycle Bin, or on
its desktop icon, sends the files to the Recycle Bin. Beyond
this and Empty the Recycle Bin, the window changes no file. The keyboard does not work it (D45). Its icons come from the icon
theme, XP's own when `WindowsXP` of `get-microsoft-files.sh` is
installed.

The Control Panel opens as a menu, as with XP's "Display as a menu". A
check mark shows the current choice of each entry, and a click on an
entry applies it and closes the menu:

- **Date and Time**: opens Date and Time Properties;
- **Desktop**: the wallpaper of every output, from the PNG, JPEG and SVG
  files in `wallpapers/WindowsXP` of the XDG data directories,
  `theme/wallpaper/` and `/usr/share/backgrounds`, without their
  subdirectories and at most 20 of each directory;
- **Display**: opens Display Properties;
- **Effects**: wobbly windows, the magic lamp, the desktop cube and XP's
  sounds, on or off until `chora-xp` ends;
- **Taskbar and Start Menu**: opens Taskbar and Start Menu Properties.

Properties of the taskbar menu, and Control Panel > Taskbar and Start
Menu, open XP's Taskbar and Start Menu Properties. `chora-xp` draws this
dialog itself, in XP's Luna style: the frame of the theme with the
title and the close button, tabs, group boxes, check boxes, radio
buttons and push buttons that light up under the pointer. A drag of the title bar moves it,
and it wobbles as a window does, as do Date and Time, Display
Properties and the other dialogs that XP lets move; the Log Off and
Turn Off dialogs stay fixed in the middle. The close button and Cancel close it without a
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

A double click on the clock, Adjust Date/Time of the clock menu and
Control Panel > Date and Time open XP's Date and Time Properties, drawn
in the same way:

- **Date & Time** shows the month in a drop-down list and the year with
  arrow buttons, the month calendar with weeks from Sunday and the chosen
  day in blue, the analog clock with its moving hands and the digital
  time. A click on the list opens the twelve months, the arrows change
  the year and a click on a day chooses it. Below, it names the time
  zone of the system: `TZ`, else the link `/etc/localtime`.
- **Time Zone** shows the zone with its offset from GMT. XP's world map
  is left out.

chora cannot set the clock of the system without privileges, so the
dialog changes nothing: OK, Cancel and the close button close it, and
Apply stays gray. Internet Time is left out.

Properties of the desktop menu and Control Panel > Display open XP's
Display Properties, 404 by 455 pixels, drawn in the same way. It opens on
the tab that was open when it last closed, Themes the first time. Apply
and OK show the background on every output and set the scale of the
output of the dialog; Cancel and the close button close it without them.
The settings last until `chora-xp` ends:

- **Themes** shows the sample of the theme: the wallpaper with an active
  window and the Recycle Bin. Windows XP is the only theme, so the list,
  Save As and Delete are gray.
- **Desktop** shows XP's monitor with the chosen background, and the
  list Background: (None) and the wallpapers of Control Panel > Desktop,
  with arrows and a track that scroll it. A click on a row chooses it.
  Position places the picture: Center at its own size in the middle,
  Tile repeated from the top-left corner, Stretch over the screen out of
  its aspect ratio, and Fill, which XP lacks, over the screen at its
  aspect ratio with the overflow cut off, as Windows 7 names it. Fill is
  the default, so that Bliss keeps its shape on a wide output; XP's own
  default is Stretch. Color chooses the color of the desktop behind a
  centered picture, or without one, from a palette of twelve colors with
  XP's desktop blue first. Browse is gray: `chora-xp` has no file
  chooser yet. Customize Desktop is gray too.
- **Screen Saver** shows the monitor and XP's controls, all gray:
  `chora-xp` has no screen saver and no power settings.
- **Appearance** shows a sample with an inactive window, an active
  window and a message box on the color of the desktop. The style, the
  color scheme and the font size are gray, as is Advanced. Effects opens
  a small dialog with check boxes for the wobbly windows, the magic lamp,
  the desktop cube and the sounds, as Control Panel > Effects; OK takes
  them.
- **Settings** shows the monitor with the desktop and a taskbar, and the
  name of the output. The slider of Screen resolution chooses the scale
  of the output, 200 %, 150 %, 125 % or 100 % from Less to More, and
  names the size that it gives, such as "1280 by 800 pixels". Color
  quality, Troubleshoot and Advanced are gray.

Log Off and Turn Off Computer open XP's dialogs in the middle of the
output, and the desktop behind them fades to gray in 1.5 seconds. In Log
Off Windows, Log Off asks every window to close. Switch User is always
disabled, because chora has no second session to switch to. In Turn off
computer, Stand By runs `systemctl suspend`. Without `systemctl`, Stand
By is disabled. Turn Off ends `chora-xp` and shuts nothing down. Restart
runs `chora-xp` again in the same process, with the same arguments. The
core keeps the nodes of `chora-xp` for the grace time of `resume` (see
[API](api.md#ipc-methods)). The
windows keep their places, desktops, maximized state and stacking. The
Explorer windows open again. The current desktop stays, and its top
window gets the focus. Cancel or a click outside the dialog
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

If frames get too slow because of `chora-xp`, the core drops its effects;
`chora-xp` then goes without them for 15 seconds. A drop of the effects
of another client changes nothing.

| Binding | Action |
|---------|--------|
| Alt+F4 | close the focused window |
| Ctrl+Escape | open or close the start menu |
| Alt+Space | open the system menu of the focused window; `chora-xp` takes the key only while one of its windows has the focus |
| Alt+Tab, Alt+Shift+Tab | open XP's switcher with the windows of the desktop, the most recent first. Tab chooses the next window, Shift+Tab the previous window. The release of Alt activates the chosen window. Minimized windows come last. Escape closes the switcher without a switch. |
| Super+e | start the terminal |
| Ctrl+Alt+Left, Ctrl+Alt+Right | turn the cube to the previous or next desktop |
| Ctrl+Alt+Shift+Left, Ctrl+Alt+Shift+Right | turn the cube and take the focused window along |
| Super and a drag with the left button | turn the cube with the pointer. A drag over the width of the output is a quarter turn. At the release, the cube turns to the nearest desktop. |
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
