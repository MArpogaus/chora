# API

!!! note "Before 1.0"
    The API changes without notice before 1.0 (D3). The
    [Roadmap](roadmap.md) lists the planned changes. Plugins, `choractl` and
    agents use one language of commands and events.

## Model

A **command** changes the scene. An **event** reports a change. Commands
travel in **transactions** (see [Architecture](architecture.md#transactions)).
[Ownership](architecture.md#ownership) decides who may change a node.

<!-- parses: commands -->
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

The nodes form a tree under `root`, which the cameras show. Each output
has a second root, `screen:NAME`, for example `screen:DP-1` (D43). Its
nodes stay fixed to the screen: they sit in the logical coordinates of the
output, without its camera, above the scene and below the top and overlay
layers of layer shell. A HUD or a progress bar goes there, also during an
animated camera move. The nodes under `screen:NAME` work as all others,
with input, effects and ownership. The overlay root stays when its output
goes, with its nodes, for the return of the output; the emergency reset
empties it.

This table is the one list of node properties.

| Property | Nodes | Meaning |
|----------|-------|---------|
| `id` | all | stable id; ids of the form `s1`, `s2` belong to surfaces, and the core assigns them |
| `owner` | all | client name of the owner; read-only |
| `transform` | all | a 4x4 matrix, or the short form translate, scale, rotate |
| `opacity`, `visible` | all | |
| `clip` | group, rect, image, shader | `{"radius": r}`: the node and its descendants show only inside its size, with corners rounded by `r` logical pixels, and take pointer input only there; the node becomes a layer; `null` removes the clip |
| `effects` | group, rect, image | up to 8 effects `{"shader": path, "uniforms": {name: number or 2 to 4 numbers}}`; the node becomes a layer, and the core applies the shaders in order; `[]` removes them |
| `input` | all | `accept`, `pass` or `block`; `pass` and `block` apply to all descendants |
| `meta` | all | free key/value pairs to tag and find nodes |
| `size` | group, rect, image, shader | width and height; an image without a size has the size of its file; for a group, the area of its layer |
| `mesh` | group, rect, image | `cols` and `rows` from 1 to 64, and `points`: the place of each grid point in node coordinates, row by row from the top left; optional `depth`: the z of each point, which the transform of the node projects; the node becomes a layer that shows through the grid; `points: []` removes the mesh |
| `color` | rect | RGBA, each from 0 to 1, with straight alpha: `[1, 1, 1, 0.5]` is white at half opacity |
| `radius` | rect | the radius of the corners in logical pixels; the rect becomes a layer |
| `border` | rect | `{"width": w, "color": [r, g, b, a]}`: a line of `w` logical pixels along the inner edge; the rect becomes a layer; `null` removes it |
| `image` | image | absolute path of a PNG file; the core loads it again after a change |
| `slice` | image | insets left, top, right, bottom in pixels of the file: the corners keep their size, the edges and the center stretch |
| `shader`, `uniforms` | shader | the absolute path of a GLSL ES fragment shader, and its uniforms as for an effect; a uniform `{"node": id}` is a `sampler2D` with that node as it shows on its own, at most 4 |

A surface node gets its size from its client. The `configure` command asks
the client for a new size.

A shader node draws its shader over its size, as a layer. Its children
render into `uniform sampler2D tex`, and the shader gets the same inputs as
an effect. A node texture shows the node without its transform, at its size
or, for a surface, at the size of the window. A node with opacity 0 hides
in the scene and still shows in a node texture. Inside a node texture,
shader nodes get no node textures of their own, so that two shader nodes
that read each other do not recurse.

<!-- parses: commands -->
```yaml
# a rounded button with a border, a round clip and a shader that reads it
- create: {id: button, kind: rect, parent: root, props: {size: [120, 40], color: [0.2, 0.4, 0.8, 1], radius: 8, border: {width: 2, color: [1, 1, 1, 1]}}}
- create: {id: avatar, kind: group, parent: root, props: {size: [64, 64], clip: {radius: 32}}}
- create: {id: glow, kind: shader, parent: root, props: {size: [400, 300], shader: /home/me/glow.frag, uniforms: {strength: 0.5, picture: {node: avatar}}}}
- update: {id: button, props: {border: null}}
```

A node with a size and a mesh, or with a transform that is not a plain 2D
translation and scale, for example a rotation or a perspective, is a layer
(see [Architecture](architecture.md#renderer)). `animate` moves the points
of a mesh of the same grid, for example for a magic lamp minimize.

With `depth`, a grid bends into 3D, for example a strip on a ring around
the viewer, and a perspective transform projects it. The texture maps
perspective-correct. A triangle with a point behind the viewer does
not show; the rest of the mesh does.

An effect is a GLSL ES fragment shader with absolute path. It reads the
layer in `uniform sampler2D tex` at `varying vec2 v_uv`, and may use
`uniform float alpha`, `uniform vec2 size` (the texture size in pixels),
`uniform float time` (seconds) and its own `float` and `vec2` to `vec4`
uniforms. The texture holds premultiplied colors, and the shader writes
premultiplied colors; the core passes the uniforms as they are, so a
color uniform is whatever the shader takes it to be. A transaction that
names a shader that does not compile fails with the log of the compiler.
The core compiles a shader again when its file changes. The package installs examples in `/usr/share/chora/shaders/`:
`grayscale.frag`, `rounded.frag` and `blur.frag` (with `radius`).

Two hints for shaders. With `precision mediump float`, the common hash
`fract(sin(dot(p, …)) * 43758.5)` loses its digits and gives garbage;
noise and hashes need `precision highp float`. `fwidth`, `dFdx` and `dFdy`
need `GL_OES_standard_derivatives`, which the core enables for a shader
that uses them.

The core loads the file of an image when a transaction names it. A file
that does not load, that is larger than 32 MiB or larger than 8192 pixels
on a side rejects the transaction. A plugin in a container shares its
asset directory with the core, as it shares the socket directory.

A client that shares no directory with the core, for example an agent in
another container, hands its files to the core with `assets.put`. The
core stores them in `$XDG_RUNTIME_DIR/chora/` and answers with the absolute
path for `image`, `shader` and effects. A file has at most 512 KiB, a
client at most 64 files, all clients together at most 64 MiB. A client
name with assets has at most 120 bytes. A second put
of a name replaces the file, and the nodes that use it load it again. The
files of a client go with its nodes: at the last disconnect without
`keep_nodes`, otherwise when chora exits. `choractl asset put NAME FILE`
prints the path.

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
| `focus` | set the keyboard focus to an own surface, or clear it without a surface; a client that cleared the focus may give it back once to the surface that had it, also of another client |
| `close` | ask a client to close a surface |
| `camera` | set the camera of an output, or restore its default camera without a transform; with a `duration`, move it there; the first client that sets the camera owns it, and a `camera` without a transform gives it up |

`animate` lets plugins and agents animate without code in every frame:

<!-- parses: command -->
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

A camera moves the same way, with `duration` and `easing`. It starts
where the camera is now, also in the middle of another move, and the end
sends `animation.done` with the output and then `output.changed`:

<!-- parses: command -->
```json
{"camera": {"output": "DP-1", "transform": {"scale": 2, "translate": [-640, -400]},
            "duration": 500, "easing": "ease_in_out"}}
```

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
| `input.key` | the grabbed `binding` and its `state`: `pressed` when its key goes down, `released` when it goes up |
| `pointer.entered`, `pointer.left` | the pointer came over or left the top node under it that takes input: node, and for `pointer.entered` `moved`: false when the scene moved under a pointer that stood still; not during a drag of a node or while locked |
| `pointer.pressed` | a button went down over a node that takes input: node, Linux button code, position in scene coordinates, or in output coordinates for a node under `screen:NAME` |
| `pointer.moved`, `pointer.released` | after a press over a rect or an image, until the button goes up: node and position; `pointer.released` with the button |
| `animation.done` | the `node` whose animation reached its targets, or the `output` whose camera did |
| `effect.failed` | the shader of an effect failed after a change of its file: path and log; the effect keeps its last working version |
| `layers.dropped` | the layers of these `nodes` did not fit into the pixel budget of a frame of the `output` and stay out of it; sent when the list changes, and an empty list means that all show again |
| `effects.dropped` | the watchdog removed the effects, the meshes and the shaders of shader nodes of all nodes, because frames were slow for two seconds |
| `output.changed` | output name, layout position and size, scale, camera, and the work area: the scene area that the output shows without panels and reserves; `default_work_area` is the same area as the default camera shows it, the place for windows that a camera move should not move |
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
| `events.subscribe(events)` | stream the named events, or all events for an empty list, as notifications; each subscribed connection gets each event, also when several connections share a client name |
| `input.grab(binding)` | get the events of a key, a button or a gesture; returns the binding as `input.key` names it |
| `input.ungrab(binding)` | end a grab of this connection; a grab also ends with its connection |
| `input.inject(events)` | send synthetic input: pointer, buttons, scrolling, keys, shortcuts and text; needs a grant |
| `screen.capture(output, node, max_side)` | capture an output or a node as a PNG file; needs a grant |
| `grants.set(client, capabilities)`, `grants.revoke(client, capabilities)`, `grants.list` | grant or revoke `input.inject`, `screen.capture` and `process.spawn` for a client name; `choractl grant` and `choractl revoke` call them |
| `output.list` | list the outputs and their cameras |
| `output.reserve(output, edges)` | leave space at the edges of an output out of its work area while the connection lives, as the exclusive zone of a panel does; `edges` has `top`, `right`, `bottom` and `left` in logical pixels, and no space removes the reserve; returns the output |
| `output.configure(output, scale, transform)` | set the scale or the transform of an output, the first one without `output`; `transform` is `normal`, `90`, `180`, `270`, `flipped`, `flipped-90`, `flipped-180` or `flipped-270`; returns the output |
| `clients.list` | list the connected client names and the methods that each exported |
| `methods.export(names)` | offer methods of this client to other clients |
| `methods.call(client, method, params)` | call a method of another client; the answer is the result of the method |
| `methods.return(call, result, error)` | answer a `method.called` event |
| `process.spawn(argv, capture)` | start a program with the environment of the core; with `capture`, the core keeps its output for `process.wait`; needs the grant `process.spawn` |
| `process.wait(pid, timeout_ms)` | wait up to `timeout_ms` (at most 30000) for a program of `process.spawn` with `capture`, and return its output so far; needs the grant `process.spawn` |
| `plugins.restart(name)` | stop the process of a client name and start its command again; needs the grant `process.spawn` |
| `assets.put(name, data)` | store a file in base64 for image nodes, shader nodes and effects; returns its `path` |
| `assets.remove(name)` | remove a file of `assets.put` |

An error has a `code` and a `message`: the JSON-RPC codes for a method or
parameters that do not exist, 1 for a request that breaks a rule, and 2
for a transaction that names a surface node that is gone, for example of a
window that closed in the meantime. A plugin can ignore code 2.

A connection sends `hello` once, within 5 s. Connections with the same name share their
nodes, so they must agree on `keep_nodes`. The core refuses a `hello` with a
different value. For example, `choractl --name tiling` fails while the tiling
plugin runs.

A reserve of `output.reserve` belongs to a connection and ends with it,
also when its client crashes. The reserves of all connections on an
output add up, and count from the edges of the area that the exclusive
zones of layer surfaces leave free. At least one pixel of the work area
stays. `choractl reserve --top 30` holds a reserve until it ends.

A key binding needs Ctrl, Alt or Super, so that typing still works. The
bindings of the safety layer, Ctrl+Alt with BackSpace, Escape or F1 to F12,
belong to no client. A binding has one owner at a time.

The core implements `wlr-output-management`, so kanshi and wlr-randr
configure the outputs (see
[Architecture](architecture.md#output-configuration)). `output.configure`
sets the scale and the transform on the same path (D44), for an agent
without wlr-randr. After a new mode, position, transform or scale,
the core sends `output.changed`. For an output that turns off, it sends
`output.removed`, and `output.list` no longer lists the output.

## Grants

`input.inject` and `screen.capture` see or drive the whole session, and
`process.spawn` runs any program as the user, so they need a grant of the
user for the client name. Milestone 0.7 replaces the grants with the
capabilities of client sockets (D33, D40):

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

## Programs

`process.spawn(argv)` starts a program with the environment of the core,
as the startup command (D30): `WAYLAND_DISPLAY` and `CHORA_SOCKET` of this
compositor, without the variables of a host session, and with no stdin.
The core looks the program up in its own `PATH`, so it works for an agent
in another namespace too. The answer has the `pid` and the `argv`.

`plugins.restart(name)` restarts a plugin, for example after a rebuild.
The core finds the process of the connections of the client name,
reads its command line and working directory from `/proc`, and sends it
SIGTERM, and a SIGKILL 5 s later. Once it exited, the core closes its
connections that only wait for answers, as these reach nobody. Once no
connection of the name is left, the core starts the same command again
and answers with the new `pid`. When that takes more than 6 s, the restart
fails. A name with several
processes, or a process in a namespace that the core cannot see, fails.

With `capture: true`, `process.spawn` keeps stdout and stderr of the
program, up to 64 KiB each, for the client that started it.
`process.wait(pid, timeout_ms)` waits until the program ends or the time
is up, and answers with `pid`, `running`, `stdout`, `stderr`, `truncated`
for output that went beyond 64 KiB, and, once the program ended, `status`:
its exit code, or 128 and the signal. 100 ms after the end of the
program, the core closes its pipes, also when a child in the background
still holds them. The first answer after the end is the last one; then
the core forgets the program. At most 32 programs of a client wait for
`process.wait`. Their output goes when the last connection of the client
name closes, so `choractl spawn --capture` waits on its own connection.

Both methods need the grant `process.spawn`, and fail while the session is
locked:

```console
$ choractl grant agent process.spawn
$ choractl --name agent spawn foot
$ choractl --name agent spawn --capture -- uname -s
{"pid": 4242, "running": false, "status": 0, "stdout": "Linux\n", "stderr": "", "truncated": false}
$ choractl --name agent restart tiling
```

Milestone 0.7 moves the start of programs from the core to `choractl run`
and systemd (D42).

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
  transparent background, but with its rounded corners, border, clip and
  the shader of a shader node. Its area covers the descendants with a
  size.
- A picture has at most 16 Mi pixels and 8192 pixels a side; a larger one
  shrinks. `max_side` must be at least 1.
- A capture renders the scene again. Layers that the pixel budget of the
  last frame left out of the screen show in the capture; `left_out` names
  them, and `layers.dropped` tells it as it happens.
- A capture fails while the session is locked, and when the PNG file is
  larger than 5 MiB; a smaller `max_side` helps then.

`choractl capture FILE` saves a picture, for example
`choractl --name agent capture --node s1 window.png`.

## Input injection

`input.inject` sends a list of events as one request:

<!-- parses: inject -->
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
$ choractl call tiling set_layout '{"layout": "fair"}'
{"columns": 1, "gap": 4, "layout": "fair", "master_width": 0.5, "masters": 1, "windows": ["s2", "s1"]}
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
| `set_layout` | `layout` of the current tag: `tile`, `tile_left`, `tile_bottom`, `tile_top`, `fair`, `fair_horizontal`, `spiral`, `dwindle`, `max`, `fullscreen`, `magnifier` or `floating`; `gap` in logical pixels; a missing field keeps its value | the layout, the gap, `master_width`, `masters` and `columns` of the current tag, and the windows that show on the current output |
| `get_layout` | none | the same |

## Plugins

A plugin is a client with a manifest:

<!-- parses: manifest -->
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
| `spawn_process`, `restart_plugin` | start a program, or restart a plugin, as `process.spawn` and `plugins.restart`; `spawn_process` with `capture` keeps the output |
| `wait_process` | the output of a program of `spawn_process` with `capture`, as `process.wait` |
| `reserve_space` | space at the edges of an output, as `output.reserve`, while the server runs |
| `configure_output` | the scale or the transform of an output, as `output.configure` |
| `put_asset` | a file for image nodes, shader nodes and effects, as `assets.put`: `name`, and `text` for a shader or `base64` for a PNG; returns the path |
| `subscribe_events` | start to collect events, and grab keys; `events` and `keys` as in `events.subscribe` and `input.grab`; replaces the current subscription |
| `wait_events` | the collected events: waits up to `timeout_ms` (5000 by default, at most 30000) for the first one, and returns at most `max` (50 by default, at most 500) |

The input schemas of `apply_commands` and `inject_input` come from the
types of `chora-scene`. They list every command, every input event and
every field, with its description, so the model sees the shape of a call
before the first try. A test checks that the schemas, the types and the
examples of this page agree.

The screenshot of `describe_screen`, `capture`, `inject_input`,
`spawn_process` and `restart_plugin` need the grants of the client name,
for example `choractl grant agent screen.capture`. Without a grant, `describe_screen`
still describes the scene. No tool grants anything (D40).

A subscription has a connection of its own under the same client name. A
thread reads its events into a buffer of 1000 events, so the core never
waits for the agent. When the buffer is full, the oldest events go, and
`wait_events` counts them in `dropped`. The grabs of the keys end with
the subscription.

A tool that fails, also on invalid arguments, answers with `isError` and
the reason, so that the model can correct its call. An unknown tool is a
JSON-RPC error.
