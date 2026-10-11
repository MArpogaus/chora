# Configuration

Void is configured with plain YAML files: readable, diffable, kept in git, and
easy for an agent to read and write. YAML's pitfalls (implicit typing such as
`no` → `false`) are handled by a strict schema: every file is validated, and
unknown keys are errors.

## Locations

```text
~/.config/void/
├── void.yaml          # core settings: outputs, safe view, socket, tokens
├── views/             # one file per view
├── flavors/           # bundles of views, plugins, services
├── plugins/           # installed plugins (manifest + .wasm)
└── shaders/           # GLSL ES shaders referenced by views
```

The directory is meant to be a git repository.

## Core settings

```yaml
# ~/.config/void/void.yaml
startup:
  flavor: tiling        # or: view: morning
safe_view: grid         # loaded by the emergency reset
outputs:
  DP-1: {mode: 2560x1440@144, scale: 1.25, position: [0, 0]}
ipc:
  socket: $XDG_RUNTIME_DIR/void/void.sock
```

## Views

A view is a scene fragment plus the plugins that drive it.

```yaml
# ~/.config/void/views/analysis.yaml
name: analysis
plugins:
  - name: tiling
    config: {layout: columns, gap: 12}
  - name: keybindings
    config:
      bindings:
        Super+Return: {spawn: wezterm}
        Super+q: close
scene:
  - id: workspace
    kind: group
    effects: [{shader: vignette, uniforms: {strength: 0.3}}]
    children:
      - {id: notebook, attach: "app_id:marimo", meta: {role: main}}
      - {id: plots, attach: "title~:Plot", meta: {role: side}}
      - {id: status, kind: rect, color: "#00000080", size: {w: 1.0, h: 32, unit: parent}}
```

Loading a view diffs it against the current scene and applies only the
difference as one transaction, so switching views can be animated.

## Flavors

A flavor bundles views, plugins and services into an installable starting
point.

```yaml
# ~/.config/void/flavors/agentic.yaml
name: agentic
plugins: [canvas, animations, keybindings]
services:
  - name: speech
    container: ghcr.io/example/void-speech:latest   # whisper + piper
  - name: webview
    container: ghcr.io/example/void-webview:latest
  - name: mcp
    exec: void-mcp
views: [morning, analysis, writing, coding]
start: morning
```

Planned flavors: **classic** (stacking + taskbar), **tiling**, **agentic**.

## Services

Services are declared in flavors and started by the coordinator — at first
`voidctl` (which wraps Podman/distrobox), later an agent. A service either
connects to the IPC socket with a token, or simply appears as a Wayland
client, or both.
