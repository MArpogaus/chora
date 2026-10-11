# Configuration

Views are YAML files in git, and an agent can read and write them. A strict
schema validates each file. It rejects unknown keys and catches the YAML
traps, for example `no` read as `false`.

The core reads no configuration file. It starts one command, usually the
systemd target of a flavor:

```bash
chora -- systemctl --user start tiling.target
```

## Locations

```text
~/.config/chora/
├── views/      # one file per view
├── flavors/    # bundles of views, plugins and services
├── plugins/    # plugin manifests
└── shaders/    # GLSL ES shaders
```

The directory is a git repository. It contains no secrets.

## Views

A view is a part of the scene and the plugins that drive it.
`choractl view load <name>` starts the plugins of the view and applies its
nodes as one transaction. Only `choractl` starts plugins.

```yaml
# ~/.config/chora/views/analysis.yaml
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
  effects: [{shader: vignette.frag, uniforms: {strength: 0.3}}]
```

The tiling plugin places the surfaces in `workspace`. The view only names the
plugin and its configuration.

## Flavors

A flavor is a directory with views, plugin manifests and services, plus a
systemd user target. The target starts the services and then loads the first
view with `choractl`. systemd handles start, restart and order, so Chora
needs no service manager.

```text
flavors/agentic/
├── agentic.target
├── views/       # morning, analysis, writing, coding
├── plugins/     # canvas layout, keybindings
└── services/    # Quadlet units: speech, browser in app mode
```

## Services

A service runs as a Quadlet unit with Podman, or in distrobox. It connects
to the IPC socket, appears as a Wayland client, or both. A container gets the
IPC socket through a bind mount.
