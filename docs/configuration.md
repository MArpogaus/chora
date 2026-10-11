# Configuration

!!! note "Draft"
    Proposal R2 in [Decisions](decisions.md) moves views, flavors and
    services out of the core. This page shows that model.

Plain YAML files configure Void. They are readable, live in git, and an agent
can read and write them. A strict schema validates each file and rejects
unknown keys. The schema also catches the YAML traps, for example `no` read as
`false`.

The core reads no configuration file. It starts with one command, for
example the coordinator:

```bash
void -- voidctl up tiling
```

## Locations

```text
~/.config/void/
├── views/      # one file per view
├── flavors/    # bundles of views, plugins and services
├── plugins/    # plugin manifests
└── shaders/    # GLSL ES shaders
```

The directory is a git repository. It contains no secrets.

## Views

A view is a part of the scene and the plugins that drive it. `voidctl` loads
a view: it starts the plugins and applies the nodes as one transaction.

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
  effects: [{shader: vignette.frag, uniforms: {strength: 0.3}}]
```

The tiling plugin places the surfaces in `workspace`. The view only names the
plugin and its configuration.

## Flavors

A flavor is a directory with views, plugin manifests and services, plus a
systemd user target that starts them. systemd handles start, restart and
order, so Void needs no service manager.

```text
flavors/agentic/
├── agentic.target
├── views/       # morning, analysis, writing, coding
├── plugins/     # canvas layout, keybindings
└── services/    # Quadlet units: speech, webview host
```

## Services

A service is a Podman container as a Quadlet unit, or a distrobox container.
A service connects to the IPC socket, appears as a Wayland client, or both.
