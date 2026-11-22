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

## r2

Brief `briefs/core-diag-r2.md`. Branch `claude/core-diag` from
`fc779a76`; step 0 (`git fetch --unshallow`, `cargo check --workspace`)
green. `origin/claude/autonomous-dev-session-3w-i33pg7` is still at
`3b14c825`, so the merge for the gate was a no-op: the gate ran on the
branch head.

### Commits

- `722d5baa` test(scene): the three red tests, all red on `fc779a76`:
  - `chora-scene` `tests::ten_thousand_creates_under_the_root_take_linear_time`:
    check + apply of 10000 `create rect` under root under 2 s, and under
    10 times the time of 2500 (50 ms floor). Red: 51.0 s.
  - `chora-scene` `tests::ten_thousand_removes_of_siblings_take_linear_time`:
    check + apply of 10000 `remove` of the children of one group under
    1 s. Red: 2.17 s.
  - `chora` `transaction::tests::a_big_transaction_answers_before_the_stall_line`:
    one `scene.apply` of the 10000 creates of `repro-sl3.sh` (under
    `MAX_LINE`, written in parts to the non-blocking test socket) answers
    under 3 s. Red: 56.0 s.
- `d845a7f5` fix(scene), then `788605f4` chore(core) (below):
  - `Scene::band` (scene.rs): two `partition_point` binary searches over
    the root children instead of two `take_while` walks. The doc comment
    says why the order holds: `link` puts a root child only inside its
    band, `set_order` unlinks and links again. Nothing else writes the
    root children (the injection mark goes under an overlay root).
  - `Scene::unlink`: `position` + `remove` instead of `retain`.
  - `crates/chora/src/main.rs`: first thing in `main()`,
    `libc::mallopt(libc::M_MMAP_THRESHOLD, 64 << 10)`, with the
    `// SAFETY:` line naming sl-1 (the no-inline-comments hook allows
    only that form). `unsafe_code` sits in the existing `#[allow(...)]` of
    `main` (`788605f4`): a second allow line raised tools/lint-ratchet to
    361 and failed ci-check in the first gate run.

All 160 `chora-scene` tests and all 649 `chora` lib tests green.
`scene.rs` 1272 lines, `main.rs` 18.

### Margins of the new tests (debug build)

| test | bound | idle (3 runs) | under the parallel lib suite |
|---|---|---|---|
| creates under root, 10000 | 2 s | 0.47 to 0.53 s | 0.45 s |
| creates, ratio 10000 / 2500 | 10 | 4.3 to 5.5 | 5.5 |
| removes of siblings, 10000 | 1 s | 0.15 to 0.18 s | 0.20 s |
| big transaction in the core | 3 s | 0.62 to 0.70 s | 0.77 s |

The absolute bounds leave 4 to 5 times margin. I raised the ratio bound
from 8 (the r1 suggestion) to 10 because a 5.5 was measured; quadratic
gives 16 to 17.

### Repros, before (r1, `3b14c825`) and after (`d845a7f5`)

- `tests/battle/repro-sl3.sh` (N=10000): 35.8 s, the watchdog aborted the
  core (RED) → GREEN:

      repro-sl3: 10000 creates in one scene.apply: 0.9 s (ok)
      repro-sl3: core alive, -- rss end: chora=172040 tiling= xp= mc=
      repro-sl3: GREEN

- `tests/battle/repro-sl1.sh`: 150 → 374 MB after the last close
  (RED by 224 MB) → GREEN:

      repro-sl1: cycle 1: all closed    rss= 172 MB anon=  58 ...
      repro-sl1: cycle 2: all closed    rss= 173 MB anon=  59 ...
      repro-sl1: cycle 3: all closed    rss= 178 MB anon=  65 ...
      repro-sl1: core RSS 152 MB at start, 178 MB after the last close (bound +60 MB)
      repro-sl1: GREEN

- `tests/battle/w9.sh`, one run: core RSS 153 MB at start, **288 MB at
  the end** (294776 kB; r1: 533 MB). Peak 476 MB after the settings and
  reload storms, back to 296 MB after D58. What stays is live: W9 ends with
  its windows open.

### Gate

`tests/merge-gate.sh --jobs 1` on `788605f4` (the integration branch
merged in, a no-op): **not green, one red cell I do not think I caused.**
red-first, identity, pre-commit, gate-lib, ci-check, build, test, clippy,
2d-guard, package-deps, package-lists, xp, matrix, virtual, xp-faces,
xp-paperclip, xp-fields, xp-question, xp-notepad, hud, layout,
tiling-audit, tiling-look, tiling-empty, chat-keys, doors PASS; **acts
FAIL** (1373 s, 232 pass, 5 known red, 2 blocking):

    FAIL: construct I6: in act 'co-42: the hint Keys of a face stays after 3 s and opens the key list by a click': on HEADLESS-1 'PANES' (construct-help-body-head-6) is 11 px, the guide wants 14 px
    FAIL: construct act:the hint Keys of a face names a key that opens the key list: in act 'the hint Keys of a face names a key that opens the key list': RuntimeError: the hint keys names no key

Both are on the construct face hint and key list. The second act reads
the timed face hint with `scene.query` (tests/acts/looks/construct.py:523)
and found no hint text: timing on this path was red before (co-keys r3:
"no the hint Keys on the screen within 10 s"; co-42). The first is the I6
breach that known-red.toml already lists for head-6 under the sibling act
("the hint Keys of a face names a key ..."). Here the key list was opened
by the co-42 act instead, so the row does not match. Neither touches root
order: every write to the root children goes through `link` (create,
attach, move, the surface return) or `set_order` (update), so the band is
the same as before. I did not rerun acts. The gate stops at the first
red, so I ran the cells after it once with `--only` (partial, nothing
pushed by it): last-words, cube, pull, placed, construct-hud,
construct-extra, out, winit, wm-pane, pane-keys, walk, keys, pixels,
mc-paths, mc-sofa, mc-column, mc-handover, launcher-click, voice-acp,
voice-events, agent, e2e **all PASS**.

The first gate run on `d845a7f5` failed ci-check:
`FAIL: lint-ratchet: allow is 361, the list holds 360`. That was mine,
fixed in `788605f4` without touching the list.

Head `788605f4`. No production file over 1500 lines (size-ratchet: 0
files).

## r3

F5 (FLK-TX-STALL): the sl-3 test measured wall time on the dispatching thread, so other test threads counted against the core.

- Base 788605f4. Red `50ca5899` "test(core): CHORA_TX_LOAD=1 turns sl-3 red on wall time" (test-only env switch spawns 4 busy threads per core).
  Red line: `sl-3 wall 3.027575339s` / `panicked at crates/chora/src/transaction.rs:1038:9: 3.027575339s` (an earlier run: 3.17 s).
- Repro (4 shell spinners, lib run, 5x, wall): 1.10, 1.46, 1.39, 1.59, 1.39 s (all green, bar 3 s).
- Green `93e9af7b`: the bar is thread CPU time (`clock_gettime(CLOCK_THREAD_CPUTIME_ID)`) < 3 s; wall < 60 s only as hang guard. No raised bar, no ignore.
- Follow-up `03e17809` (head): the lint ratchet counted my eprintln and a print_stderr allow; the test writes with `writeln!(stderr)`, the one `allow(unsafe_code, reason)` stays, list allow 361 (`ratchet-raise:` line). The first gate run was red on exactly this (allow 362 vs 360, eprintln 132 vs 131), caused by me, fixed.
- 10 runs of the lib test under 4 spinners + CHORA_TX_LOAD=1, 0 red. CPU / wall: 0.73/4.51, 0.72/4.47, 0.70/4.32, 0.70/3.97, 0.72/4.10, 0.71/4.24, 0.69/4.05, 0.76/4.50, 0.72/4.19, 0.68/3.98 s (the wall bar would have been red in all 10). The final head ran 10 more under load: CPU 0.69 to 0.77 s, 0 red.
- Gate: `tests/merge-gate.sh --jobs 2` (full) on claude/core-diag 03e17809 merged with fresh origin/claude/autonomous-dev-session-3w-i33pg7 (detached cb7fd55a): 57 cells PASS, EXIT 0, nothing pushed.

## PM review r3 (2026-11-22 19:34 UTC)
Range 788605f4..03e17809 (red 50ca5899, green 93e9af7b, 03e17809), 2 files +67/-4 (PM review, PM not the builder; test code only).
- FLK-TX-STALL: the sl-3 test now bars thread CPU time (clock_gettime CLOCK_THREAD_CPUTIME_ID) at 3 s, wall 60 s only as hang guard; comment says why. CHORA_TX_LOAD=1 spawns 4 per core spinners, dropped before the asserts. Red 3.03 s wall under load, green 10/10 at 0.68-0.77 s CPU. Matches the brief.
- lint-ratchet allow 360 -> 361: one `#[allow(unsafe_code, reason = ...)]` on the clock call. The brief asked for libc clock_gettime, so the raise is the PM's; accepted once. Backlog TX-CPU-SAFE: read /proc/thread-self/stat instead and lower allow back to 360.
- Gate: full --jobs 2, 57 PASS, EXIT 0 (lane).
Verdict (computed): 0 open MAJOR -> MERGE-OK. Joins t110 with 03e17809.
Amend 19:42: merged with mesh-churn dc6a266d (t109h) the ratchet is red (allow 362, list 361). CORE-DIAG leaves t110 and lands through core-fix r1 (base 46e98b3e = 03e17809 + dc6a266d), which also takes TX-CPU-SAFE.
