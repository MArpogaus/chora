I've completed the review across all chunks: security/grants, IPC + client sockets, transactions, input routing/grabs/repeat, scene graph, headless backend, tiling, `chora-kit`/`choractl`, the high-risk parts of the XP plugin (trash/replace/copy semantics, shell quoting of spawned commands, desktop-icon drag logic), the docs (`api.md` limits cross-checked against constants: meta limits, `MAX_LINE`/`MAX_OUTPUT`/connection caps, image size caps, repeat rate, grace time, admin-only methods, `Remove{to}` liveness check, safety-layer grab refusal, `focus_node` ownership via `owned_mut`, overlay-root spoofing impossible since only per-output `screen:<name>` roots render), and CI/examples.

Notable points that turned out to be correct by design:
- `clients.create` path safety: name charset + 0700 dirs make the unset socket mode unreachable.
- `pending_activations`/`idle_inhibitors` growth is resource-object-bound; the one truly unbounded queue (`early_requests`) was fixed in `6d492ab`.
- `pressed_by()` consumes the one-shot press even when the transaction later fails — matches the documented "press counts once" anti-replay semantics and only ever wastes the caller's own press.
- `sh -c` in `chora-xp::spawn` receives single-quote-escaped paths (`quote()` handles `'`), and Exec lines are user-owned files — same-user threat model as the admin socket itself.
- Image decode worker: budget re-checked at insert, `MemoryBuffer::from_slice` copies, SIGUSR1 blocked, no transaction-queue deadlock.

**CLEAN**

not fixed: no findings listed. Verdict is CLEAN, written as bold **CLEAN**, which the loop's `grep -qx CLEAN` misses. Fix step spot-checked the cited-by-design points (quote() handles ', early_requests cap in 6d492ab) and changed nothing.
