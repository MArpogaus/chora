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

`chora-assistant --check-hud DIR` prints each error with its file and
key and exits with 1. A running host keeps the look it has when a new
folder fails, and shows a short notice. The notice names no command;
this page does.

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
| `space` | Name of the client that owns the real output of the focused window. |
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

## What a folder cannot do

The host draws the question card, the muted badge and the words itself.
A folder styles them and cannot hide, cut, relabel or move them. The
card is always on top of every panel and of the chat view; neither a
folder nor a client picks its place. A
folder without a visible panel with `source: voice` gets the neutral
dot. Words come from one table in English and German; they follow the
`language` setting.
