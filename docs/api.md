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
| `id` | all | stable id; ids of the form `s1`, `s2` belong to surfaces, and the core assigns them; `chora:inject` belongs to the injection mark of the core |
| `owner` | all | client name of the owner; read-only |
| `focused` | all | `true` in `scene.query` on the surface with the keyboard focus and on the node of `focus_node`; read-only, left out on all other nodes |
| `rects` | all | in `scene.query`: the output name maps to `[x, y, w, h]` in logical pixels of that output. The core applies all parent transforms and the camera, and projects like the renderer. The query leaves out outputs where the node does not show. The box holds the drawn shape under rotation, perspective or a mesh. A clip or an ancestor layer cuts the box to its own box. The mesh of a layer does not move the boxes of its descendants. Read-only. |
| `to` | surface | the only client name that can attach this detached surface, after a `remove` with `to` or for a new toplevel; read-only |
| `claim` | all | `[x, y, w, h]` in scene coordinates: the area of the owner as a window manager; a new toplevel can go to it (see [below](#window-managers)); `null` removes the claim |
| `transform` | all | a 4x4 matrix in column-major order, with the translation at the indices 12 to 14, or the short form translate, scale, rotate |
| `opacity`, `visible` | all | |
| `clip` | group, rect, image, shader | `{"radius": r}`: the node and its descendants show only inside its size, with corners rounded by `r` logical pixels, and take pointer input only there; the node becomes a layer; `null` removes the clip |
| `effects` | group, rect, image | up to 8 effects `{"shader": path, "uniforms": {name: number or 2 to 4 numbers}}`; the node becomes a layer, and the core applies the shaders in order; `[]` removes them |
| `input` | all | `accept`, `pass` or `block`; `pass` and `block` apply to all descendants |
| `meta` | all | free key/value pairs to tag and find nodes; an empty value removes a key; at most 32 keys, 1 KiB per key or value and 4 KiB in all per node |
| `size` | group, rect, image, shader | width and height; an image without a size has the size of its file; for a group, the area of its layer |
| `mesh` | group, rect, image | `cols` and `rows` from 1 to 64, and `points`: the place of each grid point in node coordinates, row by row from the top left; optional `depth`: the z of each point, which the transform of the node projects; the node becomes a layer that shows through the grid; `points: []` removes the mesh |
| `color` | rect, text | RGBA, each from 0 to 1, with straight alpha: `[1, 1, 1, 0.5]` is white at half opacity |
| `radius` | rect | the radius of the corners in logical pixels; the rect becomes a layer |
| `border` | rect | `{"width": w, "color": [r, g, b, a]}`: a line of `w` logical pixels along the inner edge; the rect becomes a layer; `null` removes it |
| `image` | image | absolute path of a PNG file; the core loads it again after a change |
| `slice` | image | insets left, top, right, bottom in pixels of the file: the corners keep their size, the edges and the center stretch |
| `shader`, `uniforms` | shader | the absolute path of a GLSL ES fragment shader, and its uniforms as for an effect; a uniform `{"node": id}` is a `sampler2D` with that node as it shows on its own, at most 4 |
| `text`, `family`, `font_size`, `max_width` | text | `text`: UTF-8, needed at `create`, at most 4 KiB; `family`: a font family, at most 4 KiB, a system font without it; `font_size`: logical pixels, from 1 to 1024; `max_width`: the width at which lines wrap, and nothing else; a text node has no `size`, it sizes itself by its layout, so it is never a layer: rotated, mirrored or projected outside a layer, it draws nothing, so put it in a group with a `size` and a `clip`; the core draws it in `color`, white at 16 logical pixels by default; the raster limit is 16 MiB per output scale, and 256 MiB for all text nodes together; a raster beyond the second limit draws nothing until its node changes; a text beyond it at scale 1 gets no rect and no draw; a text beyond it only at another scale draws nothing at that scale |

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

The core loads the file of an image when a transaction names it, on a
worker thread: the transaction answers when the file is decoded, while
other clients go on, and the later transactions of the same connection
wait for it. A file that does not load, that is larger than 32 MiB or
larger than 8192 pixels on a side rejects the transaction. A plugin in a container shares its
asset directory with the core, as it shares the socket directory.

A client that shares no directory with the core, for example an agent in
another container, hands its files to the core with `assets.put`. The
core stores them in `$XDG_RUNTIME_DIR/chora/` and answers with the absolute
path for `image`, `shader` and effects. A file has at most 512 KiB, a
client at most 64 files, all clients together at most 64 MiB. A client
name with assets has at most 120 bytes. A second put
of a name replaces the file, and the nodes that use it load it again. The
files of a client go with its nodes. With `keep_nodes`, they go when
chora exits. With `resume` only, they go after the grace time. Otherwise,
they go at the last disconnect. `choractl asset put NAME FILE` prints the
path.

## Commands

| Command | Purpose |
|---------|---------|
| `create` | create a group, rect, image, shader or text node under a parent |
| `attach` | put a surface without an owner under a parent; the caller becomes its owner |
| `update` | change node properties |
| `animate` | change properties over a duration with an easing curve |
| `move` | change the parent or the position among the siblings |
| `remove` | Remove a node. A removed surface becomes detached. With `to`, only the client `to` can attach the surface and gets `surface.detached`. `to` must have a connection (D55). |
| `configure` | send size, toplevel states (activated, tiled, maximized, fullscreen) and decoration mode to a client |
| `focus` | set the keyboard focus to an own surface, or clear it without a surface. A surface of another client answers with `node.requested` to its owner (D31). A client that cleared the focus may give it back once to the surface that had it, also of another client. A focus on another surface since then ends this return. |
| `focus_node` | give the keyboard to an own node that is no surface, or without a node give it back (see [Keyboard focus of nodes](#keyboard-focus-of-nodes)) |
| `close` | ask a client to close a surface |
| `camera` | set the camera of an output that is or was plugged in, or restore its default camera without a transform; with a `duration`, move it there; the first client that sets the camera owns it, and a `camera` without a transform gives it up |

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

## Keyboard focus of nodes

The chrome of a plugin, for example a menu or a dialog, is made of nodes
and takes the keyboard with `focus_node`:

<!-- parses: command -->
```json
{"focus_node": {"node": "start-menu"}}
```

While the node has the focus, no surface gets keys: the focused surface
gets a leave, and the core sends each key as `node.key` to the
connections of the owner of the node that subscribed to it. A press
carries the text that it types:

```json
{"event": "node.key", "node": "start-menu", "key": "a", "code": 30,
 "state": "pressed", "modifiers": ["Shift"], "text": "A"}
```

- `key` is the keysym name with the modifiers applied, `code` the Linux
  key code, and `modifiers` lists `Ctrl`, `Alt`, `Shift` and `Super` as
  they are down.
- `text` is missing for a release, for a key without text such as
  `Return`, and while Ctrl, Alt or Super is down. A dead key types
  nothing; the core does no compose.
- The core repeats a held key as a Wayland client does: after 200 ms,
  25 times a second, with the same `pressed` event. Modifiers do not
  repeat. The repeat ends when the key goes up, when another key goes
  down, or when the node loses the focus.
- The safety layer and the grabs come first, also for injected keys.

The focus returns to the focused surface, which gets an enter, in these
cases:

- The owner sends `{"focus_node": {}}`.
- The node goes.
- Its client disconnects, also with `keep_nodes` or `resume`.
- The session locks.
- A surface gets the focus, by a command or by the core.

A layer surface that takes the keyboard exclusively, for example a
launcher, gets the keys before the node. Some layer surfaces take the
keyboard on demand, for example the search field of a panel. A click on
such a surface ends the focus of the node and gives it the keyboard.

## Window managers

Several window managers share the scene (D55). A window manager sets
`claim` on a node, for example on its root group. At the first commit of
a toplevel, the core reserves it for one window manager:

1. the owner of its parent toplevel, for a dialog;
2. else the owner of the node or surface with the keyboard focus, if it
   holds a claim;
3. else the owner of the smallest claim that contains the pointer.

Only clients with a connection or in their grace time count. Only the
chosen client can attach the surface, and `surface.created` names it in
`to`. Without a match, any client attaches the surface. When the last
connection of the client closes, the reservation ends with
`surface.detached`. With `resume`, it ends at the end of the grace time.
When a window manager removes its last claim, its reservations end too. A
`remove` with `to` hands a surface to another window manager in the same
way.

## Selectors

A selector finds nodes: `id:work`, `kind:surface`, `owner:tiling`,
`app_id:firefox`, `title:"make test"` (exact), `title~:Inbox` (contains),
`meta:role=editor`. Conditions separated by spaces must all match. A value
with spaces goes in double quotes. An empty selector or `*` matches every
node. Selectors work in queries only. A query returns all current matches,
or an empty list. It always sees the current titles.

`scene.query` also takes a JSON filter next to the selector: `{"selector":
"kind:surface", "filter": {"owner": "tiling", "focused": true}}`. The
fields are `owner`, `kind`, `app_id`, `focused: true`, `visible: true` and
`output: "NAME"`. All of them must match, and so must the selector.
`focused` keeps the surface with the keyboard focus and the node of
`focus_node`. `visible` keeps nodes that show on some output,
`output` nodes that show on that one. Both read the `rects` of the
result, so an invisible node matches neither. An empty filter matches
everything;
`focused` and `visible` filter only when true. Unknown fields fail.

`choractl wait SELECTOR` polls the query every 100 ms until something
matches, then prints the matches as one JSON line and exits 0. `--gone`
waits until nothing matches. `--timeout SECS` limits the wait, 10 seconds
by default; a timeout exits 1 with the reason on stderr.

## Events

| Event | Data |
|-------|------|
| `surface.created` | a toplevel waits for a plugin: surface id, `app_id`, title, and `decoration`: `server` when the client asked for a frame of the plugins through `xdg-decoration`, `client` when it draws its own. `to` names the window manager that the core reserved it for (see [Window managers](#window-managers)). `pid` is the process of the client as the socket reports it, a hint that a pid namespace can hide; `token` is the activation token that mapped the surface, when one did. A token only reaches this event when the client asked for activation before the first map. |
| `surface.changed` | surface id, `app_id`, title, size and `decoration`, after any of them changed |
| `surface.destroyed`, `surface.detached` | surface id. `surface.destroyed` follows only a `surface.created`. A toplevel that its client unmaps with a commit without a buffer also counts as destroyed. At its next initial commit, the core sends `surface.created` with the same id. After a `remove` with `to`, only `to` gets `surface.detached`. When `to` disconnects, the surface is free again, and all clients get `surface.detached`. With `resume`, this occurs at the end of the grace time. |
| `focus.changed` | the focused surface changed, by a `focus` of a client or by the core: `surface`, or `null`. Window managers draw their frames from it (D55). |
| `surface.requested` | surface id and `request`: `activate`, `close`, `fullscreen`, `unfullscreen`, `maximize`, `unmaximize` or `minimize`; a client or a taskbar asks, and plugins decide |
| `node.requested` | a client asks to change a node of another client: `node`, the asking `client`, `action` and `args`. The actions are `focus`, `move`, `resize` and `close`, with the arguments of the command. Only the owner of the node gets it, and decides by doing the change or not (D31). Other changes of a foreign node stay refused. |
| `input.key` | the grabbed `binding` and its `state`: `pressed` each time its key goes down with its modifiers, `released` once when Ctrl, Alt or Super of the binding goes up (see [IPC methods](#ipc-methods)) |
| `input.button`, `input.moved` | the button of a grabbed button binding went down or up, or the pointer moved while it is down: `binding`, `state` (not for `input.moved`), `position` in scene coordinates, `output` and `local`, the position in the output; only to the connection of the grab |
| `node.key` | a key while a node of the client has the keyboard focus: node, keysym name, Linux code, state, modifiers and text (see [Keyboard focus of nodes](#keyboard-focus-of-nodes)) |
| `pointer.entered`, `pointer.left` | the pointer came over or left the top node under it that takes input: node, and for `pointer.entered` `moved`: false when the scene moved under a pointer that stood still; not during a drag of a node or while locked |
| `pointer.pressed` | a button went down over a node that takes input: node, Linux button code, position in scene coordinates, or in output coordinates for a node under `screen:NAME`, and the `modifiers` that are down |
| `pointer.moved`, `pointer.released` | after a press over a rect or an image, until the button goes up: node, position and modifiers; `pointer.released` with the button |
| `pointer.scroll` | the wheel or the touchpad scrolled over a rect, an image or a shader node: node, position, `dx` and `dy` in logical pixels, `discrete` wheel steps, `source` (`wheel`, `finger`, `continuous` or `wheel_tilt`) and modifiers; only to the owner of the node |
| `animation.done` | the `node` whose animation reached its targets, or the `output` whose camera did |
| `effect.failed` | the shader of an effect failed after a change of its file; the single field `error` holds the path and the log, and the effect keeps its last working version |
| `layers.dropped` | the layers of these `nodes` did not fit into the pixel budget of a frame of the `output` and stay out of it; sent when the list changes, and an empty list means that all show again |
| `effects.dropped` | the watchdog removed the effects, the meshes and the shaders of shader nodes of the nodes of `client`, because frames were slow for two seconds and its visible nodes cost the most; if frames stay slow and that client's own load reaches the owner budget (D57), the core disconnects it and removes its nodes, also with `keep_nodes` or `resume`; below the budget the slowdown is shared, the effects of all clients go and none is disconnected |
| `output.changed` | output name, layout position and size, scale, camera, and the work area: the scene area that the output shows without panels and reserves; `default_work_area` is the same area as the default camera shows it, the place for windows that a camera move should not move |
| `output.removed` | output name |
| `client.connected`, `client.disconnected` | client name |
| `method.called` | another client calls a method that this connection exported: `call` id, `caller`, `method` and `params`; only the exporting connection gets it, subscribed or not |

Every event carries `seq`, the count of its connection (D32). Every event
that the core pushes to a connection carries the next number of that
connection, also before `events.subscribe`. The answer of
`events.subscribe` is a snapshot of the current scene, the same list that
`scene.query` gives for all nodes. The snapshot covers the state before
it, the events that follow cover every change after it. A number that
skips tells the client that it missed an event, so it queries again. After
a reconnect a client works from the new snapshot and forgets the old
numbers.

The response of `scene.apply` reports the result of a transaction: `null`
when it applied, an error with the index of the failed command when not.
`choractl apply FILE` reads one list of commands as JSON, from standard
input when the file is `-`; it prints nothing when it applied and the
reason when not, so the exit code is the answer. Only the `view`
subcommand reads YAML.
A request that a client makes before its first commit follows its
`surface.created`. A taskbar asks through `wlr-foreign-toplevel-management`.
The core does not act on a request, also not on `close`. A plugin answers
with commands, for example `close`. A press over a surface reaches its
client as well; a press over a rect or an image goes to plugins only, with
the moves and the release that follow, so that a plugin can drag a window
by its title bar. `modifiers` lists `Ctrl`, `Alt`, `Shift` and `Super` as
they are down, for example for a Ctrl+click.

Scrolling over a surface reaches its client through Wayland. Over a rect,
an image or a shader node, the core sends `pointer.scroll` to the
connections of the owner of the node that subscribed to it, so that a
list of a plugin scrolls:

```json
{"event": "pointer.scroll", "node": "start-menu", "position": [40, 700],
 "dx": 0, "dy": 15, "discrete": [0, 1], "source": "wheel", "modifiers": []}
```

The number of nodes has no limit. The watchdog limits their cost: when
frames stay slow, the effects of the client whose nodes cost the most go
(see `effects.dropped`). The core disconnects that client only when its
own load reaches the owner budget (D57). `CHORA_OWNER_LOAD_BUDGET` sets
that budget, 4000 by default, as `Scene::load_by_owner` counts. When no
client alone reaches it, the slowdown is shared: all effects go and no
client goes. Later milestones add events for size limits.

## IPC methods

JSON-RPC 2.0, one message per line, on the socket in `CHORA_SOCKET`. A
client sends `hello` first. A method that changes the session or needs a
grant fails before `hello`. `scene.query`, `events.subscribe`,
`output.list`, `clients.list`, `clients.create`, `methods.list`, `camera.release`,
`grants.set`, `grants.revoke` and `grants.list` also work before it,
for tools.

The core opens an admin socket next to the IPC socket, for example
`wayland-1.admin.sock` next to `wayland-1.sock` (D40). It offers all
methods. Only there do `grants.set`, `grants.revoke` and `clients.create`
work. `choractl grant`, `choractl revoke` and `choractl clients create`
use it; `choractl clients up [FILE]` applies a YAML file of
`NAME: { grants: [...] }` entries: it opens the socket of every name
with exactly the capabilities of the list (D33, D40). A socket with the
equal capabilities stays (`clients.create` answers the `socket_exists`
reason) with its connections; changed capabilities recreate the socket
and disconnect its Wayland clients. While a name is connected or in its
grace time the socket cannot open (`clients.create` answers the
`connected` reason); `choractl clients up` then syncs the grants of
that name. It reports both reasons and goes on with the remaining
names. Without a file argument the first of
`$XDG_CONFIG_HOME/chora/clients.yaml` and
`~/.config/chora/clients.yaml` is read, `-` for stdin. The core
answers every request.

| Method | Purpose |
|--------|---------|
| `hello(name, keep_nodes, resume, protocol, features)` | name the client (lowercase letters, digits, `_`, `-` and `.`, from a letter, at most 64 bytes); `keep_nodes` keeps its nodes and cameras after the disconnect, `resume` keeps them for a grace time (see below); `protocol` and `features` say what the client speaks (see [Protocol versions](#protocol-versions)); returns `{"version": ..., "protocol": ..., "features": [...]}` |
| `scene.query(selector, filter)` | read nodes |
| `scene.apply(commands)` | apply a transaction |
| `events.subscribe(events)` | stream the named events, or all events for an empty list, as notifications; answers with a snapshot of the scene first (see [Events](#events)); each subscribed connection gets each event, also when several connections share a client name; an unknown name is refused |
| `input.grab(binding)` | get the events of a key or a pointer button with modifiers, for example `Super+Return` or `Super+BTN_LEFT`; a binding needs Ctrl, Alt or Super unless the client has the capability `input.grab-any`; `Ctrl+Alt` with a key of the safety layer stays refused either way; returns the binding as `input.key` and `input.button` name it |
| `input.ungrab(binding)` | end a grab of this connection; a grab also ends with its connection |
| `input.inject(events)` | send synthetic input: pointer, buttons, scrolling, keys, shortcuts and text; needs a grant |
| `screen.capture(output, node, max_side)` | capture an output or a node as a PNG file; needs a grant |
| `camera.release(output)` | take the camera of an output away from its owner, live or gone, without telling it, so that the view goes back to its default and any client can own the camera again; the old owner takes the camera back with its next camera command; works only on the admin socket, `choractl camera release OUTPUT` calls it |
| `grants.set(client, capabilities)`, `grants.revoke(client, capabilities)`, `grants.list` | grant or revoke capabilities for a client name. `choractl grant` and `choractl revoke` call them. `grants.set` and `grants.revoke` work only on the admin socket. A name with a client socket answers code 9, because `clients.yaml` holds its capabilities. For a name of the main socket, `methods.call` changes nothing, and `activation.token` only opens the MCP tool `run_program`. |
| `clients.create(name, capabilities)` | open the client socket of a name with exactly those capabilities and return its `path` (see [Client sockets](#client-sockets)); equal capabilities keep the socket and give back what the revoke binding took, changed ones recreate it; without `capabilities`, an existing socket keeps its own; only on the admin socket; `choractl clients create NAME` and `choractl clients up` call it |
| `output.list` | list the outputs and their cameras |
| `output.reserve(output, edges)` | leave space at the edges of an output out of its work area while the connection lives, as the exclusive zone of a panel does; `edges` has `top`, `right`, `bottom` and `left` in logical pixels, and no space removes the reserve; returns the output |
| `output.configure(output, scale, transform)` | set the scale or the transform of an output, the first one without `output`; `transform` is `normal`, `90`, `180`, `270`, `flipped`, `flipped-90`, `flipped-180` or `flipped-270`; scale and mode must keep the logical size within 16384 pixels a side; returns the output |
| `output.add(spec)`, `output.remove(output)` | for tests on the headless backend only: add a virtual output `WxH[@SCALE][+X+Y]`, which returns the output, or remove one, as a monitor plugs in or out; the mode side has 1 to 16384 pixels and `WxH` divided by `SCALE` must stay within 16384 a side and at most 16 outputs are live; other backends refuse, and the last output stays; `choractl add-output` and `choractl remove-output` call them |
| `clients.list` | list the connected client names and the methods that each exported |
| `methods.export(names)` | offer methods of this client to other clients |
| `methods.list` | list the methods that clients exported, each with its description and input schema |
| `methods.call(client, method, params)` | call a method of another client; the answer is the result of the method; a client socket needs the capability `methods.call` |
| `methods.return(call, result, error)` | answer a `method.called` event |
| `activation.token` | answer an xdg activation token of no client, for `choractl run` (D42, see [Programs](#programs)); the next toplevel that activates with it shows the token in `surface.created`; one use ends it; an unused token of no client is forgotten after 10 s; a client socket needs the capability `activation.token` |
| `assets.put(name, data)` | store a file in base64 for image nodes, shader nodes and effects; returns its `path` |
| `assets.remove(name)` | remove a file of `assets.put` |

An error has a `code`, a `message` and, where it helps, a machine
readable `data`. [Error codes](#error-codes) lists the stable codes. A
rejected transaction carries `{"command": INDEX}` in `data`, the
position of the failed command in the transaction.

A request is one line of at most 1 MiB (`MAX_LINE`). The core reads a
longer line to its end, answers it with code 6 and the id `null`, and
closes the connection. The core holds at most 8 MiB (`MAX_OUTPUT`) of
answers and events that a connection did not read yet; a connection that
passes this limit closes, so a client reads its socket while it waits. An
answer that does not fit, for example of a query of all nodes, fails with
code 6 instead, and the connection stays.

The core takes at most 128 connections, at most 16 of one process. The
last 8 go only to a process without a connection, so `choractl` and a
window manager that starts again still connect while one client holds many.
A refused connection gets an error line with code 6 and the reason, `too
many connections: ...`, before the core closes it.

`hello` answers with the version of the core: the crate version and,
for a build in a git checkout, the commit, for example
`{"version": "0.5.4+1ea0e89abc", "protocol": 1, "features": []}`.
See "Protocol versions" for the other two fields. `chora --version`
prints the same version.
The client library of `chora-scene` warns once on stderr when the core is
another build, so `choractl` and the plugins tell why a request fails
after an update.

A connection sends `hello` once, within 5 s. Connections with the same name share their
nodes, so they must agree on `keep_nodes` and `resume`. The core refuses a
`hello` with different values. For example, `choractl --name tiling` fails
while the tiling plugin runs. With `keep_nodes`, `resume` has no effect.
A `hello` with `resume` also fails while another connection of the name
is live. So a second instance of a plugin cannot take the nodes of the
first. The error says that the name is connected.

`resume` lets a plugin restart without a loss, for example after a crash
or a change of its configuration. When the last connection of a name
with `resume` closes, the core keeps the nodes of the name for a grace
time of 10 s. It keeps their tree, stacking, properties and `meta`. It
also keeps the surfaces that they hold, the cameras and the assets. The
grabs, the exported methods, the reserves and the keyboard focus of a
node end as at any disconnect, and `client.disconnected` follows at once.

```json
{"jsonrpc": "2.0", "id": 1, "method": "hello", "params": {"name": "wm", "resume": true}}
```

- A `hello` of the same name with `resume` within the grace time gets the
  nodes back. The plugin finds them with `scene.query` and `owner:NAME`.
  When the list is not empty, it rebuilds its model from them instead of
  creating its nodes again.
- A `hello` of the name without `resume` removes the kept nodes first, so
  the client starts as without a grace time.
- After the grace time, the core removes the nodes as at any disconnect:
  the surfaces become detached, and the core sends `surface.detached` for
  each.
- No client of another name can change the kept nodes. Only a name with
  a client socket is safe from other clients that take it (see
  [Client sockets](#client-sockets)).
- A toplevel that maps during the grace time gets `surface.created` as
  usual, and stays without an owner if no other plugin attaches it. The
  resumed plugin takes it as at any start, with a query of `kind:surface`
  for surfaces without an owner.
- The emergency reset and a disconnect of the watchdog end the grace time
  at once.

A reserve of `output.reserve` belongs to a connection and ends with it,
also when its client crashes. The reserves of all connections on an
output add up, and count from the edges of the area that the exclusive
zones of layer surfaces leave free. At least one pixel of the work area
stays. `choractl reserve --top 30` holds a reserve until it ends.

A key binding needs Ctrl, Alt or Super, so that typing still works, unless
the client holds `input.grab-any` (D45). The
bindings of the safety layer, Ctrl+Alt with BackSpace, Escape, i or F1 to
F12, belong to no client. A binding has one owner at a time. Revoking
`input.grab-any` drops the registrations of grabs that need no modifier
and ends a held one.

A key grab is held from the press of its key until one of its modifiers
Ctrl, Alt or Super goes up; a grab without any of them ends with the
release of its own key. Shift does not end a key grab. Each press of
the key in that time sends `pressed`; the release of the key sends nothing
and reaches no surface. When the grab ends, the client gets one
`released`. So an Alt+Tab switcher sees Alt
down, Tab, Tab, Alt up as `pressed`, `pressed`, `released`, and knows when
to switch.

A button binding names a pointer button in place of the key: `BTN_LEFT`,
`BTN_RIGHT`, `BTN_MIDDLE`, `BTN_SIDE`, `BTN_EXTRA`, `BTN_FORWARD` or
`BTN_BACK`. It needs Ctrl, Alt or Super too, or `input.grab-any`, so that
a plain click still works. While its button is down, the client of the grab gets the press,
each move and the release, also when the modifiers went up first, and no
surface gets them:

```json
{"event": "input.button", "binding": "Super+BTN_LEFT", "state": "pressed",
 "position": [812, 140], "output": "DP-1", "local": [812, 140]}
{"event": "input.moved", "binding": "Super+BTN_LEFT",
 "position": [830, 152], "output": "DP-1", "local": [830, 152]}
```

A window manager moves a window with `Super+BTN_LEFT` so, wherever the
pointer grabs it.

The core implements `wlr-output-management`, so kanshi and wlr-randr
configure the outputs (see
[Architecture](architecture.md#output-configuration)). `output.configure`
sets the scale and the transform on the same path (D44), for an agent
without wlr-randr. After a new mode, position, transform or scale,
the core sends `output.changed`. For an output that turns off, it sends
`output.removed`, and `output.list` no longer lists the output.

## Error codes

Each failure has a stable code (T2). The negative codes are the JSON-RPC
codes for requests the core cannot route. The positive codes name the
kind of refusal, and a client can act on them without reading the
message.

| code | kind | example |
| --- | --- | --- |
| -32700 | the line is no JSON request | a line without a JSON object |
| -32601 | no method of this name | `scene.drop` |
| -32602 | parameters are missing or invalid | `scene.query` with `"selector": 3` |
| 1 | the request breaks a rule of the scene or of the core | updating a node of another client |
| 2 | a surface node of a transaction is gone | a window closed during the wait |
| 3 | a protocol the core does not speak | a `hello` with `protocol` 2 (D29) |
| 4 | a capability the method needs | `screen.capture` without a grant |
| 5 | an unknown node | `update` of a node id that never existed |
| 6 | a limit of the core is reached | a request longer than 1 MiB |
| 7 | a client name that is taken | a `hello` with a name that has a client socket |
| 8 | a method of the admin socket | `grants.set` on the main socket |
| 9 | a rule of client sockets | a `hello` with another name on `clients/agent/ipc.sock` |

Code 1 stays for every refusal the table has no own code for, for
example a scene rule. The message always names the rule. A plugin can
ignore code 2. A rejected transaction, so codes 1, 2, 5 and 6, carries
the index of the failed command in `data`.

## Protocol versions

Both sides of the IPC say which protocol they speak (D29). `hello` takes
an optional `protocol`, an integer, and a list `features`. It answers
with the `protocol`, the `features` and the `version` of the core:

```json
{"jsonrpc": "2.0", "id": 1, "method": "hello", "params": {"name": "wm", "protocol": 1, "features": []}}
{"jsonrpc": "2.0", "id": 1, "result": {"version": "0.7.0+1ea0e89", "protocol": 1, "features": []}}
```

The core speaks protocol 1. It refuses a client with a higher protocol
with code 3, and the client must upgrade its core. A lower protocol
works, so an older client keeps running. A missing `protocol` means a
client from before the versioning, and works. Compatible additions
arrive as named features while the protocol stays 1; a breaking change
raises the number. The core answers with its own list, never with a mix.
The client library of `chora-scene` sends its own `protocol` and
`features`. It warns once on stderr when the core answers with another
protocol.

## Grants

`input.inject` and `screen.capture` see or drive the whole session, and
`input.grab-any` lets a client catch keys and buttons without a
modifier, so they need a grant of the user for the client name. A name
with a client socket needs no grant: `clients.create` and
`clients.yaml` give it the capabilities
(D33, D40):

```console
$ choractl grant agent screen.capture input.inject
$ choractl grants
{"agent": ["input.inject", "screen.capture"]}
$ choractl revoke agent
```

A grant lasts until `choractl revoke` or the end of the session. The
emergency reset revokes all grants. A connection through a client socket
instead has exactly the capabilities the socket was opened with. A
recreate with changed capabilities ends the Wayland clients that
connected before it, and so does the reset. Only the admin socket
changes grants. A process of the user on the host reaches the admin
socket too. So a
grant protects against hostile code only when that code sees its client
socket alone, with the gaps that [Client sockets](#client-sockets) lists.
`choractl mcp` offers no tool for grants.

## Client sockets

A client socket fixes the name of its clients (D33). `clients.create`
on the admin socket opens `clients/NAME/ipc.sock` below the directory of
the main socket, which has mode 0700. `capabilities` says what a
connection through it may do; `clients.yaml` and `clients up` fill it
in. `choractl clients create agent` prints its path:

```json
{"jsonrpc": "2.0", "id": 1, "method": "clients.create", "params": {"name": "agent", "capabilities": ["screen.capture"]}}
```

- `clients.create` also opens `clients/NAME/wayland.sock`, a Wayland
  socket for the windows of the client. Its answer has the `path` of the
  IPC socket and the `wayland` path.
- A connection through it says `hello` with that name only.
- It cannot change grants or make client sockets, as these need the
  admin socket.
- It fixes the name, confines the files and narrows the scene. On it,
  `scene.apply` follows these rules. A refusal by one of them has code 9,
  names its rule and carries the index of the command in `data`. The
  rules of the scene keep codes 1, 2, 5 and 6:
  - `focus_node` with a node needs the last press of the user on a node
    or a window of the name. The press counts once, and a key press
    ends it.
  - A name that holds the focus of a node moves it to its other nodes
    without a press.
  - No `create` or `move` puts a node under `screen:*` or below it.
  - No `camera` command works.
  - `attach`, `focus`, `close` and `configure` take only a window of its
    own `wayland.sock` or a surface that the name owns. `focus` needs a
    press as `focus_node` does.
- `scene.query` on it returns only the nodes of the name and the windows
  of its `wayland.sock`. The `surface.*` events come only for these
  windows.
- A claim of the name gets only the new windows of its `wayland.sock`.
- The socket has exactly the capabilities of its `clients.create`; the
  other methods answer by them. A process that may use the socket
  drives the session as far as they reach.
- `grants.set` and `grants.revoke` answer code 9 for the name: change
  `clients.yaml` and run `choractl clients up`.
- `methods.call` needs the capability `methods.call`, and
  `activation.token` the capability `activation.token`. Without them
  both answer code 4. The main socket needs neither. A token of
  `activation.token` belongs to no client, so the window of a program
  started with it takes the focus without a press. A window of its
  `wayland.sock` asks for tokens through `xdg-activation`, under the
  focus rules above.
- Its `image`, `shader` and effect paths name only its own files of
  `assets.put`. The main socket has no such rule: a client of it runs as
  the same account as the core and could read those files anyway.
- It holds at most 16 connections, and its Wayland socket at most 16
  clients. All client sockets together leave 32 connections to the main
  socket.
- The main socket refuses a `hello` with a name that has a client
  socket, so no client of the main socket takes that name.
- Equal capabilities keep an existing socket. `clients.create` answers
  the `socket_exists` reason with the `path` of that socket. `choractl
  clients create NAME` prints `NAME has a client socket already: PATH`
  on stderr and exits 1. Changed capabilities recreate the socket at
  the same path and disconnect its Wayland clients.
- `clients.create` fails for a name with a connection or in its grace
  time, and beyond 64 client sockets.
- The name has lowercase letters, digits, `_`, `-` and `.`, and starts
  with a letter, at most 64 bytes; `hello` and `clients.create` refuse
  everything else. Its socket path must fit a Unix socket. A second
  `clients.create` of a name takes only other capabilities.
- The emergency reset removes the client sockets, and so does the exit of
  chora.

The callers of `methods.call` are certain only for names with a client
socket.

A Wayland client of `wayland.sock` gets the protocols of the main Wayland
socket, with these exceptions:

- Screencopy and image capture need the grant `screen.capture` of the
  name.
- The input method needs the grant `input.inject`.
- Output management, the session lock, both foreign toplevel protocols,
  the security context manager, idle inhibit and idle notify are not
  offered. On Wayland the client cannot lock the session, list the
  windows, control the windows of others, open a socket of its own or
  keep the screen awake. On `ipc.sock`, `focus_node` takes the keyboard
  only after a press of the user on the client, as a token below does.

The layer shell stays, so a panel can run in a container. Such a client
can draw over all windows in the top layer, but the core refuses the
overlay layer. A layer surface of it takes the keyboard on demand at
most, never exclusively. After a click on it, it gets the keys until a
click goes elsewhere. So a full-screen panel can look like a lock screen
and read what the user types into it.

An `xdg-activation` token or a popup grab of such a client needs the
serial of its last key or button press, and never 0. A token also needs
the keyboard focus on the client, and uses the press up. So the client
takes the focus only after input, once per press. Synthetic input from
`input.inject` sets that serial too, so a client with that grant can
create the press itself. When the token is used, it must still be at
most 10 s old and the client must still have the keyboard focus.

The client gets the capabilities of the socket at the time it connects.
A recreate with changed capabilities disconnects it, and so does the
emergency reset. Its frames of screencopy and image capture that wait
then get nothing. Its socket closes at its next request. Its windows carry the name: a query shows it in
`surface.client`. It reads the clipboard only while it has the keyboard
focus, as every Wayland client does.

A container that mounts only its client directory cannot take another
name or give itself capabilities.

## Programs

The core starts no program except its own startup command (D30, D42).
`choractl run` starts programs outside the core, and systemd keeps
plugins running.

`choractl run [--] CMD ARGS...` starts a program outside the core. It
asks the core for an activation token with `activation.token` and
passes it to the program in `XDG_ACTIVATION_TOKEN`. It prints the pid,
the unit and the token as one JSON line while the program runs.

- When `systemctl --user is-system-running` shows a state other than
  `offline`, a user manager runs. `systemd-run --user` then starts the
  program in the transient unit `chora-run-PID-TIME.service`.
  `journalctl --user -u UNIT` shows its output. The unit lives until the
  last process in it ends, so an app that a launcher starts lives on.
  The arguments reach the program literally, without the expansion of
  `$` by systemd.
- This needs systemd 254 or later. When `systemd-run` fails, for example
  because the unit exists already, `run` exits 1 with its error and
  starts nothing.
- The unit gets the environment of the user manager, plus the token and
  the values of the caller for `PATH`, `XDG_RUNTIME_DIR`,
  `WAYLAND_DISPLAY` and `CHORA_SOCKET`. It gets `DISPLAY` and the other
  host session variables only from the caller, never from the user
  manager. It starts in the directory of the caller.
- Without a running user manager, `choractl` starts the program itself,
  with the environment of the caller.
  Its stdout goes to `/dev/null`, its stderr to the stderr of the
  caller, and `unit` is `null`.
- When the unit starts but its `MainPID` stays unknown after a short
  wait, for example because the program ended at once, `pid` is `null`.
  The command never starts the program twice.

A window that activates with the token before its first map shows the
token in `surface.created`. `chora-tiling` and `chora-xp` start apps with
`choractl run` too, so a restart of a window manager leaves its apps
running.

```console
$ choractl run -- foot
{"pid":4343,"token":"a1b2c3d4e5f60718","unit":"chora-run-4342-1761400000123456789.service"}
```

`choractl restart NAME` restarts a plugin with `systemctl --user restart`.
The unit is `chora-NAME-SESSION.service` of a view in this session,
where SESSION is a short hash of `CHORA_SOCKET`, or else
`chora-NAME.service` of a flavor (see [Plugins](#plugins)). The flavor
unit counts only in the session whose `WAYLAND_DISPLAY` the user
manager holds, so a nested chora never restarts a plugin of its host.
A name without an active unit is an error: the command exits 1 with the
reason on stderr, and starts nothing. It prints the unit as one JSON
line and calls no core.

```console
$ choractl restart tiling
{"unit":"chora-tiling.service"}
```

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

A click on a node needs no client math, whatever pan, zoom or parent
scale the scene shows:

<!-- parses: inject -->
```json
{"events": [
  {"pointer_move": {"node": "menu", "point": [10, 20]}},
  {"click": {"button": 272}}
]}
```

| Event | Fields |
|-------|--------|
| `pointer_move` | `position` in the logical coordinates of the output, as in a capture; `output`, the first one by default; or `node` with `point` inside it |
| `button` | `button`, a Linux code, 272 (left) by default; `state`: `pressed` or `released`; `hold`: a press stays down when the request ends |
| `click` | `button`; presses and releases it |
| `scroll` | `dx` and `dy` in logical pixels; a positive `dy` scrolls down |
| `key` | `key`: a keysym name such as `Return`, or a Linux key code such as 28; `state` |
| `shortcut` | `keys`, a binding such as `Ctrl+Shift+t`: the modifiers go down, the key goes down and up, the modifiers go up |
| `text` | `text`; with an enabled text input at the focus the core commits the whole string, else a newline presses Return, a tab presses Tab |

- `node` and `point` replace `position`. The core maps the point through
  the same transforms that give the `rects` of a query for the node. The
  point must lie in the box the node draws. An `output` limits the search
  to that output, and a node that shows no such point rejects the
  request.
- The core checks all events before the first one runs. An event that does
  not resolve, for example a character without a key, rejects the request.
- Without an enabled text input at the focus, key names and text use the
  first layout of the keymap. A character then needs a key with Shift or
  `AltGr` at most.
- A focused surface with an enabled text input receives each text whole as
  one commit. The commit carries characters the keymap cannot reach, for
  example German umlauts on a US layout.
- Keys and buttons that the events leave down go up at the end of the
  request, so that no key sticks.
- A button press with `hold` stays down. A later button event releases
  it, in the same request or in a later one.
- Injected keys never reach the safety layer: Ctrl+Alt+BackSpace,
  Ctrl+Alt+Escape, Ctrl+Alt+i and the VT switch do nothing. The grabs of
  plugins see them, and the focused surface gets the rest.
- Every executed request shows the injection mark: a small yellow rect of
  the core next to the pointer, gone 500 ms after the last injected event
  (see [Safety layer](architecture.md#safety-layer)).
- A request has at most 256 events, 4096 characters of text in all and
  32,768 steps, a key press and a release being two. It fails while the
  session is locked.
- With `"dry_run": true` next to `events`, the core checks every event and
  answers ok or the error, without sending input.

`choractl inject` reads the list of events as JSON, for example
`echo '[{"text": {"text": "ls\n"}}]' | choractl --name agent inject`.
`choractl inject --dry-run` checks the events without sending input.

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

Each entry of `methods.export` is a plain name, or an object with `name`,
`description` and `input_schema`. `methods.list` answers every exported
method as `{"client", "name", "description", "input_schema"}`, with `null`
for a plain export. The core passes description and schema through
unchanged, and `chora-tiling` gives one to each of its methods.

- A method name has at most 64 characters: lowercase letters, digits, `_`,
  `-` and `.`, starting with a letter.
- A connection exports at most 64 methods. Another connection of the same
  client name cannot export the same method.
- A call without an answer fails after 10 s, and so does a call whose
  exporter disconnects. A connection has at most 64 calls in flight, and it
  cannot call its own methods.

`chora-tiling` exports these three; [chora-tiling](configuration.md#chora-tiling)
lists all of its methods:

| Method | Parameters | Result |
|--------|------------|--------|
| `set_layout` | `layout` of the current tag: `tile`, `tile_left`, `tile_bottom`, `tile_top`, `fair`, `fair_horizontal`, `spiral`, `dwindle`, `max`, `fullscreen`, `magnifier` or `floating`; `gap` in logical pixels; a missing field keeps its value | the layout, the gap, `master_width`, `masters` and `columns` of the current tag, and the windows that show on the current output |
| `get_layout` | none | the same |
| `give` | `to`: a client name | hands the focused window to `to` with a `remove` with `to`, and answers with its surface id |

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
[Configuration](configuration.md#plugin-manifests)). The start goes
through `choractl run`. With a user manager, the unit
`chora-NAME-SESSION.service` restarts the plugin when it fails. It
carries the session, so that every chora on the same user manager
starts its own plugins. A flavor starts its plugins as its own systemd
units `chora-NAME.service` instead. A client socket loads only views
whose plugins run already: a plugin that it starts would get its socket
and could not say `hello` with its own name.

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
| `run_program` | start a program as `choractl run` does, on the machine of the MCP server; answers pid, unit and token |
| `reserve_space` | space at the edges of an output, as `output.reserve`, while the server runs |
| `configure_output` | the scale or the transform of an output, as `output.configure` |
| `put_asset` | a file for image nodes, shader nodes and effects, as `assets.put`: `name`, and `text` for a shader or `base64` for a PNG; returns the path |
| `subscribe_events` | start to collect events, and grab keys and buttons; `events` and `keys` as in `events.subscribe` and `input.grab`; replaces the current subscription |
| `wait_events` | the collected events: waits up to `timeout_ms` (5000 by default, at most 30000) for the first one, and returns at most `max` (50 by default, at most 500, 0 takes nothing) |

The input schemas of `apply_commands` and `inject_input` come from the
types of `chora-scene`. They list every command, every input event and
every field, with its description, so the model sees the shape of a call
before the first try. A test checks that the schemas, the types and the
examples of this page agree.

The screenshot of `describe_screen`, `capture` and `inject_input` need
the grants of the client name, for example
`choractl grant agent screen.capture`. Without a grant, `describe_screen`
still describes the scene. No tool grants anything (D40). On a client
socket, `call_method` and the tools of plugin methods need the capability
`methods.call`. `run_program` needs the grant `activation.token` on
every socket, because a started program can do all that the user can.
On the main socket, `choractl grant agent activation.token` gives it; a
client socket lists it in `clients.yaml`. The program starts where the
MCP server runs, for example inside the container of an agent.

A subscription has a connection of its own under the same client name. A
thread reads its events into a buffer of 1000 events, so the core never
waits for the agent. When the buffer is full, the oldest events go, and
`wait_events` counts them in `dropped`. The grabs of the keys end with
the subscription.

A tool that fails, also on invalid arguments, answers with `isError` and
the reason, so that the model can correct its call. An unknown tool is a
JSON-RPC error.

Every method that a plugin exports is a tool of its own, named
`client.method`, with the description and the input schema of the
export. `call_method` stays for the general case. The tool list answers
fresh on every `tools/list`, so a later `methods.export` needs no
restart. A call of such a tool passes its whole arguments object as the
params of the method.

While the core is not reachable, `tools/list` answers with the built-in
tools alone and logs the reason on stderr. The plugin tools come back with
the next call.

The server answers requests in parallel: a running `wait_events` blocks
no other call. A `notifications/cancelled` ends the wait of
`wait_events` within 100 ms; the cancelled call gets no answer, as the
protocol keeps it.
