# API

!!! note "Draft"
    A first sketch of the one API of Chora. Names and shapes will change. The
    principle stays: plugins, IPC clients and agents use one language of
    commands and events.

## Model

A **command** changes the scene. An **event** reports a change. Commands
travel in **transactions** (see [Architecture](architecture.md#transactions)).

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
| `owner` | all | the client that created or attached the node; read-only |
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
| `viewport` | set the viewport of an output: a transform from scene coordinates to output pixels |

Only the owner of a node can change it. `animate` lets plugins and agents
animate without code in every frame.

## Selectors

A selector finds nodes without an id: `id:work`, `app_id:firefox`,
`title~:Inbox`, `meta:role=editor`, `kind:surface`. Selectors work in queries
only. A query returns all current matches, or an empty list. It always sees
the current titles.

## Events

| Event | Data |
|-------|------|
| `surface.created`, `surface.destroyed` | surface id, `app_id`, title, role (toplevel, layer, popup) |
| `surface.changed` | title, `app_id`, min and max size, requests for maximize, fullscreen and activation |
| `input.key`, `input.pointer`, `input.touch`, `input.gesture` | event data for grabs |
| `output.added`, `output.removed`, `output.changed` | output, mode, scale |
| `plugin.connected`, `plugin.disconnected` | name, reason |
| `transaction.applied`, `transaction.rejected` | id, error, for example a shader compile error |

## IPC methods

JSON-RPC 2.0 on the socket in `CHORA_SOCKET`.

| Method | Purpose |
|--------|---------|
| `scene.query(selector)` | read nodes |
| `scene.apply(transaction)` | apply a transaction |
| `events.subscribe(filter)` | stream events |
| `input.grab(binding)` | get the events of a key, a button or a gesture |
| `input.inject(events)` | send synthetic input; needs approval |
| `screen.capture(target)` | capture an output, a group or a surface; needs approval |
| `output.list` | list the outputs and their viewports |
| `methods.export(names)` | offer methods of this plugin to other clients |
| `methods.call(plugin, method, params)` | call a method of a plugin |

The core implements `wlr-output-management`, so kanshi and wlr-randr
configure the outputs. The API has no output configuration of its own.

## Plugins

A plugin is an IPC client with a manifest:

```yaml
name: tiling
exec: chora-tiling
subscribe: [surface.created, surface.destroyed, output.changed]
grabs: [Super+h, Super+l]
methods: [set_layout, resize]
```

A plugin owns the nodes that it creates. The core removes them when the
plugin disconnects.

## MCP tools

`choractl mcp` offers the IPC methods as MCP tools. It also offers tools for
whole tasks, for example "describe the screen", which returns a screenshot
and the scene tree. The agent then sees the result of its changes.
