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
- create: {id: bg, kind: shader, parent: work, shader: aurora.frag}
- attach: {surface: s42, parent: work}
- configure: {surface: s42, size: [900, 600], states: [activated]}
- update: {id: s42, transform: {translate: [40, 40, 0]}}
- update: {id: work, effects: [{shader: blur.frag, uniforms: {radius: 8}}]}
- focus: {surface: s42}
```

## Nodes

This table is the one list of node properties.

| Property | Nodes | Meaning |
|----------|-------|---------|
| `id` | all | stable id; the core assigns ids to surfaces |
| `owner` | all | client name of the owner; read-only |
| `transform` | all | a 4x4 matrix, or the short form translate, scale, rotate |
| `opacity`, `visible` | all | |
| `clip` | all | rectangle, optionally rounded |
| `effects` | all | list of shaders with uniforms |
| `input` | all | `accept`, `pass` or `block` |
| `meta` | all | free key/value pairs to tag and find nodes |
| `size` | rect, shader | width and height |
| `color`, `radius`, `border` | rect | |
| `shader`, `uniforms` | shader | GLSL ES file and its inputs; a uniform can name another node as a texture |

A surface node gets its size from its client. The `configure` command asks
the client for a new size.

## Commands

| Command | Purpose |
|---------|---------|
| `create` | create a group, rect or shader node under a parent |
| `attach` | put a detached surface node under a parent |
| `update` | change node properties |
| `animate` | change properties over a duration with an easing curve |
| `move` | change the parent or the position among the siblings |
| `remove` | remove a node; a removed surface becomes detached |
| `configure` | send size, toplevel states (activated, tiled, maximized, fullscreen) and decoration mode to a client |
| `focus` | set the keyboard focus |
| `close` | ask a client to close a surface |
| `camera` | set the camera of an output |

`animate` lets plugins and agents animate without code in every frame.

## Selectors

A selector finds nodes: `id:work`, `app_id:firefox`, `title~:Inbox`,
`meta:role=editor`, `kind:surface`. Selectors work in queries only. A query
returns all current matches, or an empty list. It always sees the current
titles.

## Events

| Event | Data |
|-------|------|
| `surface.created`, `surface.destroyed` | surface id, `app_id`, title, role (toplevel, layer, popup) |
| `surface.changed` | title, `app_id`, min and max size, requests for maximize, fullscreen and activation |
| `surface.detached` | surface id |
| `input.key`, `input.pointer`, `input.touch`, `input.gesture` | event data for grabs |
| `output.added`, `output.removed`, `output.changed` | output, mode, scale, position |
| `client.connected`, `client.disconnected` | client name |
| `transaction.applied`, `transaction.rejected` | id, error, for example a shader compile error |

## IPC methods

JSON-RPC 2.0, one message per line, on the socket in `CHORA_SOCKET`. Every
client sends `hello` first.

| Method | Purpose |
|--------|---------|
| `hello(name, keep_nodes)` | name the client; `keep_nodes` keeps its nodes after the disconnect |
| `scene.query(selector)` | read nodes |
| `scene.apply(transaction)` | apply a transaction |
| `events.subscribe(filter)` | stream events as notifications |
| `input.grab(binding)` | get the events of a key, a button or a gesture |
| `input.inject(events)` | send synthetic input; needs a grant |
| `screen.capture(target)` | capture an output, a group or a surface; needs a grant |
| `output.list` | list the outputs and their cameras |
| `methods.export(names)` | offer methods of this client to other clients |
| `methods.call(client, method, params)` | call a method of another client |

The core implements `wlr-output-management`, so kanshi and wlr-randr
configure the outputs. The API has no output configuration of its own.

## Plugins

A plugin is a client with a manifest:

```yaml
name: tiling
exec: chora-tiling
subscribe: [surface.created, surface.destroyed, output.changed]
grabs: [Super+h, Super+l]
methods: [set_layout, resize]
```

`choractl` reads the manifest when it loads a view. Before views exist (M3),
the startup command starts plugins directly.

## MCP tools

`choractl mcp` offers the IPC methods as MCP tools. It also offers tools for
whole tasks, for example "describe the screen", which returns a screenshot
and the scene tree. The agent then sees the result of its changes.
