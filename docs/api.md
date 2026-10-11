# API

!!! note "Draft"
    A first sketch of the one API Void exposes. Names and shapes will change;
    the principle — one command/event language for plugins, IPC and MCP —
    should not.

## Model

Everything is either a **command** (changes or queries the scene) or an
**event** (something happened). Commands are grouped into **transactions**
that apply atomically at the next frame boundary.

```yaml
# one transaction, as sent over IPC or returned by a plugin
transaction:
  - create: {id: work, kind: group, parent: root, transform: {x: 0, y: 0}}
  - create: {id: bg, kind: shader, parent: work, shader: shaders/aurora.frag,
             size: {w: 1.0, h: 1.0, unit: parent}}
  - attach: {surface: "app_id:org.wezfurlong.wezterm", parent: work,
             transform: {x: 40, y: 40}, size: {w: 900, h: 600}}
  - update: {id: work, effects: [{shader: blur, uniforms: {radius: 8}}]}
  - focus:  {surface: "app_id:org.wezfurlong.wezterm"}
```

## Scene commands

| Command | Purpose |
|---------|---------|
| `create` | create a node (group, rect, image, shader) under a parent |
| `attach` | place a surface node (by id or selector) under a parent |
| `update` | change properties of a node (transform, opacity, effects, meta, …) |
| `animate` | interpolate properties over time with an easing curve (convenience, so simple plugins and agents need no per-frame logic) |
| `move` | reparent / reorder a node |
| `remove` | remove a node (a removed surface becomes detached, not closed) |
| `focus` | set keyboard focus |
| `close` | ask a client to close a surface |
| `query` | read nodes, filtered by selector |

**Selectors** find nodes without knowing ids: `id:work`,
`app_id:firefox`, `title~:"Inbox"`, `meta:role=editor`, `kind:surface`.

## Events

| Event | Payload |
|-------|---------|
| `surface.created` / `surface.destroyed` | surface id, app_id, title, kind (toplevel, layer, popup) |
| `surface.changed` | title, app_id, requested size, fullscreen/maximize requests |
| `input.key`, `input.pointer`, `input.touch`, `input.gesture` | raw and resolved data |
| `output.added` / `output.removed` / `output.changed` | outputs and modes |
| `frame` | time, dt (plugins only) |
| `plugin.loaded` / `plugin.failed` | name, error |
| `transaction.applied` / `transaction.rejected` | id, error |
| `view.loaded` | view name |

## IPC methods

JSON-RPC 2.0 over `$XDG_RUNTIME_DIR/void/void.sock`.

| Method | Capability |
|--------|------------|
| `scene.query(selector)` | `scene.read` |
| `scene.apply(transaction)` | `scene.write` |
| `scene.undo(n)` | `scene.write` |
| `events.subscribe(filter)` | `scene.read` (input events: `input.read`) |
| `input.inject(events)` | `input.inject` |
| `screen.capture(target)` | `screen.capture` |
| `view.list / view.load / view.save / view.diff` | `scene.write` / `scene.read` |
| `plugin.list / plugin.load / plugin.unload / plugin.reload` | `plugin.manage` |
| `shader.compile(source)` | `scene.write` |
| `output.list / output.configure` | `output.read` / `output.write` |

## Capabilities

| Capability | Grants |
|------------|--------|
| `scene.read` | query nodes and subscribe to non-input events |
| `scene.write` | apply transactions, load/save views, compile shaders |
| `input.read` | observe input events |
| `input.inject` | synthesize keyboard/pointer input |
| `screen.capture` | screenshots of outputs, groups or surfaces |
| `plugin.manage` | load, unload, reload plugins |
| `output.write` | change modes, scale, arrangement |

Tokens are issued by the coordinator (`voidctl token create --cap scene.read,scene.write`).

## Plugin interface

A plugin is a WASM module with a small set of exports; data crosses the
boundary as serialized commands and events (JSON first; a binary encoding
later if profiling demands it).

| Export | Called |
|--------|--------|
| `init(config) -> transaction` | once on load, with the plugin's config from the view |
| `on_event(event) -> {consumed, transaction}` | for every event the plugin subscribed to |
| `on_frame(frame) -> transaction` | every frame, only if the plugin asked for frames |
| `shutdown()` | before unload / reload |

Host functions available to plugins: `scene_query`, `log`, `timer_set`,
`state_get` / `state_set` (state that survives a reload).

A plugin declares its subscriptions and capabilities in a manifest:

```yaml
name: tiling
version: 0.1.0
module: tiling.wasm
subscribe: [surface.created, surface.destroyed, input.key, output.changed]
frames: false
capabilities: [scene.read, scene.write]
```

## MCP tools

`void-mcp` maps the IPC methods 1:1 to MCP tools (`scene_query`,
`scene_apply`, `screen_capture`, `view_load`, …) and exposes the current
scene and the list of views as MCP resources. Screenshots are returned as
images, so the agent can see what it built.
