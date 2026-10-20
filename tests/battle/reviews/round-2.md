**No findings to report.**

After deep line-by-line review of the security-critical and logic-dense paths in `v0.5.4..HEAD`, I could not confirm a single real bug. Everything I traced was explicitly guarded. What I verified, and the failure modes I checked and disproved:

**Core / IPC (`crates/chora/src`)**
- **Client sockets (D33)** — `private_dir` creates `clients/NAME` with `0700` and rejects a planted symlink (`symlink_metadata` + uid check, ipc.rs:201-215). `confine_one()` blocks Create/Move onto foreign parents, Create of surfaces, and Remove/Configure/FocusNode of foreign nodes. Name is validated to `[a-z0-9-]`, socket-path length capped (107).
- **Grants** — every privileged method is capability-gated; `grants.rs` carries a test asserting each method requires its capability; revoke disconnects clients holding it.
- **Connection caps** — total (128), per client-socket (16), kept-for-main (32), per process — all enforced at accept (ipc.rs:461-483).
- **Lock screen** — generation counter prevents a pre-lock frame from confirming a newer lock; 1 s confirm deadline can't leak into a later lock (timer token removed on state change); lock surfaces follow output scale/size changes; grabs are dropped on lock/unlock.
- **Transactions** — 150 ms deadline, 16 pending per connection, the commit-time re-check (2c2125e) prevents applying mutations to foreign/removed nodes; `drop_conn` releases held reservations.
- **`input.inject`/capture** — grant + lock checks, caps (256 events, 4 KiB text, 512 KiB PNG file, 64 exports), keys/buttons forced released at the end; `Pixels::crop` bounds-checked.
- **Headless backend** — output count cap (683145f), specs parsed with `u16` bounds checks; `output.add/remove` and `input.inject` refuse non-virtual inputs.

**Scene (`chora-scene`)**
- `Move` rejects cycles (`SceneError::Cycle`, scene.rs:579) and requires a group parent, so the `Remove{id, to}` branch genuinely cannot hide nested surfaces (I built that repro and it fails the `group(&parent)?` check).
- `unreserve` runs on all three paths (re-reserve, client disconnect, keep-nodes disconnect, ipc.rs:1013), so a `to`-reservation can't strand a surface.
- `scene.apply` verifies the whole batch before mutating; late image loads re-check existence and ownership at commit (transaction.rs:686-700).

**XP plugin (sampled deeply: wm input routing, folder, restart, decoration, explorer, session)**
- Shell injection: every `spawn` caller either passes fixed strings or quotes paths with proper `'\''` escaping (desktop_icons.rs:889).
- Node-id parsing uses exact `strip_prefix`/`split_once` forms (`Part::of`), so `s1`/`s12`-style prefix collisions can't occur; item indices go through `nth()`, not raw indexing.
- Recycle Bin writes `.trashinfo` with `create_new` and removes it again if the rename fails (folder.rs:544-547).
- Restart hand-off: windows the new instance doesn't reclaim are removed; reserved surfaces are adopted, never double-owned.

**Docs vs. code** — the `docs/api.md` method table matches the IPC dispatcher exactly (I diffed both lists; the earlier `input.get`/`window.grab` suspicions were wrong), the MCP tool list matches `choractl/src/mcp.rs`, and the security notes (grant list, press-token rule, spawn-socket gap documented as a known 0.7 gap) match the implementation.

Caveat for the record: core, scene, IPC, transactions, security, and the XP input/security paths were reviewed line-by-line; the XP plugin's pure drawing/layout internals (~10k lines) were reviewed by scanning and targeted checks, leaning on its passing test suite.
