# Project structure

The layout of this repository: one Cargo workspace with as few crates as
possible. A module moves into its own crate only when a second consumer
can use it. The plugins move into repositories of their own with 1.0
(D36).

```text
chora/
├── crates/
│   ├── chora-scene/   # scene model, commands, events, selectors, IPC types and client; no Smithay
│   ├── chora/         # the compositor: Smithay, backends, renderer, input, safety, IPC
│   ├── chora-kit/     # building blocks for plugins: the plugin runtime, wobbly windows, magic lamp, XDG dirs, icons, desktop entries
│   └── choractl/      # CLI over the IPC socket; `choractl mcp` is the MCP bridge
├── plugins/
│   ├── chora-tiling/  # reference plugin: the tags, layouts and bindings of awesome
│   ├── chora-xp/      # stacking window manager in the style of Windows XP
│   ├── chora-mc/      # experimental: a voice-first desktop, one full-screen scene at a time
│   └── chora-construct/ # experimental: a spatial desktop with panes that face the camera
├── flavors/           # systemd units of the flavors (tiling, xp, construct); the theme of the XP flavor
├── examples/          # loader prototype and showcase apps over the API
├── services/voice/    # the voice agent and the quadlets of its models
├── dist/              # RPM build script, Fedora build requirements, session files, Quadlet examples
├── tests/             # end-to-end scripts, headless sessions, input probe
├── deny.toml          # cargo-deny: allowed licenses, advisories
├── ruff.toml          # ruff: the rules for the Python in services/voice/
├── docs/              # this documentation (MkDocs)
├── mkdocs.yml
└── Cargo.toml         # workspace, lints
```

## Reasons for this split

- **`chora-scene` has no Smithay dependency.** Tests of the data model and
  the transaction logic need no GPU and no compositor. The core, `choractl`
  and plugins share this one definition of the API.
- **`chora-kit` holds the building blocks that plugins share** and that
  are not part of the API of the core: the IPC and event connection, the
  plugin runtime, the self-computed effects and the freedesktop files.
  Code of one plugin stays in that plugin.
- **Reference plugins are workspace members.** They keep the API honest.
- **`choractl` builds the JSON schemas** from the Rust types at run time, in
  `crates/choractl/src/schema.rs`. The repository does not contain them.
  The MCP tools and the test of the documentation examples then see the
  same contract.

## Related repositories

| Repository | Role |
|------------|------|
| `secureblue-images` | derived image that installs chora as an additional session |
| `image-builder-action` | builds and signs that image |
| `distrobox-images` | container images for services |
