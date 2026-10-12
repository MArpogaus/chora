# API

!!! note "Draft"
    A first sketch of the one API of chora. Names and shapes will change. The
    principle stays: plugins, `choractl` and agents use one language of
    commands and events.

## Model

A **command** changes the scene. An **event** reports a change. Commands
travel in **transactions** (see [Architecture](architecture.md#transactions)).
[Ownership](architecture.md#ownership) decides who may change a node.

```yaml
# one transaction
- create: {id: work, kind: group, parent: root}
- create: {id: bar, kind: rect, parent: work, props: {size: [1280, 30], color: [0.2, 0.4, 0.8, 1]}}
- attach: {surface: s42, parent: work}
- configure: {surface: s42, size: [900, 600], states: [activated]}
- update: {id: s42, props: {transform: {translate: [40, 40]}}}
- focus: {surface: s42}
```

`create` and `update` take the node properties in `props`. Unknown fields
are errors, so a typo never passes silently.

## Nodes

This table is the one list of node properties.

| Property | Nodes | Meaning |
|----------|-------|---------|
| `id` | all | stable id; ids of the form `s1`, `s2` belong to surfaces, and the core assigns them |
| `owner` | all | client name of the owner; read-only |
| `transform` | all | a 4x4 matrix, or the short form translate, scale, rotate |
| `opacity`, `visible` | all | |
| `clip` | all | rectangle, optionally rounded |
| `effects` | group, rect, image | up to 8 effects `{"shader": path, "uniforms": {name: number or 2 to 4 numbers}}`; the node becomes a layer, and the core applies the shaders in order; `[]` removes them |
| `input` | all | `accept`, `pass` or `block`; `pass` and `block` apply to all descendants |
| `meta` | all | free key/value pairs to tag and find nodes |
| `size` | group, rect, image, shader | width and height; an image without a size has the size of its file; for a group, the area of its layer |
| `mesh` | group, rect, image | `cols` and `rows` from 1 to 64, and `points`: the place of each grid point in node coordinates, row by row from the top left; the node becomes a layer that shows through the grid; `points: []` removes the mesh |
| `color`, `radius`, `border` | rect | |
| `image` | image | absolute path of a PNG file; the core loads it again after a change |
| `slice` | image | insets left, top, right, bottom in pixels of the file: the corners keep their size, the edges and the center stretch |
| `shader`, `uniforms` | shader | GLSL ES file and its inputs; a uniform can name another node as a texture |

A surface node gets its size from its client. The `configure` command asks
the client for a new size.

A node with a size and a mesh, or with a transform that is not a plain 2D
translation and scale, for example a rotation or a perspective, is a layer
(see [Architecture](architecture.md#renderer)). `animate` moves the points
of a mesh of the same grid, for example for a magic lamp minimize.

An effect is a GLSL ES fragment shader with absolute path. It reads the
layer in `uniform sampler2D tex` at `varying vec2 v_uv`, and may use
`uniform float alpha`, `uniform vec2 size` (the texture size in pixels),
`uniform float time` (seconds) and its own `float` and `vec2` to `vec4`
uniforms. A transaction that names a shader that does not compile fails
with the log of the compiler. The core compiles a shader again when its
file changes. The package installs examples in `/usr/share/chora/shaders/`:
`grayscale.frag`, `rounded.frag` and `blur.frag` (with `radius`).

The core loads the file of an image when a transaction names it. A file
that does not load, that is larger than 32 MiB or larger than 8192 pixels
on a side rejects the transaction. A plugin in a container shares its
asset directory with the core, as it shares the socket directory.

## Commands

| Command | Purpose |
|---------|---------|
| `create` | create a group, rect, image or shader node under a parent |
| `attach` | put a surface without an owner under a parent; the caller becomes its owner |
| `update` | change node properties |
| `animate` | change properties over a duration with an easing curve |
| `move` | change the parent or the position among the siblings |
| `remove` | remove a node; a removed surface becomes detached |
| `configure` | send size, toplevel states (activated, tiled, maximized, fullscreen) and decoration mode to a client |
| `focus` | set the keyboard focus |
| `close` | ask a client to close a surface |
| `camera` | set the camera of an output, or restore its default camera without a transform |

`animate` lets plugins and agents animate without code in every frame:

```json
{"animate": {"id": "frame", "props": {"opacity": 0, "visible": false},
             "duration": 300, "easing": "ease_out"}}
```

The core moves `transform`, `opacity`, `size` and `color` toward the
targets along the curve: `linear` (the default), `ease_in`, `ease_out` or
`ease_in_out`. The other properties change at the end, so this example
fades the node out and then hides it. The duration is in milliseconds, at
most one minute. A later `animate` or `update` of the node ends the
animation where it is. The end sends `animation.done`.

## Selectors

A selector finds nodes: `id:work`, `kind:surface`, `owner:tiling`,
`app_id:firefox`, `title:"make test"` (exact), `title~:Inbox` (contains),
`meta:role=editor`. Conditions separated by spaces must all match. A value
with spaces goes in double quotes. An empty selector or `*` matches every
node. Selectors work in queries only. A query returns all current matches,
or an empty list. It always sees the current titles.

## Events

| Event | Data |
|-------|------|
| `surface.created` | a toplevel waits for a plugin: surface id, `app_id`, title, and `decoration`: `server` when the client asked for a frame of the plugins through `xdg-decoration`, `client` when it draws its own |
| `surface.changed` | surface id, `app_id`, title, size and `decoration`, after any of them changed |
| `surface.destroyed`, `surface.detached` | surface id; `surface.destroyed` follows only a `surface.created` |
| `surface.requested` | surface id and `request`: `activate`, `close`, `fullscreen`, `unfullscreen`, `maximize`, `unmaximize` or `minimize`; a client or a taskbar asks, and plugins decide |
| `input.key` | the grabbed binding |
| `pointer.entered`, `pointer.left` | the pointer came over or left the top node under it that takes input: node; not during a drag of a node or while locked |
| `pointer.pressed` | a button went down over a node that takes input: node, Linux button code, position in scene coordinates |
| `pointer.moved`, `pointer.released` | after a press over a rect or an image, until the button goes up: node and position; `pointer.released` with the button |
| `animation.done` | the node whose animation reached its targets |
| `effect.failed` | the shader of an effect failed after a change of its file: path and log; the effect keeps its last working version |
| `effects.dropped` | the watchdog removed the effects of all nodes, because frames were slow for two seconds |
| `output.changed` | output name, layout position and size, scale, camera, and the work area: the scene area that the output shows without panels |
| `output.removed` | output name |
| `client.connected`, `client.disconnected` | client name |
| `method.called` | another client calls a method that this connection exported: `call` id, `caller`, `method` and `params`; only the exporting connection gets it, subscribed or not |

The response of `scene.apply` reports the result of a transaction: `null`
when it applied, an error with the index of the failed command when not.
A request that a client makes before its first commit follows its
`surface.created`. A taskbar asks through `wlr-foreign-toplevel-management`.
The core does not act on a request, also not on `close`. A plugin answers
with commands, for example `close`. A press over a surface reaches its
client as well; a press over a rect or an image goes to plugins only, with
the moves and the release that follow, so that a plugin can drag a window
by its title bar. Later milestones add events for size limits.

## IPC methods

JSON-RPC 2.0, one message per line, on the socket in `CHORA_SOCKET`. Every
client sends `hello` first. The core answers every request.

| Method | Purpose |
|--------|---------|
| `hello(name, keep_nodes)` | name the client; `keep_nodes` keeps its nodes and cameras after the disconnect |
| `scene.query(selector)` | read nodes |
| `scene.apply(commands)` | apply a transaction |
| `events.subscribe(events)` | stream the named events, or all events for an empty list, as notifications |
| `input.grab(binding)` | get the events of a key, a button or a gesture; returns the binding as `input.key` names it |
| `input.inject(events)` | send synthetic input: pointer, buttons, scrolling, keys, shortcuts and text; needs a grant |
| `screen.capture(output, node, max_side)` | capture an output or a node as a PNG file; needs a grant |
| `grants.set(client, capabilities)`, `grants.revoke(client, capabilities)`, `grants.list` | grant or revoke `input.inject` and `screen.capture` for a client name; `choractl grant` and `choractl revoke` call them |
| `output.list` | list the outputs and their cameras |
| `clients.list` | list the connected client names and the methods that each exported |
| `methods.export(names)` | offer methods of this client to other clients |
| `methods.call(client, method, params)` | call a method of another client; the answer is the result of the method |
| `methods.return(call, result, error)` | answer a `method.called` event |

A connection sends `hello` once, within 5 s. Connections with the same name share their
nodes, so they must agree on `keep_nodes`. The core refuses a `hello` with a
different value. For example, `choractl --name tiling` fails while the tiling
plugin runs.

A key binding needs Ctrl, Alt or Super, so that typing still works. The
bindings of the safety layer, Ctrl+Alt with BackSpace, Escape or F1 to F12,
belong to no client. A binding has one owner at a time.

The core implements `wlr-output-management`, so kanshi and wlr-randr
configure the outputs (see
[Architecture](architecture.md#output-configuration)). The API has no output
configuration of its own. After a new mode, position, transform or scale,
the core sends `output.changed`. For an output that turns off, it sends
`output.removed`, and `output.list` no longer lists the output.

## Grants

`input.inject` and `screen.capture` see or drive the whole session, so they
need a grant of the user for the client name (D40):

```console
$ choractl grant agent screen.capture input.inject
$ choractl grants
{"agent": ["input.inject", "screen.capture"]}
$ choractl revoke agent
```

A grant lasts until `choractl revoke` or the end of the session. The
emergency reset revokes all grants. A grant protects against mistakes of
agents, not against hostile code of the same user, which can call
`grants.set` itself. `choractl mcp` offers no tool for grants.

## Screen capture

`screen.capture` renders an output, or a node with its descendants, and
answers with a PNG file in base64:

| Parameter | Meaning |
|-----------|---------|
| `output` | the output name; the first output if neither `output` nor `node` is set |
| `node` | a node id, for example a surface or a group |
| `max_side` | shrink the picture until its longer side has at most this many pixels |

The result has the `width` and `height` in pixels, the `scale` in pixels per
logical pixel, the `origin` and the `png`. A pixel at (px, py) shows the
point `origin + (px, py) / scale`: in the logical coordinates of the output,
whose origin is (0, 0), or in the coordinates of the node.

- An output shows as on screen, without the cursor.
- A node shows without its own transform, opacity, mesh and effects, on a
  transparent background. Its area covers the descendants with a size.
- A picture has at most 16 Mi pixels and 8192 pixels a side; a larger one
  shrinks. `max_side` must be at least 1.
- A capture fails while the session is locked, and when the PNG file is
  larger than 5 MiB; a smaller `max_side` helps then.

`choractl capture FILE` saves a picture, for example
`choractl --name agent capture --node s1 window.png`.

## Input injection

`input.inject` sends a list of events as one request:

```json
{"events": [
  {"pointer_move": {"position": [640, 400], "output": "DP-1"}},
  {"click": {"button": 272}},
  {"text": {"text": "make test\n"}},
  {"shortcut": {"keys": "Super+Return"}}
]}
```

| Event | Fields |
|-------|--------|
| `pointer_move` | `position` in the logical coordinates of the output, as in a capture; `output`, the first one by default |
| `button` | `button`, a Linux code, 272 (left) by default; `state`: `pressed` or `released` |
| `click` | `button`; presses and releases it |
| `scroll` | `dx` and `dy` in logical pixels; a positive `dy` scrolls down |
| `key` | `key`: a keysym name such as `Return`, or a Linux key code such as 28; `state` |
| `shortcut` | `keys`, a binding such as `Ctrl+Shift+t`: the modifiers go down, the key goes down and up, the modifiers go up |
| `text` | `text`; a newline presses Return, a tab presses Tab |

- The core checks all events before the first one runs. An event that does
  not resolve, for example a character without a key, rejects the request.
- Key names and text use the first layout of the keymap. A character needs
  a key with Shift or `AltGr` at most.
- Keys and buttons that the events leave down go up at the end of the
  request, so that no key sticks.
- Injected keys never reach the safety layer: Ctrl+Alt+BackSpace,
  Ctrl+Alt+Escape and the VT switch do nothing. The grabs of plugins see
  them, and the focused surface gets the rest.
- A request has at most 256 events, 4096 characters of text in all and
  32,768 steps, a key press and a release being two. It fails while the
  session is locked.

`choractl inject` reads the list of events as JSON, for example
`echo '[{"text": {"text": "ls\n"}}]' | choractl --name agent inject`.

## Plugin methods

A plugin exports methods, and other clients call them by the client name
and the method name, for example `tiling.set_layout`:

```console
$ choractl call tiling set_layout '{"layout": "columns"}'
{"gap": 4, "layout": "columns", "windows": ["s1", "s2"]}
```

The core sends the exporting connection a `method.called` event. The
plugin answers with `methods.return`: the same `call` and a `result`, or an
`error` with a `code` and a `message`. The core passes the answer to the
caller. The `Client` of `chora-scene` answers with `Client::answer`.

- A method name has at most 64 characters: lowercase letters, digits, `_`,
  `-` and `.`, starting with a letter.
- A connection exports at most 64 methods. Another connection of the same
  client name cannot export the same method.
- A call without an answer fails after 10 s, and so does a call whose
  exporter disconnects. A connection has at most 64 calls in flight, and it
  cannot call its own methods.

`chora-tiling` exports two methods:

| Method | Parameters | Result |
|--------|------------|--------|
| `set_layout` | `layout`: `master_stack`, `columns` or `rows`; `gap` in logical pixels; `reserve`: pixels left free at the left, top, right and bottom of the work area; a missing field keeps its value | the layout, the gap, the reserve and the tiled windows |
| `get_layout` | none | the same |

## Plugins

A plugin is a client with a manifest:

```yaml
name: tiling
exec: chora-tiling
configure: set_layout
```

`choractl view load` reads the manifest of each plugin of a view: it starts
a plugin that does not run with `exec`, and passes the `config` of the view
to the method in `configure` (see
[Configuration](configuration.md#plugin-manifests)). A flavor starts its
plugins as systemd units instead.

## MCP tools

`choractl mcp` serves the API as MCP tools over stdio (D34). It connects
as the client `agent`, or under `--name`. An MCP client starts it, for
example in the `.mcp.json` of Claude Code:

```json
{"mcpServers": {"chora": {"command": "choractl", "args": ["--name", "agent", "mcp"]}}}
```

| Tool | Purpose |
|------|---------|
| `describe_screen` | a screenshot of an output, at most 1280 pixels a side by default, and in words: the outputs, the windows with their positions in scene coordinates and sizes, and the plugins with their methods |
| `capture` | a picture of an output or of a node, as `screen.capture`, at most 1920 pixels a side by default |
| `query_scene` | the nodes of a selector |
| `apply_commands` | a transaction |
| `list_plugins` | the connected clients and their methods |
| `call_method` | a method of a plugin, for example `tiling.set_layout` |
| `inject_input` | synthetic input, as `input.inject` |
| `load_view`, `save_view`, `unload_view` | the views of [Configuration](configuration.md#views), by name only |

The screenshot of `describe_screen`, `capture` and `inject_input` need the
grants of the client name, for example
`choractl grant agent screen.capture`. Without a grant, `describe_screen`
still describes the scene. No tool grants anything (D40).

A tool that fails, also on invalid arguments, answers with `isError` and
the reason, so that the model can correct its call. An unknown tool is a
JSON-RPC error.
