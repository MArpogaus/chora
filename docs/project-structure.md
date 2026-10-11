# Project structure

The planned layout of this repository: one Cargo workspace with as few crates
as possible. A module moves into its own crate only when a second consumer
needs it. Only `docs/` exists today.

```text
void/
├── crates/
│   ├── void-scene/   # scene model, commands, events, transactions, selectors; no Smithay
│   ├── void/         # the compositor: Smithay, backends, renderer, input, safety, IPC, plugin host
│   └── voidctl/      # CLI over the IPC socket; `voidctl mcp` is the MCP bridge
├── plugins/          # reference plugins: tiling, stacking, keybindings
├── dist/             # void.desktop session file, packaging
├── docs/             # this documentation (MkDocs)
├── mkdocs.yml
└── Cargo.toml        # workspace, lints
```

## Reasons for this split

- **`void-scene` has no Smithay dependency.** Tests of the data model and the
  transaction logic need no GPU and no compositor. The core, `voidctl` and
  plugins share this one definition of the API.
- **MCP stays outside the core.** The MCP bridge is a subcommand of `voidctl`
  and talks to the core like every other IPC client.
- **Reference plugins live in this repository** first. They keep the API
  honest. They can move out when the API is stable.
- **Schemas are generated** from the Rust types, not committed. Docs,
  validation and agents then see the same contract.

## Related repositories

| Repository | Role |
|------------|------|
| `secureblue-images` | derived image that installs Void as an additional session |
| `image-builder-action` | builds and signs that image |
| `distrobox-images` | container images for services |
