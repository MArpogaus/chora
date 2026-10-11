# Project structure

The planned layout of this repository: one Cargo workspace, small crates with
clear boundaries. Nothing here exists yet except `docs/`.

```text
void/
├── crates/
│   ├── void-scene/       # scene model, commands, events, transactions, selectors (no Smithay)
│   ├── void-core/        # the compositor binary `void`: Smithay, backends, renderer, input, safety
│   ├── void-plugin-host/ # wasmtime host: loading, chain, budgets, rollback
│   ├── void-ipc/         # JSON-RPC protocol types + client library
│   ├── void-pdk/         # plugin development kit (Rust); WIT bindings for other languages
│   ├── voidctl/          # CLI: query, apply, views, plugins, tokens, services
│   └── void-mcp/         # MCP bridge over void-ipc
├── plugins/              # reference plugins: tiling, stacking, keybindings, animations
├── flavors/              # reference flavors: classic, tiling, agentic
├── schemas/              # JSON Schemas for views, flavors, manifests (validation + agent docs)
├── dist/                 # void.desktop session file, packaging, image integration
├── docs/                 # this documentation (MkDocs)
├── mkdocs.yml
└── Cargo.toml            # workspace
```

## Why this split

- **`void-scene` is pure.** The data model and transaction logic are testable
  without a GPU or a compositor, and shared by core, plugins (via the PDK),
  `voidctl` and the MCP bridge — one definition of the API.
- **MCP lives outside the core.** `void-mcp` is just another IPC client.
- **Reference plugins live in-tree** at first, to keep the API honest; they
  can move out once the API is stable.
- **Schemas are generated** from the Rust types, so docs, validation and
  agents always see the same contract.

## Related repositories

| Repository | Role |
|------------|------|
| `secureblue-images` | derived image that installs Void as an additional session |
| `image-builder-action` | builds and signs that image |
| `distrobox-images` | container images for services |
