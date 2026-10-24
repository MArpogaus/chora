## Review findings — `v0.5.4..HEAD`

**`crates/choractl/src/main.rs:593` — endangers** — `clients up` aborts at the first name whose `clients.create` fails with anything but `socket_exists`, so one name that is already connected on the main socket (or in its 10 s grace time) silently prevents sockets and grant-sync for every remaining name, contradicting docs/api.md:311 ("it opens the socket of **every name** and makes its grants equal to the list").

Repro:
1. Start any client that does `hello {"name": "beta"}` on the main `CHORA_SOCKET` (no client socket), or kill it and stay within the 10 s grace.
2. Put two names in `clients.yaml`, e.g. `alpha: {grants: [screen.capture]}` before `beta: {}` (names are processed in `BTreeMap` order).
3. Run `choractl clients up` → it exits with "beta is connected or in its grace time"; `alpha` gets neither a socket nor synced grants, and the run is not repeatable until the grace passes. The tolerable error class should be handled like `socket_exists` (keep going), or the docs promise dropped.

fixed in 77db138

---

Everything else in scope checked out:
- Core (ipc, clients/`confine`/`check_paths`, transactions, grants, safety, process, watchdog, headless, capture, mesh premultiply, `O_NOFOLLOW`/`O_NONBLOCK` file opens): all new limits (`MAX_LINE` 1 MiB, 128/16/32 connections, 512 KiB/64/64 MiB assets, 32-key meta, 150 ms txn deadline, `MAX_EARLY_REQUESTS`) match the docs; permission checks and reservation semantics (`remove` with `to`, grace, `unreserve`) match api.md §events.
- Scene (`clone-and-swap` apply, `remove_subtree`, `manager_of_new` incl. NaN pointer handling), input routing, and tiling's `ShownFocus` logic have matching tests and no reachable panic path found.
- Plugins: `chora-tiling` diff and the `chora-xp` restart/trash/decoration paths — `unwrap`/`expect` occurrences are test-only or on self-registered data; trash naming, claim boxes and focus takeover are tested.
- Docs spot-checks (limits, events, admin socket, method-call timeout 10 s, `give`, capture size) agree with the code.
