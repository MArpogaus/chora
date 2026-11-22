# core-diag

## r1

Brief `briefs/core-diag-r1.md`. Diagnosis only, no product change.
Branch `claude/core-diag` from `3b14c825`; head `fc779a76` adds
`tests/battle/repro-sl3.sh` and `tests/battle/repro-sl1.sh`. Step 0:
cargo, foot, python3, valgrind and gdb exist; heaptrack and perf do not.
`cargo check --workspace` and `cargo build --workspace` green. The debug
build here has line tables only (`~/.cargo/config.toml`), so locals were
read from a second core built with `CARGO_PROFILE_DEV_DEBUG=full` into
`target/dbg` (not committed). All times are the debug build, as in the
battle runs.

### sl-3: one transaction stalls the core until the watchdog aborts it

**Repro:** `tests/battle/repro-sl3.sh` (38 s, red). One session, no
plugin; the confined client box0 sends one `scene.apply` with 10000
`create rect` under `root`. Output on `3b14c825`:

    repro-sl3: 10000 creates in one scene.apply: 35.8 s ({'message': 'closed'})
    chora: the main loop stalls; ... (stalled 3 s)
    chora: the main loop hangs; the watchdog aborts chora with a core dump (stalled 30 s)
    repro-sl3: core dead: the core ended with 134
    repro-sl3: RED

`N=` sets the count, `MAX_S=` the bound (5 s);
`CHORA_HANG_ABORT_S=0` keeps the core alive to time the whole stall.

**Where the time goes:** 6 of 6 gdb samples of the main thread during
the stall (3 s apart) are in the same place:

    Scene::order_of           crates/chora-scene/src/scene.rs:943  (imbl OrdMap::get, str compare)
    Scene::band {closure}     crates/chora-scene/src/scene.rs:936  (take_while over root children)
    Scene::band               crates/chora-scene/src/scene.rs:937
    Scene::link               crates/chora-scene/src/scene.rs:905
    Scene::create             crates/chora-scene/src/scene/tree.rs:41
    Scene::apply_one          crates/chora-scene/src/scene/commands.rs:37
    Scene::run                crates/chora-scene/src/scene.rs:428
    Scene::check              crates/chora-scene/src/scene.rs:420

`band()` (scene.rs:922-940) finds the place of a new root child by its
`Order` (D89). It walks **all** root children twice (`take_while` for
`low` and for `high`) and looks up each sibling in the node map
(`order_of`, an `imbl::OrdMap` get with string compares). For N creates
under root that is about N² map lookups. The transaction runs the
commands twice on the main loop: `Scene::check` from
`State::start` (crates/chora/src/transaction.rs:286) and `Scene::apply`
from `State::finish` (transaction.rs:431).

**Measurement** (one scene.apply, `CHORA_HANG_ABORT_S=0`):

| creates | parent | time |
|---|---|---|
| 1250 | root | 0.8 s |
| 2500 | root | 3.3 s |
| 5000 | root | 13.6 s |
| 10000 | root | 56.9 s |
| 10000 | a group `g` | 0.5 s |
| 5000-chain of nested groups | root | 0.1 s (refused at command 64) |

Each doubling of N costs 4.1 to 4.2 times: **quadratic** CPU on the main
thread. Not a lock and not a blocking write (all samples are in user code
on thread 1). Under a group, `band()` returns `[0, len]` at once and
10000 creates take 0.5 s. The scene limit (`MAX_NODES` 16384,
scene.rs:179) lets one client send about 16000 creates (about 2.5 min
by this curve).

**What W12 saw:** the "5000-nested-groups step took 15.0 s" is not the
cost of that step. The chain is refused at command 64 by `MAX_DEPTH`
(scene.rs:184) in 0.1 s. The 15.0 s is the 15 s socket timeout of the
W12 client (`s.settimeout(15)`), spent waiting while the core was still
in `Scene::check` of the 10000-creates transaction sent before. The
watchdog aborts at 30 s, before the transaction ends (57 s).

**Next quadratic path, same pattern:** `Scene::unlink`
(scene.rs:961-968) runs `children.retain(|c| c != id)` over all siblings
for each remove. 10000 `remove` of the children of one group in one
transaction: 3.4 s (over the 3 s stall line). At 16384 nodes that is
about 9 s.

**Red test first** (crate `chora-scene`, `src/tests.rs`):

- `ten_thousand_creates_under_the_root_take_linear_time`: a
  `Scene::default()`, a transaction of 10000 `create rect` under `root`;
  `scene.check` plus `scene.apply` take under 2 s (debug), and the time
  of 10000 is under 8 times the time of 2500 (quadratic gives 16). Today:
  about 57 s, ratio 17.
- `ten_thousand_removes_of_siblings_take_linear_time`: 10000 children
  of one group, then one transaction of 10000 `remove`; under 1 s.
  Today: 3.4 s in the core.
- In crate `chora` (`transaction.rs` tests, the `Core` harness): `a_big_transaction_answers_before_the_stall_line`:
  10000 creates under root answer in under 3 s (`STALL_AFTER`,
  hang.rs:37).

**Fix (round 2), in `crates/chora-scene/src/scene.rs`:**

1. `band()` (922-940): replace the two `take_while(..).count()` with
   `siblings.partition_point(|s| self.order_of(s) < order)` and
   `partition_point(|s| self.order_of(s) <= order)`. `link()` and
   `set_order()` keep root children sorted by order, so the binary search
   holds: O(log N) lookups per create.
2. `unlink()` (961-968): `position` + `remove` instead of `retain`
   (stops at the first match), or keep an index of child positions.
3. Optional, `crates/chora/src/transaction.rs`: keep the scene that
   `check` built in `Pending` and swap it in at `finish` when nothing
   changed in between, so that the commands run once, not twice.

### sl-1: W9 core RSS +236 % that does not come back

**Repro:** `tests/battle/repro-sl1.sh` (38 s, red). chora-tiling on two
outputs; three times: open 12 probe windows one by one, then end them
all. RSS of the core with its split by mapping, after 4, 8, 12 windows
and after each close:

    repro-sl1: start                  rss= 150 MB anon=  40 ...
    repro-sl1: cycle 1: 4 windows     rss= 313 MB anon= 199 ...
    repro-sl1: cycle 1: 8 windows     rss= 360 MB anon= 246 ...
    repro-sl1: cycle 1: 12 windows    rss= 374 MB anon= 257 ...
    repro-sl1: cycle 1: all closed    rss= 360 MB anon= 247 ...
    repro-sl1: cycle 2: all closed    rss= 373 MB anon= 259 ...
    repro-sl1: cycle 3: all closed    rss= 374 MB anon= 260 ...
    repro-sl1: core RSS 150 MB at start, 374 MB after the last close (bound +60 MB)
    repro-sl1: RED

**The step:** W9 step 1, "open/close churn" (12 windows, kill 6, open
6). A 0.5 s RSS sampler on a full `w9.sh` run: 159 MB at start, 405 MB
2 s after the first windows open, 487 MB at the end of step 1, peak
646 MB after the 36 relayouts of step 2. It never goes back down to the
start. In the repro, cycles 2 and 3 add only 13 MB and 1 MB, so this is a
high-water mark, not a leak per window.

**What holds it:** no live structure. It is free memory that glibc keeps
in the main arena:

- The growth is all in `[heap]` (the brk heap of the main thread):
  238 MB resident of 265 MB after the close.
- `malloc_stats()` (via gdb) after the close: arena 0 `system bytes`
  278 MB, `in use bytes` 62 MB. `malloc_info`: 144 MB of it in 52
  free chunks of 0.5 to 20 MB inside the heap.
- `malloc_trim(0)` (via gdb) after the close: RSS 377 MB to 168 MB, the
  heap from 238 MB to 33 MB resident.

**Who allocates it:** the layer textures of `Layers`
(`crates/chora/src/mesh.rs:147`, `textures: HashMap<(String, usize, i32,
i32), Slot>`, keyed by node, slot **and size**). `Layers::slot`
(mesh.rs:347-368) makes them with `GlesRenderer::create_buffer`
(mesh.rs:358). On the headless backend that is llvmpipe, which allocates
the pixels with `posix_memalign` on the main thread.

- gdb breakpoint on `posix_memalign` of 64 KiB or more, one 12-window
  cycle: 1358 calls, 632 MB in total. Of these, **1270 calls (594 MB)
  come from `Layers::slot`** (from `Layers::draw` mesh.rs:415 and
  `Layers::texture` mesh.rs:381 in `run_passes` mesh.rs:1208), and 88
  calls (38 MB) from smithay's shm import. The sizes are 150 KB to 2.3 MB:
  the tile sizes of the windows, 51 distinct sizes.
- `mesh.rs:358` is hit **1280 times** per 12-window cycle, the same
  count with `look.animations` true or false.
- The full-debug core, gdb at `Layers::retain` (mesh.rs:190):
  **`textures` holds 120 entries (82 layer nodes) with 12 windows open**
  (RSS 400 MB). After the close it holds **0** (RSS 397 MB). Each tiling
  window has about 7 layer nodes (radius, border and shadow rects are
  layers, `Node::is_layer`, crates/chora-scene/src/node.rs:316).

So each relayout gives every window new tile sizes, and so new
textures of 150 KB to 2.3 MB, because the size is part of the key. The
old sizes are dropped once no frame used them for 2 s (`UNUSED_FOR`,
mesh.rs:54; the cap `MAX_BYTES` is 512 MB, mesh.rs:50). glibc serves these
blocks from the brk heap: its mmap threshold rises to the size of the
first freed mapped block. When they are freed, the holes stay resident
under the top of the heap. The count that grows is the peak of live
layer textures (120 with 12 windows, about 200 MB of llvmpipe memory).
The RSS keeps that peak after the count falls back to 0.

**Proof that the allocator holds it:** with
`GLIBC_TUNABLES=glibc.malloc.mmap_threshold=65536` (a fixed threshold,
so blocks of 64 KiB or more are mmapped and unmapped on free):

- repro-sl1 is **green**: 150 MB at start, 168, 175 and 176 MB after the
  three closes (370 MB while 12 windows are open, as before).
- Full `w9.sh`: end RSS **288 MB** instead of 533 MB. The rest is live:
  W9 ends with its windows still open, about 120 MB of layer textures.

The RSS growth is a symptom of llvmpipe (the headless and CI path), whose
texture memory is process heap. On a GPU the same churn (1280 texture
allocations for 12 windows) costs GPU allocations, not RSS.

**Red test first:**

- Battle level: `tests/battle/repro-sl1.sh` as it is (RSS after the last
  close at most 60 MB above the start); red today by 224 MB.
- Crate `chora`, `src/render/cost.rs` (the `Core` harness with
  `layers.sizes`): `a_layer_that_changes_size_keeps_one_texture_per_slot`.
  One rect with radius 4 is resized 10 times, one frame each.
  `layers.sizes(id).len()` is at most 2 after the last frame, and
  `textures_allocated` grows by at most 10. Today it keeps all 10 sizes
  until 2 s pass.

**Fix (round 2), 3-6 lines:**

1. `crates/chora/src/main.rs`, first thing in `main()`: `unsafe {
   libc::mallopt(libc::M_MMAP_THRESHOLD, 64 << 10) }` (libc is already a
   dependency). A fixed threshold turns off glibc's dynamic raise, so
   texture-sized blocks go back to the system on free. This turns
   repro-sl1 green (measured with the tunable above).
2. Optional, `crates/chora/src/mesh.rs` `Layers::retain` (189-197): when
   it dropped textures, call `libc::malloc_trim(0)` (measured: 377 MB
   to 168 MB). This catches holes left by smaller blocks.
3. Churn, in `Layers::slot` (mesh.rs:347-368): when a vacant key's
   `(node, slot)` holds a texture of another size that no view drew in
   this frame, drop it now (a resize replaces the texture) instead of
   keeping it `UNUSED_FOR`. Keep sizes that another view (another scale)
   still draws, the reason the key has a size.

### Files

- `tests/battle/repro-sl3.sh`: `N`, `MAX_S`; `CHORA_HANG_ABORT_S=0` to
  time the whole stall.
- `tests/battle/repro-sl1.sh`: `CYCLES`, `WINDOWS`, `MAX_MB`;
  `INSPECT=<program>` runs with the core pid before the end (gdb,
  smaps); `TILING_SETTINGS` is sent with `settings.set` first.
- Clean-up of a run that was cut off: `tests/battle/kill-session.sh
  repro-sl3 repro-sl1`.

Gate on `fc779a76`: `tests/merge-gate.sh --jobs 1 --only
pre-commit,ci-check`: PASS (pre-commit 84 s, ci-check 108 s).
