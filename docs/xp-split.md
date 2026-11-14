# Split of chora-xp

The design of the XP split (D73). `chora-xp` is one process of 18,258
lines in 33 files. It splits into neutral blocks and the XP
gimmick. Each block is a plugin that any flavor can use. The gimmick is
what stays XP: the look files, the dialogs, the cube and the paperclip.
The steps X1 to X7 of the roadmap do the split. Each step changes plugins
only and keeps `tests/xp.sh` green.

## Rules

- A block is a plugin with its own client name, settings and asset folder.
  It does one job.
- Blocks do not call each other in code. They meet through the core: the
  events of the core, `methods.call`, nodes and the session bus.
- No plugin depends on another plugin. Shared code lives in `chora-kit`
  or in `chora-draw`. One exception is open: `chora-xp` may link the
  library of `chora-stack`, decided in X5.
- A block loads its pictures and a few values from an asset folder. The
  block ships a neutral folder. A flavor ships another. There is no theme
  system and no API for looks (D73).
- No crate stays unused. A crate appears with the step that moves the
  first code into it.
- The blocks go to the repository `chora-plugins` with `chora-xp` (D77).

## Blocks

| Block | Crate | Client name | Job | First step |
|-------|-------|-------------|-----|------------|
| Notifications | `chora-notify` | `notify` | the notification server on the session bus; shows each notification as a card | X2 |
| Assistant | `chora-assistant` | `assistant` | shows the state and the speech of the voice agent (D65) | X2 |
| Desktop | `chora-desk` | `desk` | wallpaper, icons of the Desktop folder, context menu of the desktop | X3 |
| Panel | `chora-panel` | `panel` | taskbar, start menu, clock and tray (D51) | X4 |
| Stacking window manager | `chora-stack` | `stack` | stacking windows with frames, focus, Alt+Tab, resize, hand-over | X5 |
| XP gimmick | `chora-xp` | `xp` | the XP look files, the dialogs of D63, the cube, the sounds, the paperclip | X7 |

`chora-draw` is a library, not a block. It has no client, settings or
asset folder. It rasterizes SVG files and text into image nodes and holds
the context menu widget. The blocks share it, and X2 creates it.

Why these names:

- Each name is `chora-` and the noun of the job. `chora-stack` names the
  stacking window manager next to `chora-tiling`.
- The crate for the desktop is `chora-desk`. The name `chora-desktop`
  would collide with the organization of D36 and with "desktop" as the
  whole session.
- The assistant keeps its own block. The paperclip is one look of it.
- Effects stay a library of `chora-kit` (D73). Wobbly windows and the magic
  lamp are already there. Step X6 adds the pass-through of D58.

### Only two crates in X1

The skeleton holds `chora-notify` and `chora-assistant`. Both get their
first real user in X2. Each starts as a plugin through `chora-kit`,
subscribes to no event and exports no method.

The other crates have no code to hold before their step. `chora-draw`
comes with X2, because the balloon and the paperclip both need
the rasterizer. `chora-desk`, `chora-panel` and `chora-stack` come with
X3, X4 and X5. An empty crate before that would stay unused (D73).
The skeleton is not in the RPM or the Arch package yet. X2 adds the
binaries.

## Where the files go

Line counts are from `wc -l` on 2026-11-09. `wm.rs` has 3,324 lines and
holds the parts of every block. Each step takes its functions out of it.

| File of `plugins/chora-xp/src` | Lines | Goes to |
|-----|------|---------|
| `notify.rs` | 274 | notifications |
| `balloon.rs` | 506 | notifications: queue, wrapping, layout and placement. The tail and the colors become assets |
| `assistant.rs` | 664 | assistant: poses and the speech balloon. The paperclip drawing goes to the XP asset folder in X7 |
| `wm/voice.rs` | 260 | assistant |
| `wallpaper.rs` | 204 | desktop |
| `desktop_icons.rs` | 1149 | desktop |
| `wm/items.rs` | 86 | desktop for the icons, panel for the start menu entries |
| `taskbar.rs` | 710 | panel |
| `startmenu.rs` | 1166 | panel |
| `cascade.rs` | 378 | panel |
| `apps.rs` | 148 | panel |
| `window.rs` | 587 | stack |
| `decoration.rs` | 489 | stack: the frame from the asset folder |
| `switcher.rs` | 176 | stack |
| `wm/resize.rs` | 121 | stack |
| `wm/restart.rs` | 175 | stack |
| `wm/effects.rs` | 307 | done: the driver is `chora_kit::effects`; the file keeps the cube turn and the frame of `chora-xp` |
| `wm.rs` | 3324 | split by function: events and keys, window handling and `give` to stack; task list, start menu and tray clicks to panel; icon and wallpaper clicks to desktop; dialogs and cube to the gimmick |
| `theme.rs` | 894 | `chora-draw` |
| `text.rs` | 416 | `chora-draw` |
| `icons.rs` | 16 | `chora-draw` |
| `context.rs` | 1225 | `chora-draw` for the menu widget; the menu contents stay with the block that opens the menu |
| `cube.rs` | 327 | gimmick |
| `wm/desktops.rs` | 295 | gimmick |
| `dialog.rs` | 1215 | gimmick: the Luna controls |
| `session.rs` | 413 | gimmick: Log Off and Turn Off |
| `display.rs` | 1093 | gimmick: Display Properties |
| `datetime.rs` | 506 | gimmick: Date and Time Properties |
| `taskbar_dialog.rs` | 286 | gimmick: Taskbar and Start Menu Properties |
| `control.rs` | 243 | gimmick: the Control Panel menu |
| `sound.rs` | 80 | gimmick |
| `config.rs`, `main.rs` | 277, 248 | each block gets its own settings and `main`; the gimmick keeps what is left |

`theme.rs` knows seven fixed SVG files today. In `chora-draw` it loads the
files that the calling block names. The block, not the library, knows its
asset folder.

## How blocks work together

A block needs nothing from another block at build time. At run time:

- The panel, the notifications and the assistant draw into the root
  `screen:NAME` of an output. The nodes stay fixed to the screen, above the
  scene and below the layer shell (D43).
- The panel calls `output.reserve` for the space of its bar. The work area
  of the output shrinks, and the window manager places windows in it.
- The panel learns about windows from the events `surface.*`. A click on a
  task button calls `minimize`, `restore` or `activate` on the client that
  its setting `window_manager` names.
- The desktop draws at the bottom of the scene. Step X3 checks the order
  against the windows of the window manager and records it in
  `docs/api.md` only if the core must change.
- The notifications own `org.freedesktop.Notifications` on the session
  bus. Only one program can hold the name, so a flavor starts one server.
- The assistant subscribes to `voice.*` and shows the state.
- A dialog of the gimmick changes the setting of a block with
  `methods.call` of `settings.set`, as D63 says for the dialogs. A
  button of a block starts a dialog with a shell command, for example
  `choractl call xp turn_off`.

## Methods and settings

Each block exports `settings.get`, `settings.set` and `settings.reload`
through the runtime of `chora-kit` (D62). The file is
`$XDG_CONFIG_HOME/chora/chora-NAME.yaml`. The default is the file
`NAME.yaml` next to the crate, compiled into the binary. A key that the
file of the user leaves out takes the default. The skeleton of X1 exports none of them yet. Each block adds them
in its step.

| Block | Methods besides the settings | Settings |
|-------|------------------------------|----------|
| `notify` | none; the session bus is its interface | `position` (a corner of the output), `shortest` and `longest` (seconds on screen) |
| `assistant` | none; it reacts to `voice.*` | `enabled`, `position` |
| `desk` | none | `wallpaper` (`picture`, `position`, `color`), `terminal`, `file_manager`, `icons` (on or off) |
| `panel` | `start_menu {open}` | `terminal`, `launcher`, `file_manager`, `programs`, `clock_format`, `taskbar` (`auto_hide`, `on_top`, `clock`), `session` (the four commands of the start menu footer), `window_manager` |
| `stack` | `minimize`, `restore`, `activate`, `restart`, `window.give`; the keys `Super+Alt+1` to `Super+Alt+4` give the focused window to tiling, xp, mc or construct | `terminal`, `effects` (`wobbly`, `lamp`) |
| `xp` | `turn_to`, `log_off`, `turn_off`, and the methods of `stack` under the client name `xp` | `session` (the commands that the dialogs run), `effects` (`cube`, `sounds`), `taskbar_locked` |

Notes:

- The names and the types of the settings are those of `xp.yaml` today.
  The table lists only where each setting moves.
- A setting that several blocks need, such as `terminal`, stays in each of
  them. A flavor sets the same value for all. A shared value would need a
  new interface.
- `taskbar.locked` does nothing today. It stays a check mark of the dialog
  and moves to the gimmick as `taskbar_locked`.
- Wobbly windows and the magic lamp default to off in `stack`. The XP
  defaults turn them on.
- Settings are the choices of the user. Values of the asset folder are the
  choices of the flavor author. The two files stay apart.
- The default `position` of `notify` and `assistant` is `bottom_right`, where
  the notification area and the paperclip of the XP flavor are, so that X2
  needs no mechanism for the defaults of a flavor.

## Asset folder of a block

A block loads its look from one folder. The flag `--assets DIR` or the
variable `CHORA_NAME_ASSETS` names it, as `--theme` and `CHORA_XP_THEME`
do today. It is not a setting (D62 keeps `--theme` as a flag).

```text
assets/
├── README.md       # source and license of every file
├── values.yaml     # sizes, colors, the font family, a few margins
├── *.svg           # the pictures, with the element ids the block lists
└── fonts/          # optional
```

Rules:

- The neutral folder `plugins/chora-NAME/assets` of a block is created in
  the step that creates the block. X2 creates the first two.
- Without the flag, a block uses the installed neutral folder
  `/usr/share/chora/NAME/assets`. Run from `target/<profile>/`, it uses
  `plugins/chora-NAME/assets` of the source tree. This is the order of
  `resolve_theme` in `chora-xp` today, without the flavor step.
- A folder replaces the neutral one. It does not merge. A missing or
  broken picture stops the start with an error that names the file.
- A key that `values.yaml` leaves out takes the value that the block
  compiles in.
- The block documents the file names and element ids that it reads. The
  neutral folder is the reference. A test loads each shipped folder.

The neutral folder:

- Pictures drawn for chora, GPL-3.0-or-later. No third-party artwork.
- No font file. Text uses `sans-serif` of the system.
- A flat look: a one pixel frame, three round title buttons, a plain
  bar, a card with rounded corners, a dot for the assistant.
- Wallpaper: a color, no picture.

Where the XP files go:

| Today in `flavors/xp/theme` | Goes to |
|-----------------------------|---------|
| `expose/*` | `assets/stack` |
| `winxp/winxp.svg`, `winxp/tasks.svg`, `winxp/start-mark.svg`, `winxp/user.svg`, `winxp/cascade-arrow*.svg` | `assets/panel` |
| `winxp/balloon-close*.svg` | `assets/notify`; `balloon-info.svg` went with the icon of the notification area |
| `wallpaper/bloff.png` | `assets/desk` |
| `winxp/dialog*.svg`, `winxp/log-off.svg`, `winxp/turn-off.svg`, `winxp/switch-user.svg`, `winxp/stand-by.svg`, `winxp/restart.svg`, `winxp/check*.svg` | the folder of the gimmick |
| the paperclip, drawn in `paperclip.rs` of `chora-assistant` until X7 | `flavors/xp/assets/assistant` as SVG files |
| `fonts/*` | one folder, linked into each folder that needs it |
| `cursors/ModernXP` | `flavors/xp/cursors`, installed as today |

The Arch package links the fonts of each XP folder to the theme. The RPM
carries a copy in each folder, because `cargo-generate-rpm` cannot package a
link.

The licenses stay with their files. `theme/README.md` splits into the
`README.md` of each folder.

## How a flavor composes blocks

A flavor is a directory of systemd units (D50). It picks blocks and gives
each its asset folder. The XP flavor after X7:

```text
flavors/xp/
├── chora-xp.target             # wants all units below
├── chora-xp.service            # chora-xp: stack and the gimmick
├── chora-xp-panel.service      # chora-panel --assets .../assets/panel
├── chora-xp-desk.service       # chora-desk --assets .../assets/desk
├── chora-xp-notify.service     # chora-notify --assets .../assets/notify
├── chora-xp-assistant.service  # chora-assistant --assets .../assets/assistant
├── chora-xp-xwayland.service
├── assets/                     # one folder per block, and the gimmick
└── cursors/
```

`chora-xp` links the library of `chora-stack`. It is the window
manager of the XP session, with the cube and the dialogs added. It exports
the methods of `stack` under the client name `xp`, so the XP panel needs
`window_manager: xp`. The decision to link the
library and not to run two window managers is open until X5.

A flavor without the gimmick uses the neutral folders. A plain stacking
flavor starts `chora-stack`, `chora-panel`, `chora-desk` and
`chora-notify` without `--assets`. The tiling flavor stays as it is and
keeps its tools (D51).

## Steps

| Step | Moves | Check |
|------|-------|-------|
| X2 | `chora-draw` with `theme.rs`, `text.rs` and `icons.rs`; the files of notifications and assistant; their neutral folders; units, packaging and `--assets`; the paperclip stays drawn in code | `tests/xp.sh` passes with both as separate units |
| X3 | `chora-desk` with the wallpaper, the icons and the desktop menu; `context.rs` moves into `chora-draw`; `bloff.png` to `assets/desk` | a capture shows the wallpaper from the neutral folder |
| X4 | `chora-panel` with taskbar, start menu, cascades, programs, clock and tray; `output.reserve` for the bar; the footer buttons call `choractl call xp` | the start menu starts a program; the task list shows its window |
| X5 | `chora-stack` with windows, frames, switcher, resize, restart and the driver of the effects; `chora-xp` links its library | windows move, resize and switch with Alt+Tab |
| X6 | a function of `chora-kit` for `update` and `animate` requests (D58); the driver of the effects came early, see below | a request of an effects plugin reaches a window of `chora-tiling` and of `chora-stack` |
| X7 | the XP folders, the dialogs of D63, the paperclip as SVG files, the cube, the sounds; `theme/` is gone | `tests/xp.sh` passes; a flavor without the gimmick shows a plain desktop |

Each step also:

- adds the binaries to the RPM and the Arch package, and its units to the
  flavor;
- adds the settings to `docs/configuration.md` and to
  `tests/settings-docs.sh`;
- updates `tests/xp.sh` to start the new units;
- removes the moved code from `chora-xp`, so the old and the new code never
  run side by side.

## The driver of the effects

The driver moved into `chora-kit` as `chora_kit::effects` ahead of X6.
It has hooks for a grab, a move, a release, a minimize, a restore and a
frame, and returns `mesh` updates for the nodes that the caller owns, so
every window manager sends them as the owner and the core needs no new
rule. A window manager opts in with `effects: [wobbly, lamp]`. `chora-xp`
turns on both through its Control Panel toggles; `chora-tiling` takes
`wobbly` from its setting `effects`, off by default, and refuses `lamp`; `chora-mc` has no
window drag, and `chora-construct` waits for the 3D step. X6 keeps the
requests of one client for the windows of another, unchanged.

## Open points

- A flavor may need other defaults than the compiled ones, such as the
  XP clock format and `window_manager: xp`. X4 decides how a flavor sets
  them. Until a flavor needs it, there is no mechanism for it.
- The panel needs a way to find which window manager answers `minimize`.
  The setting `window_manager` solves it for stacking window managers.
  `chora-tiling` exports no such methods, so a panel on tiling shows its
  windows but cannot minimize them.
- `chora-xp` as the library host of `chora-stack` makes one plugin depend on
  another in the same repository. X5 decides the surface of the library.
- The fonts of the XP folders are 290 KB. The RPM carries a copy for each
  of the two folders of X2.
- The XP balloon drew an icon in the notification area while it showed, and
  played a sound. A notification block that calls no other block cannot do
  either; the panel (X4) and the sounds (X7) decide whether the core or the
  block that shows the balloon sends an event for them.
- The feedback of a program that starts (`plugins/chora-xp/src/launch.rs`:
  the wait for the first window by its activation token, "Starting NAME",
  "Cannot open NAME") belongs to the panel. It moves with the start menu to
  `chora-panel` in X4. It talks to `notify` only through the bus, with
  `chora_kit::notify`, so it needs no other call between blocks. The panel
  learns the token of a window from `surface.created`.
- The tail of a card of `notify` always points down. At a top corner it
  points away from the corner; a flavor with a tail uses a bottom corner.
- The XP balloon sits about 22 pixels further left than before. Its tail
  tip followed the tray icon; now `margin: [46, 22]` and `tail_inset: 40`
  place it, and the width of the tray, the auto-hide of the taskbar and the
  work area are not taken into account. X4 revisits it when the panel exists.
- `figure: paperclip` in the `values.yaml` of the XP folder selects code,
  `paperclip.rs`, from a neutral block. The key exists only until X7 moves
  the paperclip into the gimmick.
- `chora-draw` still carries XP specifics: the font variants `Title`,
  `Start`, `Name` and `Dialog`, the fixed XP font names, the search of
  `fonts/WindowsXP`, and the faux bold and unsmoothed rendering. The blocks
  use only `Ui` and `UiBold`. The XP-only parts leave the library when the
  gimmick takes them.
- The neutral `margin` of `notify`, `[16, 56]`, leaves room for the dot of
  `chora-assistant` at the bottom. At a top corner the 56 pixels count from
  the top.
- A missing or broken picture of `notify` is reported as "no picture"
  with the name of the file but without the cause.
