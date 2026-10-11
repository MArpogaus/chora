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
| `pointer.pressed` | a button went down over a node that takes input: node, Linux button code, position in scene coordinates |
| `pointer.moved`, `pointer.released` | after a press over a rect or an image, until the button goes up: node and position; `pointer.released` with the button |
| `animation.done` | the node whose animation reached its targets |
| `effect.failed` | the shader of an effect failed after a change of its file: path and log; the effect keeps its last working version |
| `effects.dropped` | the watchdog removed the effects of all nodes, because frames were slow for two seconds |
| `output.changed` | output name, layout position and size, scale, camera, and the work area: the scene area that the output shows without panels |
| `output.removed` | output name |
| `client.connected`, `client.disconnected` | client name |

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
| `input.inject(events)` | send synthetic input; needs a grant |
| `screen.capture(target)` | capture an output, a group or a surface; needs a grant |
| `output.list` | list the outputs and their cameras |
| `methods.export(names)` | offer methods of this client to other clients |
| `methods.call(client, method, params)` | call a method of another client |

A connection sends `hello` once, within 5 s. Connections with the same name share their
nodes, so they must agree on `keep_nodes`. The core refuses a `hello` with a
different value. For example, `choractl --name tiling` fails while the tiling
plugin runs.

A key binding needs Ctrl, Alt or Super, so that typing still works. Ctrl+Alt
belongs to the safety layer, so no client can grab it. A binding has one
owner at a time.

The core implements `wlr-output-management`, so kanshi and wlr-randr
configure the outputs (see
[Architecture](architecture.md#output-configuration)). The API has no output
configuration of its own. After a new mode, position, transform or scale,
the core sends `output.changed`. For an output that turns off, it sends
`output.removed`, and `output.list` no longer lists the output.

## Plugins

A plugin is a client with a manifest:

```yaml
name: tiling
exec: chora-tiling
subscribe: [surface.created, surface.detached, surface.destroyed, output.changed]
grabs: [Super+Return, Super+d, Super+q, Super+j, Super+k]
```

`choractl` reads the manifest when it loads a view. Before views exist (M4),
the startup command starts plugins directly.

## MCP tools

`choractl mcp` offers the IPC methods as MCP tools. It also offers tools for
whole tasks, for example "describe the screen", which returns a screenshot
and the scene tree. The agent then sees the result of its changes.
