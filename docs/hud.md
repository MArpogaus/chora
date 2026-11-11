# HUD folders

A HUD folder defines how the voice agent looks on the screen: the figure
that shows its state, the sentence it says, the list of helpers and the
look of the chat. The host plugin `chora-assistant` draws it. A folder
holds data only. Nothing in it runs code, and a folder cannot change a
word, a key or what a click does.

The format described here is version 1. The code that reads it is the
module `chora-kit::hud`.

## Folder

```
fancy/
  hud.yaml        required
  README.md       what the look is, its licence, the fonts it needs
  dot.svg         pictures, one SVG file per figure
  fonts/          optional
```

A shipped folder lives in `chora/huds/NAME` of a directory of
`$XDG_DATA_DIRS`, by default `/usr/share/chora/huds/NAME`, a user folder
in `$XDG_DATA_HOME/chora/huds/NAME`. A user folder wins over a shipped one
of the same name. chora ships `neutral`, `xp`, `mc` and `construct`. A folder that is dropped in changes nothing until a
setting names it.

### Limits

| Limit | Value |
|---|---|
| `hud.yaml` | 64 KiB |
| panels | 32 |
| one file | 1 MiB |
| all files of the folder | 8 MiB |
| entries (files and folders) | 256 |
| depth of folders | 4 |
| YAML anchors and aliases | none |

A link in the folder is allowed only if it leads to a regular file
inside the folder. A link to another place, to a folder or to a device
is refused, and so is a path with `..` or an absolute path in a key. A
file with more than one hard link is refused. A folder that breaks a
limit is refused as a whole, and `hud.yaml` is read only when the folder
passes these checks.

### Checking a folder

A folder is checked when it loads: the limits above, `hud.yaml` with each
error naming its file and key, and the pictures and fonts as listed under
"Pictures, fonts and colours".

`chora-assistant --check-hud DIR` runs these checks without the host and
adds the checks of the pictures (a link or an `href` that leaves the
folder, an SVG that is too large) and of the colours of `chat:` against the
neutral chat (`Chat::contrast_errors`). It prints one problem on each
line, with its file and key, and exits 1 when there is one. A folder that
is fine prints `OK. Turn it on: chora-assistant --use-hud NAME` and exits
0; when `--use-hud` would not find it, because it lies outside the
folders that `--list-huds` reads, it prints `OK, but` with those folders
and the `cp -r` that puts it in the first one.

### Using a folder

One setting names the folder: `hud: NAME` in
`~/.config/chora/chora-assistant.yaml`, `neutral` by default. The host
looks for `NAME` in `$XDG_DATA_HOME/chora/huds`, then in `chora/huds` of
each directory of `$XDG_DATA_DIRS` (`/usr/local/share` and `/usr/share`
when the variable is empty), then, for a binary run from
`target/<profile>/`, in its own `plugins/chora-assistant/huds` of the
source tree, which links the folders of xp, mc and construct.

| Command | Does |
|---|---|
| `chora-assistant --list-huds` | Lists the folders by name, with their place. |
| `chora-assistant --use-hud NAME` | Checks the folder and writes `hud: NAME`; refuses a folder with problems. `default` takes the line out. |
| `choractl call assistant settings.reload` | Makes a running host take the new setting. A folder that fails keeps the look on the screen, and the host logs the error. |

The host reads a folder only through `Theme::load_hud`. The folder of a
flavor (`--assets`, `CHORA_ASSISTANT_ASSETS`) is the one other way to
name a look, and it replaces the folder of the setting. With a
`hud.yaml` it is a HUD folder and is read as one, as the XP flavor does;
without one it is an asset folder with `values.yaml`, which the host
trusts.

### Folders by context

The setting `huds` maps the context to folders. The first rule whose
`when` matches wins; `hud` draws when none does. A rule without `when`
always matches. The shipped settings have these rules, so each look
shows its own folder and tiling shows `hud`:

```yaml
huds:
  - {when: {space: construct}, folder: construct}
  - {when: {wm: mc}, folder: mc}
  - {when: {wm: xp}, folder: xp}
```

A matching rule overrides `hud`, and so `--use-hud`, which prints a warning
when `huds` has rules.

A rule with a key that is no field of the context, a value that a field
with a closed list does not know, or a `folder` that is no folder name
(see `--list-huds`) is an error that names `huds[N]` and the key; the
setting is refused. A folder that is not found keeps the look on the
screen and the host logs it.

The host reads the context at its start. Then it reads it again 150 ms
after the last change of a burst, at the latest 1 s after its first, so
that a run of focus changes switches the look once; a look shows at least
1 s before the next switch. Another folder loads only when a rule names
it; a change of a field that a panel reads picks the panels again from the
loaded folder, and the figure is built again only if they differ. A folder
that fails to load keeps the look and is not asked for again.
The look then changes for the folder and the panels of the new context,
with the cross-fade of the figure, on the output of the focused window.
The context stays as it is:

* while a question waits. The card never moves and its style never
  changes. The change is evaluated 150 ms after the question ends.
* while the chat field has the keyboard.

A switch never sends `focus` or `focus_node`: the keyboard stays where it
is. Without a focus, the window manager that claims the first output
counts as `wm` and `space`, and `app` is empty. The window manager of the
space (`space`) is the owner of the largest claim at the centre of the work
area of the output; it differs from `wm` when a window manager runs in a pane
of the construct. Panels anchor in the work area of the output, which leaves
out the panels and bars of the window managers.

## hud.yaml

Unknown keys are errors, at every level. Each error names the file and
the key, for example `hud.yaml: chat.size: 11 is not from 14 to 48`.

```yaml
hud: 1
font: sans-serif
figure: paperclip.svg
panels:
  - id: figure
    source: voice
    when: {wm: [xp]}
    anchor: bottom_right
    margin: [16, 48]
    picture: dot.svg#{state}
    action: mute
  - id: line
    source: say
    near: figure
    style: bubble
    text: "{text}"
    max_chars: 160
  - id: agents
    source: agents
    anchor: top_right
    rows: "{name}  {state}  {task}"
    max_rows: 4
    action: chat
    colours: {text: [32, 32, 32], muted: [90, 90, 90], accent: [0, 80, 160]}
chat:
  style: plain
  size: 16
  anchor: right
  width: 400
  margin: 0
```

| Key | Meaning |
|---|---|
| `hud` | The version. Must be `1`; any other number is refused. |
| `font` | The default font family. Default `sans-serif`. |
| `figure` | The picture of the figure, for a folder that needs no place for it: an SVG file of the folder with one element for each state, named `idle`, `asleep`, `listening`, `thinking`, `working`, `speaking` and `question`; a missing one shows `idle`. It shows at the bottom right of the work area, 16 pixels from the edges, and a click on it mutes the mic. A panel with `source: voice` and a `picture` that shows in the context takes its place. Optional. |
| `panels` | A list of panels. |
| `chat` | The look of the chat. Optional. |
| `dark` | The colours of the dark theme; see Themes. Optional. |

### Panels

| Key | Meaning |
|---|---|
| `id` | A name, unique in the folder: 1 to 32 characters of `a-z`, `0-9`, `_` and `-`. |
| `source` | Where the data comes from: `voice`, `say`, `agents`, `chat`, `context`, `clock` or `event`. |
| `event` | `CLIENT.EVENT`, for `source: event` and only there. The host checks the exact client name. |
| `when` | A condition on the context; see below. Optional. |
| `anchor` | One of `top_left`, `top`, `top_right`, `left`, `center`, `right`, `bottom_left`, `bottom`, `bottom_right`, in the work area of the output. |
| `margin` | `[horizontal, vertical]` gap to the edges. |
| `near` | The `id` of another panel to stand beside. Use `anchor` or `near`, one of them. The chain of `near` must end at a panel with an `anchor`; a cycle is an error. |
| `picture` | `FILE#ID`: an SVG file of the folder and an element id in it. Only the id is a template; the file name must not contain `{`. A missing id shows `idle`. |
| `action` | What a click does: `mute`, `cancel`, `sleep`, `wake` or `chat`. Nothing else exists. `mute` only on a panel with `source: voice`, and it only mutes. `chat` on a panel with `source: voice` opens the chat on a click on the figure, which then stands in for the "Chat ..." row. |
| `style` | `bubble` or `plain`; `island` on a panel with `source: voice`. |
| `text` | A template for the text. |
| `max_chars` | The most characters of the text, 1 to 2000. |
| `rows` | A template for one row of a list. |
| `max_rows` | The most rows of a list, 1 to 200. |
| `colours` | The colours of the text, `[r, g, b]` each. For the rows of the helpers: `text` for the name, `accent` for the state word and `muted` for the task; a colour left out is the text colour of the chat. For a sentence: `text`. Each needs a contrast of 4.5:1 against the fill behind it. |
| `fill`, `border` | The colour inside the box of a sentence or of a row, and of its hairline. Left out, a sentence has the light card of the host, or with a `bubble` chat the fill, hairline and text of the chat, and a row the fill of the chat. The text needs 4.5:1 against the fill, the hairline 1.5:1. |
| `tail` | `left` or `right`: the bubble of a sentence gets a tail that points at the figure, near that end; the bubble grows to the other side. Left out: no tail. |

A panel needs at least one of `picture`, `text` or `rows`.

The `text` of a panel with `source: voice` is the state word beside the
figure, toward the middle of the output, on the backing of a plain
sentence: `text: '{state}'` shows `Listening` in the language of the
screen. There `{state}` is the word of the word table, not the name of
the state. The neutral and the mc folders show it, so the state never
depends on a colour alone.

A sentence (`source: say`) shows in a bubble above the figure, or below
a figure at the top. With `style: plain` it has no hairline and no
tail: the text stands beside the figure, on the side of the middle of
the output, in the text colour of the chat on a backing in the fill of
the chat, so that it reads on any window behind it. A sentence that the agent still speaks shows again
when the look is built again.

With `style: island` the voice panel is one glass capsule after the
glass of the construct (`docs/construct-style.md`, rules 18 to 25), 40
pixels high: the picture of the state at the left, 24 by 24, the state
word in capitals, and while the agent thinks or works the task of the
busy helper with its step, `Tidy notes · 1 of 2`, after a separator. A
sentence of `voice.say` takes the place of the word and the task, at
most two lines of 480 pixels with its end kept, and goes 2 seconds after
its sound. The capsule hugs its content from the edge of its anchor; the
figure does not move or blink, and the folder shows no rows of helpers.
In a busy state the ring moves, drawn by the shader `ring.frag` of the
host in its cache: it pulses while the agent listens or a question
waits, and an arc turns while it thinks (1.2 s a turn) or works (2.4 s
a turn). In every other state the picture of the folder stands still,
so a still screen draws nothing.
While the camera of the construct is in a pane, which it tells with its
event `construct.view {pane: true}`, the island dims to 0.6 over 160 ms.

A picture of a figure may hold an element `ID-shut` beside an element
`ID`, for example `idle-shut`: the figure then blinks with it, a
moment every few seconds.

The host draws the rows of the helpers itself: a panel with
`source: agents` and `action: chat` gives their place, `max_rows` (4 if
left out) and `colours`. Below the "Chat ..." row stands one row for each
helper, `Atlas · working · Write file (2/2)`: the name, the state word of
the word table and the task with its step. The rows stack away from the
edge of the anchor; while the `when` of the panel does not match, only
the "Chat ..." row shows, in the same place. A folder whose figure
opens the chat has no "Chat ..." row. A click on a row opens the chat with that helper.
While the chat is open, no row shows: its header names the helper, and
the rows would stand over it. The
words and their order are the same in every folder; `rows` does not
change them. The node of a row is `hud-row-NAME`. A helper whose name
has other characters than letters, digits, `_` and `-`, or that is named
`chat`, has no row.

The data of a source is not computed in a folder. A person who needs
other data writes an ordinary plugin that emits an event, and the folder
shows it with `source: event`.

### Templates

A template has `{field}` and `{field.sub}` and nothing else. A value is
inserted as plain text and never expanded again, so a value that
contains `{state}` shows those characters. A field that does not exist
is empty. There are no expressions, loops, method calls or scripts.

The fields of each source:

| Source | Fields |
|---|---|
| `voice` | `state`, `awake`, `listen`, `muted` |
| `say` | `text`, the sentence of `voice.say` |
| `agents` | `name`, `state`, `task`, once per helper in `rows` |
| `chat` | `name`, once per open chat |
| `context` | `wm`, `space`, `app`, `state`, `agents`, `chat`, `output`, `theme` |
| `clock` | `time` |
| `event` | the fields of the data of the event, as `{field}` and `{field.sub}` |

### When

`when` compares fields of the context with a value or a list of values.
All keys must match. A value of `state`, `agents`, `chat` or `theme` that is not in the list below is an error that names the key, for example `when.chat: "opened" is not one of open, none`. The other fields take any name. The fields:

| Field | Values |
|---|---|
| `wm` | Name of the window manager that owns the focused window: `tiling`, `xp`, `mc`, `construct`. |
| `space` | Name of the client that claims the real output of the focused window; it differs from `wm` when a window manager runs in a construct pane. |
| `app` | The `app_id` of the focused window. The client chooses it, so a rule on `app` is a style choice and no security boundary. |
| `state` | `asleep`, `listening`, `thinking`, `working`, `question`, `speaking`. |
| `agents` | `none`, `any`, `working`, `question`. |
| `chat` | `open`, `none`. |
| `output` | The name of the output. |
| `theme` | `light`, `dark`: the theme of the client of `space`, else of `wm`, else `light`. |

```yaml
when: {wm: [xp, mc], agents: any}
```

### Themes

A window manager names its theme with its event `NAME.theme {theme:
light|dark}`, at its start, when its settings change and on the event
`assistant.ready`, which the host emits once it listens; mc and
construct do. The host listens to `construct.theme`,
`mc.theme`, `tiling.theme` and `xp.theme`, and the context field `theme`
is the theme of the client of `space`, else of `wm`, else `light`.

The block `dark:` gives the colours of the dark theme over those of the
folder: `chat` takes `fill`, `text`, `name`, `border` and `glow`, and
`panels` takes, by the `id` of a panel, `fill`, `border` and `colours`.
A colour left out stays as it is in the light theme. An id that no panel
has is an error, and the dark colours pass the same contrast check.

```yaml
dark:
  chat: {fill: [28, 28, 28], text: [217, 212, 201]}
  panels:
    agents: {colours: {text: [217, 212, 201]}}
```

### Chat

The block `chat:` styles the chat view and its input field. It holds
values only. A folder without the block gets the neutral chat.

| Key | Meaning |
|---|---|
| `style` | `plain`, `bubble` or `glass`; `bubble` is the balloon of the figure, see below. |
| `fill`, `text`, `name`, `border` | Colours as `[r, g, b]`, 0 to 255. A colour left out takes the host's. |
| `font` | Font family. Default: the font of the folder. |
| `size` | Text size in pixels, 14 to 48; a value outside is refused. Default 14. |
| `anchor` | The place. Default `right`. |
| `width` | The width in pixels, 1 to 1600. Default 400. |
| `margin` | The gap between the view and the edges of the work area in pixels, 0 to 128. Default 0. |
| `glow` | The colour of the glow around a `glass` chat, which falls to nothing within 16 pixels. Left out: no glow. |
| `slot` | The name of a client, for example `mc`. The window manager with that name announces a box with its event `NAME.chat.slot {output, rect}`, and the host draws the view inside the box, `[x, y, width, height]` in pixels of the output, four numbers. The host clips the box to the work area and to the widest chat, 1600 pixels; a box outside the work area is no box. Only the client of that name fills the slot; the core gives a name to one client. Without a box, and after `rect: null` or when the client leaves, the view stands at `anchor`. |

A `glass` chat is the glass column of the construct style (rules 18 to
25, the tokens of `chora_kit::glass`, light or dark after the theme): 28
hundredths of the work area wide, from 400 to its `width`, corners of
24, the strong tint and a long shadow. Its header names the state of the
helper beside the ring of the folder's figure and, at the right, the
helper, its task and its step. The person's messages stand at the right
in accent bubbles, the helper's in Inter 16 under its name, its tool
steps in the mono family with a check, the running one with an arc. The
field is a well of 48 with the focus ring while it has the keyboard; the
key line under it names Return (send), Ctrl+. (stop) and Esc (close),
and a click on the words of Stop or Close does the same, without a
button. The island of the folder hides while the chat is open: the
header takes its place. A question card in the chat keeps its place,
and the transcript flows under it. The text of a glass chat is checked
against its `fill` over a black and over a white pane.

The view never covers the figure, its bubble or the "Chat ..." row: a
view that would, moves sideways beside them, also a view in a slot. A
view at `left` or `right` is a side panel and keeps its edge.

The open chat is the one surface of the helpers in every look: a
sentence of the agent shows inside it, under the transcript, not in a
balloon of its own, and a question that it shows has its card inside.

A `bubble` chat of a folder with a figure is the balloon of the figure
itself, as in xp: it stands above the head of the figure, `width` wide
and as high as its content, at most six tenths of the work area, so it
grows with the transcript, inside the work area, with
a tail that ends at the head, and the field inside it. A click on the
balloon of a sentence or on a figure with `action: chat` expands the
balloon into the chat; a click on the figure or Close collapses it, and
it shows the last sentence again. A question opens it, with the card
inside. A figure that opens the chat draws no rows:
the balloon lists the helpers.

A font family that the system does not have, such as TeX Gyre Pagella
of the mc folder on a system without the TeX Gyre fonts, falls back to
the default family of the system; the folder still loads.

## Pictures, fonts and colours

The host loads a folder with `chora_draw::theme::Theme::load_hud`.

- A file is read only inside the folder, through the same checks as the
  loader: no link out, one name, at most 1 MiB. It is opened without
  following a link in its last component, and refused if it is not the
  file that was checked.
- The `href` of an image in an SVG is a relative path inside the folder.
  An absolute path, `..`, `file:` and `http:` draw nothing. Only PNG and
  JPEG load as images: an SVG in an image, by path or in a `data:` URI,
  draws nothing, so that pictures do not nest.
- An SVG file is refused if it is over 2048 px on a side, has more than
  10,000 nodes (copies made by `<use>` count), has more than 16 images
  and filters, or has a pattern tile over 2048 px on a side. A parsed
  file is kept, not parsed again for each draw.
- Fonts load from `fonts/` of the folder into a font database that only
  the text of the folder uses. The first 256 entries are listed, at most
  16 files load, and a `fonts` that is a link loads nothing. The question
  card uses the host fonts (`Fonts::host`) and never a font of a folder.
- The host loads no folder with a colour below these contrasts, however
  the folder is named: by `hud`, by a rule of `huds` or by `--assets`.
- Text colours need a contrast of 4.5:1 against their fill
  (`chora_kit::hud::contrast`, `Hud::contrast_errors`): the `text` and
  `name` of the chat and the `colours` of a panel. The key line and
  the buttons of the question card are the host's and keep 7:1.

## What a folder cannot do

The host draws the question card, the muted badge and the words itself.
A folder styles them and cannot hide, cut, relabel or move them. The
card is always on top of every panel and of the chat view; neither a
folder nor a client picks its place. A
folder without a visible panel with `source: voice` gets the neutral
dot. Words come from one table in English and German; they follow the
`language` setting.

### The question card

`voice.confirm` shows as a card above the figure, once, with the title, the
prompt, the detail, and, for a question that only a key or a click may
answer, the key line of the event and the note that a spoken yes does not
count; the buttons Yes and No follow when the event has `buttons`. The
card appears in one place and stays there until the question ends. It
is drawn with the host fonts at 14 px, dark text on a light fill. A press
on a button answers with `confirm.answer`, except an injected press, which
shows "Please use your own mouse or keyboard." on the card, and a click in
the first 400 ms after the card appeared, which does nothing. The card is
at most 560 px wide and leaves 64 px at the sides of the output. The key
line and the note are always there. When the whole text, with the whole
detail, does not fit in 80 percent of the height of the output, the card
shows no buttons and says "Too long to show. Refused.", and the host
answers `confirm.answer {yes: false}` at once. The host removes control,
bidi and zero-width characters from the prompt and the detail.

A folder with a `glass` chat gets the glass card of the construct style:
400 px wide, the title as a label, the prompt at 16 px, the detail in a
well at 14 px in the mono family, the note, and Yes and No as key
segments that hold the key line, `Alt Y  Yes` and `Alt N  No`; its
hairline and glow are in the accent while it waits. It stands under the
figure, and in an open chat 48 px below the top of the view. A question
that names its helper (`name`) has the title "Question from NAME".

### The mic

A panel with `source: voice` and `action: mute` mutes the mic on a
click and never unmutes it. While the mic is muted, the host draws a
badge, "Microphone off. Click to turn it on.", beside the figure, above
the card; a click on it that a person made turns the mic on. An injected
click on the badge or the figure does not; it shows the line "Please use your own mouse or
keyboard.". A folder whose voice panel has no `mute`
action has a figure that takes no click, and the badge is still there.
