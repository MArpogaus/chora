# Project structure

The layout of this repository: one Cargo workspace with as few crates as
possible. A module moves into its own crate only when a second consumer
needs it.

```text
chora/
├── crates/
│   ├── chora-scene/   # scene model, commands, events, selectors, IPC types and client; no Smithay
│   ├── chora/         # the compositor: Smithay, backends, renderer, input, safety, IPC
│   └── choractl/      # CLI over the IPC socket; `choractl mcp` is the MCP bridge
├── plugins/
│   └── chora-tiling/  # reference plugin: master and stack layout, keybindings
├── dist/              # RPM build script and Fedora build requirements
├── deny.toml          # cargo-deny: allowed licenses, advisories
├── docs/              # this documentation (MkDocs)
├── mkdocs.yml
└── Cargo.toml         # workspace, lints
```

## Reasons for this split

- **`chora-scene` has no Smithay dependency.** Tests of the data model and
  the transaction logic need no GPU and no compositor. The core, `choractl`
  and plugins share this one definition of the API.
- **Reference plugins live in this repository** first, as workspace members.
  They keep the API honest. They can move out when the API is stable.
- **The build generates the schemas** from the Rust types. The repository
  does not contain them. Docs, validation and agents then see the same
  contract.

## Related repositories

| Repository | Role |
|------------|------|
| `secureblue-images` | derived image that installs chora as an additional session |
| `image-builder-action` | builds and signs that image |
| `distrobox-images` | container images for services |
