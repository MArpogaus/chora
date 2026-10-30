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
├── clients.yaml        # the names and capabilities of `clients up`
└── chora-tiling.yaml   # the options and bindings of chora-tiling
```

The directory is a git repository. It contains no secrets. The package
installs the reference flavors to `/usr/share/chora/`.

Fonts are not there: the core draws text with the system fonts and with
every directory listed in `CHORA_FONT_DIRS`, separated by `:`. A user
font in `~/.local/share/fonts` needs no setting at all.

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
- A node has an `id`, a `kind` (`group`, `rect`, `image`, `shader` or
  `text`), a `parent`
  (the root by default) and its `props` as in a `create` command.
- A name has lowercase letters, digits, `_`, `-` and `.`, and starts
  with a letter, so that its file stays in its directory. A name does
  not end in `.yaml`, because such a name is a path. The file
  `NAME.yaml` must name the same view.
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

### Clients file

`choractl clients up [FILE]` applies a file that maps client names to
their grants (see [API](api.md#client-sockets)):

<!-- parses: clients -->
```yaml
# ~/.config/chora/clients.yaml
agent:
  grants: [screen.capture]
showroom:
  grants: []
```

The grants are the six capabilities of `choractl grant` (see
[API](api.md#client-sockets)). An agent that
drives the window manager needs `methods.call`, and one that starts apps
needs `activation.token`. Both let it take the keyboard focus. A name
that may end chora needs `session.quit`.

The names follow the rules of `hello`. `clients up` opens the socket of
every name with exactly the capabilities of its list. A socket that
already has the equal capabilities stays with its connections and gets
them back after the revoke binding. Changed capabilities recreate the
socket and disconnect its Wayland clients. A name with a client socket
takes no `choractl grant` and no `choractl revoke`; the file decides.
While a name is connected, its socket cannot open. `clients up` then
only syncs the grants of the name and says on stderr that the socket
stays closed.

## chora-tiling

`chora-tiling` connects with the client name `tiling`. Grants and
`choractl restart` use this name.

| Flag | Meaning |
|------|---------|
| `--config FILE` | the settings file instead of `$XDG_CONFIG_HOME/chora/chora-tiling.yaml` |
| `--print-config` | prints the default settings and exits |
| `--output NAME` | manages only this output; can come more than once (see below) |
| `--ignore APP_ID` | leaves the windows with this app id to other clients; can come more than once |

`chora-tiling` reads `chora/chora-tiling.yaml` in `$XDG_CONFIG_HOME`, by
default `~/.config`, or the file of `--config`. Without a file, it uses its
default, which `chora-tiling --print-config` prints. The file holds the
settings of D62: `settings.set` changes them at once and writes the file
again, without its comments (see [API](api.md#plugin-methods)). Edits by
hand apply with `settings.reload` or `MOD+Ctrl+r`, or at the next start
of the plugin. The
default file lists every option. A file can leave options out, and they
take their defaults; a map such as `bindings` in the file replaces the
default map:

| Option | Meaning |
|--------|---------|
| `modifier` | what `MOD` means in the bindings: Super or Alt |
| `gap` | the gap around each window, in logical pixels |
| `tags` | the number of tags of each output, at most 16 |
| `layouts` | the layouts in the order of `next_layout`; each tag starts with the first |
| `master_width`, `masters`, `columns` | the start values of each tag for the tile layouts |
| `new_window` | `master` or `stack`: where a new window goes |
| `focus` | `click` or `follow_mouse` |
| `style` | how a new window looks: `opacity` from 0 to 1, and `effects`, a list of shaders as in `scene.apply` (#99); `chora-tiling` sets it once, when it takes the window; a look that another client asks for wins (see below) |
| `bindings` | a binding with `MOD`, and its action; the default file uses every action at least once; `run: CMD` starts a shell command with `choractl run` |

The layouts and the default bindings follow the awesome window manager.
Each output has its own tags, and each tag its own layout and parameters.
A window can be on several tags, and an output can show several tags. A
floating, maximized or fullscreen window leaves the tiles. A click or a
binding raises the window that it focuses; the focus that follows the
pointer raises nothing. The binding
`MOD+Ctrl+r` reads the file again. Mouse moves and resizes of floating
windows do not exist.

`chora-tiling` exports methods for other clients (see
[Plugin methods](api.md#plugin-methods)). Every method but `get_layout`
changes something and answers with the layout info of the current tag,
like `set_layout`. The `gap` of `set_layout` is a live value: the file
keeps the old gap, and the next `settings.set` or `settings.reload` sets
the gap of the file again.

| Method | Parameters | Does the same as |
|--------|------------|------------------|
| `set_layout`, `get_layout` | see [API](api.md#plugin-methods) | the layout of the current tag |
| `give` | `to`: a client name | handing the focused window to another client |
| `view_tag` | `tag`: a tag number | `MOD+1` to `MOD+9` |
| `move_to_tag` | `surface`, `tag` | `MOD+Shift+1` to `MOD+Shift+9`, for this window |
| `focus` | `surface` | a click: focuses and raises the window, and its output; a window that shows nowhere only gets the urgent mark |
| `focus_next`, `focus_prev` | none | `MOD+j` and `MOD+k` |
| `swap_master` | `surface` | `MOD+Ctrl+Return`, for this window |
| `settings.get`, `settings.set`, `settings.reload` | see [API](api.md#plugin-methods) | an edit of the file and `MOD+Ctrl+r` |

`chora-tiling` connects with `resume` (see [API](api.md#ipc-methods)). It
keeps the tags and the layouts of each output, and the tags, the states
and the stacking of each window, in the `meta` of its nodes. A restart
within the grace time rebuilds them from there. In a flavor or a view,
`choractl restart tiling` does that after a rebuild.

Another client changes the opacity, the effects or the meta of a tiling
window with an `update` of the window (D58). The core asks
`chora-tiling` with `node.requested`, and `chora-tiling` applies it,
except the meta key `window`, which holds its own state. A layout or a
tag never changes the opacity or the effects, so the requested look
stays across relayouts, layout changes, tag moves and restarts with
`resume`. The `style` applies only when `chora-tiling` takes a new
window, so a requested value wins over the style for that window.

`chora-tiling` claims the box around the work areas of its outputs
(D55). `--output NAME` limits it to some outputs, as for `chora-xp`, and
can come more than once. So each window manager takes its own outputs.
It leaves a new window that the core reserves for another client to that
client. When the last of its `--output` outputs goes, it gives its
windows away, and another window manager adopts them. It takes new
windows again when one of its outputs comes back. Next to `chora-xp`,
tiling needs `modifier: Super`: with `Alt`, tiling grabs Alt+Space for
`next_layout`, and the system menu of XP does not open.

## chora-mc (experimental)

!!! warning "Experimental"
    `chora-mc` is an experiment. It is not in the packages, and its
    methods and files change without notice.

`chora-mc` connects with the client name `mc`. It is a voice-first
desktop without window management: the whole screen shows one scene at a
time on the first output, and quiet regions around it show the context:
the running head and the clock, what the voice agent does, notes and a
docked terminal. The voice agent and small collector scripts fill the
scenes through methods; the plugin reads no sources itself.

A scene of `chora-mc` is not a view of D35. It is a YAML file
`NAME.yaml` in `$XDG_CONFIG_HOME/chora/mc/scenes/`: a grid of panels, each
with a title and lines, or one app whose window fills the screen without
a frame. The plugin has the built-in scenes `agenda`, `agents`, `mission`
(Mission Control), `morning` (the morning paper) and `terminal`. A file of
the same name replaces a built-in scene.

```yaml
# ~/.config/chora/mc/scenes/day.yaml
grid: [2, 2]          # columns and rows
chapter:              # all optional: the place, the name, the date
  number: "2"
  title: Day
  running_head: Week 44
panels:
- id: mail
  cell: [0, 0, 1, 1]  # column, row, width and height in cells
  title: Mail
  style: {base: body, color: accent}   # a style of the settings, changed
- id: agents
  cell: [1, 0, 1, 2]
  title: Agents
```

Each scene of panels has a head in the style of the LaTeX package
classicthesis: the running head in the header (see the regions below),
and the number and the title of the scene as `chapter.layout` sets. The panels
share the text block, one column of `page.measure` per column of the
grid. A line `TIME<tab>TEXT` puts the time in the margin column left of
the text, flush right, and the text wraps under it. A panel names the
styles of its lines, its title and its notes with `style`, `title_style`
and `note_style`; each takes a `base` (by default `body`, `heading` and
`margin`) and changes single fields.

A scene may set its own `page` (`measure`, `margin` and `gap`, as in the
settings), for example narrow columns side by side on a wide screen, and
`chapter.layout`, also `none` for a scene without a head. Columns of a
grid leave room for the margin of the column to their right. A panel may
set a `rule`, a color of the theme for a hairline left of it, an
`opacity` from 0 to 1 and a `blur` radius in pixels up to 16, for
context that steps back (`plugins/chora-mc/blur.frag`).

| Method | Parameters |
|--------|------------|
| `scene.show` | `name`; the scene fades in, the old one fades out |
| `scene.list` | none; answers `scenes` and `current` |
| `scene.save` | `name`, `grid`, `panels` or `app`; writes the file |
| `panel.set` | `scene`, `panel`, `lines` and an optional `title`; until the next start. A line with a tab puts the part before the first tab in the margin |
| `hud.set` | `state`, for example `listening`, `thinking` or `working`, and `text`; both empty hide the line |
| `hud.say` | `text`, `duration_ms` and an optional `state`: the text shows word by word over the duration, the time that the voice takes to speak it |
| `app.open` | `command`: a shell command that starts in a scene of its own |
| `region.list` | none; answers the current `scene`, each region with `shown`, `visible` and its `contents`, and each content with its `kind`, `title`, `text` and whether its app runs |
| `region.toggle` | `region` and an optional `shown`; without it the region flips. Answers `shown` |
| `region.put` | `content`, `region` and an optional `slot`: the content goes there over the rules, until the next start. Without `region` the content follows the rules again |
| `content.set` | `content`, `lines` and an optional `title` of a text content |
| `content.add` | `content` and `line`: adds a line to a text content, such as a note; the oldest go after 12 lines |
| `region.solo` | `content`, `region` and an optional `shown`: the content shows alone in the region over any scene, such as the test overlay in the drawer; without `shown` it flips. Hidden again, the region shows as before. Answers `shown` |
| `agent.set` | `name` and optional `state`, `task`, `said` and `voice`: adds an agent or changes it; a missing field stays. The `state` is a name of `states` |
| `agent.remove` | `name` |
| `agent.list` | none; answers `agents`, each with `name`, `state`, `task`, `said`, `voice` and `since` (the time of its first report) |
| `notify.push` | `title`, an optional `body` and an optional `urgency` (`low`, `normal` or `critical`): the line `HH:MM  title: body` goes on top of the text `notify`; a critical one also shows in red in the text `urgent` until the next one |
| `notify.clear` | none; empties `notify` and `urgent` |

New text writes itself, as Write of manim: a soft front draws the
outlines line by line, and the fill follows. This happens when a scene
shows and when `panel.set` changes the text of a panel; the same text
again changes nothing. The effect is `plugins/chora-mc/write.frag`, and
the `animate` of the core moves its uniform `progress` from 0 to 1.

A window that no scene started, for example from a launcher, gets a scene
of its own. The settings of D62 are in `chora/chora-mc.yaml`; edits by
hand apply with `settings.reload`:

- `theme` (`light` or `dark`) and the colors of each theme by name, at
  least a `background`.
- `scale`: the font sizes as steps of a modular scale (`base`, `ratio`),
  and the `baseline` grid; each line of text is whole lines of the grid.
- `styles`: named styles with `base`, `family`, `weight`, `style`
  (`normal` or `italic`), `size` or `step`, `color` (a name of the theme
  or RGBA), `line_height`, `letter_spacing` in em, OpenType `features` and
  `uppercase`. The plugin uses `chapter_number`, `chapter`,
  `chapter_large`, `running_head`, `heading`, `body`, `margin` and `hud`.
  The default sets TeX Gyre Pagella with small caps and old-style figures
  for the text, from the Fedora package `texlive-tex-gyre`, whose
  directory `/usr/share/texlive/texmf-dist/fonts/opentype/public/tex-gyre`
  must be in `CHORA_FONT_DIRS`, and Inter with tabular figures for the
  running head, the notes and the HUD.
- `chapter.layout`: `number` (a large light number, the title in spaced
  small caps below), `caps` (the title only), `large` (one large light
  title) or `none`.
- `page`: the `measure` of a column of text, the `margin` column and the
  `gap`, in em of `scale.base`.
- `write`: `enabled` (default `true`) and `duration_ms` (default 900).
- `default_scene`, and the `bindings` with the actions `next_scene`,
  `previous_scene`, `scene: NAME`, `run: COMMAND`, `toggle: REGION` and
  `solo: {content: NAME, region: REGION}` (as `region.solo`; the default
  `Super+F12` shows the test overlay in the drawer).
- `states`: the states of `agent.set`, each with a `sign`, a `color` of
  the theme and an optional `label` for the agents scene. The default has
  `idle` ○, `listening` ◉, `thinking` ◌, `working` ●, `speaking` ◐,
  `question` ? (has a question), `blocked` ✕, `done` ✓ and `asleep` ☾:
  blue hears or thinks, green works or is done, the accent speaks, amber
  needs you, red is blocked, faint rests. The themes have the colors
  `green`, `amber`, `blue` and `red` for them.
- `regions`, `contents`, `rules`, `preset` and `presets`: see below.

### Regions

The regions around the page follow the Emacs package
[auto-side-windows](https://github.com/MArpogaus/auto-side-windows): rules
put each content into a region and a slot, and a key shows or hides a
region. The page itself never moves when a region shows.

```text
 ______________________________________________________
|  header:  1  AGENDA       LISTENING   FRIDAY 16:08   |
|        |                                  |          |
|  left  |  margin   the page of the scene  |  right:  |
|        |                                  |  NOTES   |
|      __|__________________________________|__        |
|     |  drawer: a docked terminal (foot)     |       |
|     |_______________________________________|       |
|  footer:  words of the agent          15:15 Call    |
|______________________________________________________|
```

- `header` and `footer` are lines over and under the text block, each
  `size` lines of the baseline grid high. Their contents stand side by
  side: a slot below 0 goes left, 0 in the middle, above 0 right. The
  right end keeps its place, the left end comes next, and a text that
  does not fit ends in an ellipsis.
- `left` and `right` are columns beside the page, `size` em wide, their
  contents one under the other with a title in the style `running_head`.
  The page leaves room for a side that has contents, so that its text
  never moves; where both sides would leave the text less than three
  quarters of its measure, the sides do not show.
- `drawer` is a sheet of `size` lines over the bottom of the page, with a
  hairline above it, for a docked app.

A region shows only with something in it. Whether it shows comes from a
toggle, else from the `regions` of the scene file (for example
`regions: {right: false}`), else from the preset, else from `shown` in
`regions`. Over an app, the left and the right side show only after a
toggle, and the app window leaves the header, the footer, the drawer and
a shown side free. A toggle stays for all scenes.

The `contents` are named. Their `kind` is one of:

| Kind | Shows |
|------|-------|
| `head` | the running head of the scene: `chapter.running_head`, else its number and title; the name of an app |
| `clock` | the time as `date` formats it with `format`, each minute anew |
| `state` | the `state` of `hud.set` and `hud.say`, in the accent color |
| `hud` | the `text` of `hud.set` and `hud.say` |
| `agents` | the sign and the name of each agent of `agent.set`, each in the color of its state, in the style `hud` |
| `text` | a `title` and `lines` that `content.set` and `content.add` change |
| `app` | a docked `command`; `{NAME}` in it is the color NAME of the theme as RRGGBB |

Each content may set a `style`. The `rules` are a list, and the first
rule that matches a content wins. A rule names a `content` or a `kind`,
optionally the `scenes` by name or `@app` and `@page`, and the `region`
and `slot`. A rule cannot put an app into a line.

`toggle: REGION` as a binding shows or hides a region. A region with a
docked app works as a quake terminal: the first press starts the app and
gives it the keyboard, a press while the app has no keyboard gives it
back to it, and the next press hides the region and gives the keyboard
back to the scene. When the app ends, its region hides. A drawer that a
preset or a scene shows starts its app when the settings load.

The default bindings are `Super+Return` for the drawer, `Super+n` for the
right side, and `Super+Up` and `Super+Down` for the header and the
footer.

The default `desk` setup: the header shows the running head left, and
the agents, the state of the agent and the clock right; the footer shows
the words of the agent left, a critical notification in red in the
middle and the next entry of the agenda right (the collector fills the
content `next`); the right side shows the notes, which an agent fills
with `content.add`, and the notifications below them, newest on top;
`Super+Return` opens `foot` in the drawer in the colors of the theme.
Over an app, the words of the agent move into the middle of the header.
On a narrow page the header has no room for many agents and the long
clock: the end of the line is cut. Set the rule of `time` instead of
`clock` there.

`preset` picks an arrangement of `presets`. A preset sets which regions
show in `shown` and has its own `rules`; without rules, those of the
settings apply:

| Preset | Arrangement |
|--------|-------------|
| `quiet` | only the header: the running head, the words and the state of the agent, the time |
| `desk` | the default described above |
| `studio` | an outline and the notes in the left side, a terminal in the drawer that starts at once |
| `book` | the running head and the date above, the words of the agent, the time as a folio and the state below, the notes in the margin |

```yaml
# ~/.config/chora/chora-mc.yaml
preset: desk              # desk has no rules of its own
contents:
  todo: {kind: text, title: Today}
rules:                    # the list replaces the default rules
- {content: head, region: header, slot: -1}
- {content: clock, region: header, slot: 1}
- {content: hud, region: footer, slot: -1}
- {content: todo, scenes: ['@page'], region: left}
- {content: notes, region: right}
- {kind: app, region: drawer}
```

```sh
choractl call mc content.add '{"content": "notes", "line": "Call Lena"}'
choractl call mc region.toggle '{"region": "drawer"}'
choractl call mc region.put '{"content": "notes", "region": "left"}'
```

`chora-mc --print-config` prints the default. An option that the file
leaves out takes its default. The collector
`plugins/chora-mc/collectors/agenda.sh` puts the org agenda of today into
the scene `agenda`, with `emacs --batch`: the entries with a time, and the
tasks scheduled or due today, each as a time in the margin and a text.
It leaves out overdue, upcoming and done tasks. `ORG_TODO` sets the TODO
keywords, by default `PROJ TODO NEXT WAIT | DONE KILL`.

The other collectors in `plugins/chora-mc/collectors/` are Python 3
scripts without dependencies. They only read their sources, and a source
that is missing leaves a short note or nothing. Each takes `--print` to
show the JSON without chora:

| Collector | Fills | Sources |
|-----------|-------|---------|
| `morning.py [ORG_FILE...]` | the scene `morning`: the lead (the first task due today, else the next entry), the weather, the day, the mail, overnight and the reading list | `agenda.sh`; [wttr.in](https://wttr.in) for the place `MC_WEATHER` (by IP without it), cached for an hour in `$XDG_CACHE_HOME/chora-mc`, nothing when offline; `notmuch` (unread mail in the inbox), a hint without it; the agents of `agent.list` that are `done`, nothing without them; the reading list `MC_READING` (default `~/Notes/reading.md`): items of markdown or org, the done ones left out |
| `system.py [--every S] [--once]` | the scene `mission`, every 5 seconds: load with a cpu sparkline of the last 24 rounds, the busy processes, the errors of the journal today, the failed units, host facts | `/proc/stat`, `/proc/meminfo`, `/proc/net/dev`, `/sys/class/thermal`, `ps`; once a minute `journalctl -p err -b --since today -o json`, `systemctl --failed` and `systemctl --user --failed` |
| `tests.py [--every S]` | the text `overlay`, the test sheet of `Super+F12` | `gh run list` in `MC_REPO` (the repository of the script by default); gate results in `MC_GATES` (default `MC_REPO/target/gates`), one file per gate whose last `ok:`, `SKIP:` or `FAIL:` line counts; `choractl query '*'` for the nodes by owner; the memory of the chora processes |

Run `morning.py` from a timer in the morning, and `system.py` while the
session runs, for example as a user unit. The agents scene needs no
collector: `agent.set` fills the panel `agent-N` with the N-th agent (its
title in the color of its state, the sign and the state, its task and
its last words) and `band-N` below it with its voice and its start; a
scene file with more columns shows more agents.

```sh
choractl call mc agent.set '{"name": "coder", "state": "working", "task": "Runs the tests"}'
choractl call mc notify.push '{"title": "CI", "body": "dev failed", "urgency": "critical"}'
choractl call mc region.solo '{"content": "overlay", "region": "drawer"}'
```

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
├── chora-tiling.service             # chora-tiling
└── chora-tiling-xwayland.service    # xwayland-satellite, if installed
```

The target also wants `waybar.service`, which the waybar package installs.
A copy of the directory in `~/.config/chora/flavors/tiling/` replaces it.

### The XP flavor

The XP flavor is the showroom (D39): a stacking desktop in
the style of Windows XP. It is a demo, not a daily driver. `chora-xp`
connects with the client name `xp`.

```text
/usr/share/chora/flavors/xp/
├── chora-xp.target              # wants the two units below
├── chora-xp.service             # chora-xp
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

The Luna GTK theme and the icons of `xfce-winxp-tc` give GTK
applications, such as the file manager, an XP look. They hold
non-free files of Windows XP: the user installs them, and chora does
not ship them.

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
Maximize for a maximized one. Move and Size are always gray: `chora-xp`
moves and sizes windows with the mouse, and does not have XP's keyboard
commands for them. The click on the icon opens the menu below the icon, the others at
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

`chora-xp` draws no programs into the scene except the dialogs of the XP
shell (D63): Log Off, Turn Off, Display Properties, Taskbar and Start
Menu Properties, Date and Time Properties and the Control Panel menu.
Folders open in the file manager of the setting `file_manager`, `thunar`
by default: My Documents, My Pictures and My Music of the start menu
(`XDG_DOCUMENTS_DIR`, `XDG_PICTURES_DIR` and `XDG_MUSIC_DIR`, else the
home directory and its folders `Pictures` and `Music`), and My Computer
as `/`. `chora-xp` adds the folder to the command, for example
`thunar '/home/user/Music'`.

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
desktop takes the selection away. A double click opens the icon. A
folder and My Documents open in the file manager, My Computer opens `/`
and the Recycle Bin opens `$XDG_DATA_HOME/Trash/files` in it. A desktop
file runs its program, and any other file opens with `xdg-open`. A drag
moves an icon to the nearest free cell of the grid, until `chora-xp` ends.
A drag on the empty desktop draws XP's selection rectangle, in XP's
selection blue with a translucent fill and a line of 1 pixel, and selects
every icon whose picture or first line of its label it touches, while it
grows and shrinks. A drag of one of the selected icons moves all of them,
each to the cell at the same offset; if one of them would land on an icon
that stays, or two in one cell, they all go back. A click on one of the
selected icons selects it alone, and a click on the empty desktop selects
nothing. Icons dropped on a window or on the Recycle Bin go back to
their places: `chora-xp` changes no file.
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
  the selection stays. The rows act on the icon under the pointer. Open,
  in bold, does what a double click does. The other rows are gray,
  among them Delete and Empty Recycle Bin: the file manager does that.
- **A program or a place of the start menu, or a program of All
  Programs**: Open in bold, and Remove from This List for the programs
  of the left column except Terminal, which removes the program from the setting
  `programs`. Run as, Pin to Start menu, Send To and Rename are gray.

The Control Panel opens as a menu, as with XP's "Display as a menu". A
check mark shows the current choice of each entry, and a click on an
entry applies it and closes the menu. Desktop and Effects change the
settings, as `settings.set` does:

- **Date and Time**: opens Date and Time Properties;
- **Desktop**: the wallpaper of every output, from the PNG, JPEG and SVG
  files in `wallpapers/WindowsXP` of the XDG data directories,
  `theme/wallpaper/` and `/usr/share/backgrounds`, without their
  subdirectories and at most 20 of each directory;
- **Display**: opens Display Properties;
- **Effects**: wobbly windows, the magic lamp, the desktop cube and XP's
  sounds, on or off;
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
keeps it open. OK and Apply change the setting `taskbar`, as
`settings.set` does. The dialog stays above the windows, and the
keyboard does not work it (D45):

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
and OK change the setting `wallpaper`, as `settings.set` does, and set
the scale of the output of the dialog with `output.configure` of the
core. Cancel and the close button close it without a change:

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
  the desktop cube and the sounds, as Control Panel > Effects; OK changes
  the setting `effects`.
- **Settings** shows the monitor with the desktop and a taskbar, and the
  name of the output. The slider of Screen resolution chooses the scale
  of the output, 200 %, 150 %, 125 % or 100 % from Less to More, and
  names the size that it gives, such as "1280 by 800 pixels". Color
  quality, Troubleshoot and Advanced are gray.

Log Off and Turn Off Computer open XP's dialogs in the middle of the
output, and the desktop behind them fades to gray in 1.5 seconds. Each
button runs its command of the setting `session` (D63):

| Button | Default command | Without a command (`null`) |
|--------|-----------------|----------------------------|
| Turn Off | `systemctl poweroff` | ends `chora-xp` and shuts nothing down |
| Restart | `systemctl reboot` | runs `chora-xp` again, as the method `restart` does |
| Stand By | `systemctl suspend` | disabled |
| Log Off | `choractl quit`, which ends chora through the core method `session.quit` | asks every window to close and ends `chora-xp` |

Stand By is also disabled when the first word of its command is not on
the `PATH`. A command such as `loginctl terminate-session
$XDG_SESSION_ID` ends a login session. Switch User is always disabled,
because chora has no second session to switch to. Cancel or a click
outside the dialog closes it. The buttons light up under the pointer; the
keyboard does not work the dialogs (D45).

The method `restart` runs `chora-xp` again in the same process, with the
same arguments, and so reads the settings file again. The core keeps
the nodes of `chora-xp` for the grace time of `resume` (see
[API](api.md#ipc-methods)). The windows keep their places, desktops,
maximized state and stacking, and the opacity and effects that other
clients asked for. The current desktop stays, and its top
window gets the focus.

Another client changes the opacity, the effects or the meta of an XP
window with an `update` of the surface or of its group `xp-win-ID`
(D58). `chora-xp` puts the opacity and the effects on the group, so the
frame looks the same as the window, and the meta on the node that the
client named, except the key `xp`. The meta of a surface does not stay
over a restart.

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

The safety layer keeps only Ctrl+Alt with BackSpace, Escape, i and F1 to
F12, so the cube takes Ctrl+Alt with the arrows. Ctrl+Alt+i is the revoke
binding of [Safety layer](architecture.md#safety-layer). When another client holds
these bindings, Super+Ctrl with the arrows stands in.

`chora-xp` exports methods for other clients (see
[Plugin methods](api.md#plugin-methods)). Each one answers with nothing.

| Method | Parameters | Does the same as |
|--------|------------|------------------|
| `start_menu` | `open`: whether the menu should be open afterwards | a click on the start button of the first output |
| `minimize` | `surface` | a click on the task button of the active window |
| `restore`, `activate` | `surface` | a click on the task button: the window comes back, is raised and focused, and the cube turns to its desktop |
| `restart` | none | Restart in the Turn Off dialog without a command: `chora-xp` runs again and reads its settings file |
| `turn_to` | `desktop`: a number from 0 to 3 | a click on the desktop button |
| `settings.get`, `settings.set`, `settings.reload` | see [API](api.md#plugin-methods) | the dialogs and an edit of the file with `restart` |

| Option | Default | Meaning |
|--------|---------|---------|
| `--theme DIR` | `/usr/share/chora/flavors/xp/theme`, or `CHORA_XP_THEME` | the theme directory |
| `--config FILE` | `chora/chora-xp.yaml` in `$XDG_CONFIG_HOME` | the settings file |
| `--slow-motion FACTOR` | `1` | stretch the magic lamp and the cube, to show them slowly |
| `--output NAME` | all outputs | manage only this output; repeat for more |

`chora-xp` reads `chora/chora-xp.yaml` in `$XDG_CONFIG_HOME`, by default
`~/.config`, or the file of `--config`. Without a file, it uses its
default, `plugins/chora-xp/xp.yaml` of the source tree. The file holds
the settings of D62: `settings.set` and the dialogs change them at once
and write the file again, without its comments. Edits by hand apply with
the method `settings.reload` or `restart`. The default file lists every option; an option
that the file leaves out takes its default:

| Option | Default | Meaning |
|--------|---------|---------|
| `terminal` | `foot` | the terminal of Super+e and of Terminal at the top of the start menu; `CMD -e` runs the programs with `Terminal=true` |
| `launcher` | `fuzzel` | Run in the start menu, and All Programs when no desktop file lists a program |
| `file_manager` | `thunar` | the file manager of the places of the start menu and of the folder icons; `chora-xp` adds the folder to the command |
| `programs` | none | the programs at the top left of the start menu below Terminal, each with `title`, `command` and `icon`, the name of an icon of the icon theme |
| `clock_format` | `%-I:%M %p` | the clock of the taskbar, in the format of strftime |
| `wallpaper` | Bliss if installed, else Bløff | `picture`: `default`, `none` or `{file: PATH}` with an absolute path of a PNG, JPEG or SVG file; `position`: `center`, `tile`, `stretch` or `fill`; `color`: red, green and blue of the desktop |
| `taskbar` | locked, on top, with the clock | `locked`, `auto_hide`, `on_top` and `clock`, as Taskbar and Start Menu Properties |
| `effects` | all on | `wobbly`, `lamp`, `cube` and `sounds`, as Control Panel > Effects |
| `session` | see above | the commands of `turn_off`, `restart`, `stand_by` and `log_off`; `null` for none |

`settings.set` merges its object into the settings (see
[API](api.md#plugin-methods)), so a change of the wallpaper names only
what changes:

```bash
choractl call xp settings.set \
  '{"wallpaper": {"picture": {"file": "/usr/share/backgrounds/a.png"}}}'
```

To try the flavor from a source tree, start it nested:

```bash
cargo build --workspace
PATH=$PWD/target/debug:$PATH chora -- chora-xp --theme flavors/xp/theme
```

## Services

A service runs as a Quadlet unit with Podman, or in distrobox. It
connects to the IPC socket, appears as a Wayland client, or both. A
container mounts only its client directory, which `choractl clients
create NAME` makes (in Quadlet: `%t/chora/clients/NAME`). It sets
`CHORA_SOCKET` to the `ipc.sock` and `WAYLAND_DISPLAY` to the
`wayland.sock` in that directory. The directory of the core holds the
admin socket, so a container never mounts it (see
[API](api.md#client-sockets)). Through its `ipc.sock`, the container sees
and changes only its own nodes and the windows of its `wayland.sock`. It
sets no camera and puts no node under `screen:*`. It takes the keyboard
with `focus_node` or `focus` only after a press of the user on it.
