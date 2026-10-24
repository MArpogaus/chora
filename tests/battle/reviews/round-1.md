Review complete — I read the full `v0.5.4..HEAD` diff of the core (`security`, `grants`, `ipc`, `ipc/clients`, `input`, `transaction`, `state`, `images`, `uploads`, `safety`, `render`, `output`, `headless`, `protocols`, `handlers`, `methods`, `process`, `time`, scene crate) and checked `docs/api.md` and `architecture.md` against the behaviour, plus targeted passes over the plugins (`sh -c` sites, ownership, restart) and the trust boundaries (admin vs. client sockets, grants, path/asset scoping, connection caps).

Findings:

**1. `crates/chora/src/backend/headless.rs:160` (`add`) + `crates/chora/src/backend/headless.rs:212` (`ipc_headless_output`) — `endangers`**
`output.add` has no cap on the number or total size of virtual outputs and needs only `hello` — not the main socket, not a grant — so any named client, including one on a confined client socket, can exhaust memory of a headless session.
Repro: run `chora --headless 640x480`, connect to `ipc.sock`, send `hello`, then repeatedly send `{"method":"output.add","params":{"spec":"16384x16384"}}`. Each spec passes `parse_specs` (16384 is the allowed side limit, `output.rs:29`), and the frame clock allocates a ~1 GiB ABGR texture per output at `headless.rs:324` → the core is OOM-killed after a handful of requests. A bound on the number of IPC-added outputs (or total buffer bytes) fixes it.
fixed in 683145f

**2. `crates/chora/src/protocols.rs:43-46` (`State::request`) — `endangers` (pre-existing: unchanged since before v0.5.4, flagging because the range ships it)**
The `early_requests` queue per surface is unbounded: requests that arrive before the first commit are pushed into a `Vec` with no length limit, and cleanup only happens at the first commit (`send_early_requests`) or surface destruction.
Repro: connect to any Wayland socket, create and `map` an `xdg_toplevel`, never commit a buffer, and call `set_fullscreen` (needs no serial) in a loop — `self.scene.node(&surface)` stays `None`, so `early_requests` grows without bound until the core swaps or OOMs. A cap per surface (drop-or-merge beyond N) fixes it.
fixed in 6d492ab

Everything else in range checked out: the admin-socket enforcement of grants/`clients.create`, name validation (no traversal into the `clients/` tree), 0600/0700 socket and dir modes, per-socket and per-process connection caps (16/16 + 32 kept, matching the docs), asset path ownership (`uploads::owns`), the `O_NONBLOCK`+regular-file guard on client-named image/shader files, the image worker's SIGUSR1 masking, transaction ordering/barriers/`MAX_PENDING` and drop-on-close surface release, press-scoped `focus_node` from client sockets, watchdog blame→disconnect (and the doc text already matches the new behaviour), and the scale-corrected `Under::location` fix. No breaks found.
