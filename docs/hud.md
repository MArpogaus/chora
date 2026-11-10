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

A shipped folder lives in `/usr/share/chora/huds/NAME`, a user folder in
`$XDG_DATA_HOME/chora/huds/NAME`. A user folder wins over a shipped one
of the same name. A folder that is dropped in changes nothing until a
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
0.

### Using a folder

One setting names the folder: `hud: NAME` in
`~/.config/chora/chora-assistant.yaml`, `neutral` by default. The host
looks for `NAME` in `$XDG_DATA_HOME/chora/huds`, then in
`/usr/share/chora/huds`, then in the source tree of a binary run from
`target/<profile>/`.

| Command | Does |
|---|---|
| `chora-assistant --list-huds` | Lists the folders by name, with their place. |
| `chora-assistant --use-hud NAME` | Checks the folder and writes `hud: NAME`; refuses a folder with problems. `default` takes the line out. |
| `choractl call assistant settings.reload` | Makes a running host take the new setting. A folder that fails keeps the look on the screen, and the host logs the error. |

The host reads a folder only through `Theme::load_hud`. The asset
folder of a flavor (`--assets`, `CHORA_ASSISTANT_ASSETS`) is the one
other way to name a look; it is trusted and replaces the folder of the
setting.

### Folders by context

The setting `huds` maps the context to folders. The first rule whose
`when` matches wins; `hud` draws when none does. A rule without `when`
always matches.

```yaml
huds:
  - {when: {space: construct}, folder: construct}
  - {when: {wm: mc}, folder: mc}
  - {when: {wm: xp}, folder: xp}
```

A rule with a key that is no field of the context, a value that a field
with a closed list does not know, or a `folder` that is no folder name
(see `--list-huds`) is an error that names `huds[N]` and the key; the
setting is refused. A folder that is not found keeps the look on the
screen and the host logs it.

The host keeps one context and reads it again 150 ms after the first
change of a burst, so that a run of focus changes switches the look once.
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
chat:
  style: plain
  size: 16
  anchor: right
  width: 400
```

| Key | Meaning |
|---|---|
| `hud` | The version. Must be `1`; any other number is refused. |
| `font` | The default font family. Default `sans-serif`. |
| `panels` | A list of panels. |
| `chat` | The look of the chat. Optional. |

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
| `action` | What a click does: `mute`, `cancel`, `sleep`, `wake` or `chat`. Nothing else exists. `mute` only on a panel with `source: voice`, and it only mutes. |
| `style` | `bubble` or `plain`. |
| `text` | A template for the text. |
| `max_chars` | The most characters of the text, 1 to 2000. |
| `rows` | A template for one row of a list. |
| `max_rows` | The most rows of a list, 1 to 200. |

A panel needs at least one of `picture`, `text` or `rows`.

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
| `context` | `wm`, `space`, `app`, `state`, `agents`, `chat`, `output` |
| `clock` | `time` |
| `event` | the fields of the data of the event, as `{field}` and `{field.sub}` |

### When

`when` compares fields of the context with a value or a list of values.
All keys must match. A value of `state`, `agents` or `chat` that is not in the list below is an error that names the key, for example `when.chat: "opened" is not one of open, none`. The other fields take any name. The fields:

| Field | Values |
|---|---|
| `wm` | Name of the window manager that owns the focused window: `tiling`, `xp`, `mc`, `construct`. |
| `space` | Name of the client that claims the real output of the focused window; it differs from `wm` when a window manager runs in a construct pane. |
| `app` | The `app_id` of the focused window. |
| `state` | `asleep`, `listening`, `thinking`, `working`, `question`, `speaking`. |
| `agents` | `none`, `any`, `working`, `question`. |
| `chat` | `open`, `none`. |
| `output` | The name of the output. |

```yaml
when: {wm: [xp, mc], agents: any}
```

### Chat

The block `chat:` styles the chat view and its input field. It holds
values only. A folder without the block gets the neutral chat.

| Key | Meaning |
|---|---|
| `style` | `plain`, `bubble` or `glass`. |
| `fill`, `text`, `name`, `border` | Colours as `[r, g, b]`, 0 to 255. A colour left out takes the host's. |
| `font` | Font family. Default: the font of the folder. |
| `size` | Text size in pixels, 14 to 48; a value outside is refused. Default 14. |
| `anchor` | The place. Default `right`. |
| `width` | The width in pixels, 1 to 1600. Default 400. |

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
- Text colours need a contrast of 4.5:1 against their fill
  (`chora_kit::hud::contrast`, `Chat::contrast_errors`). The key line and
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

### The mic

A panel with `source: voice` and `action: mute` mutes the mic on a
click and never unmutes it. While the mic is muted, the host draws a
badge, "Microphone off. Click to turn it on.", beside the figure, above
the card; a click on it that a person made turns the mic on. An injected
click on the badge or the figure does not; it shows the line "Please use your own mouse or
keyboard.". A folder whose voice panel has no `mute`
action has a figure that takes no click, and the badge is still there.
