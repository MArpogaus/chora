# review mesh-churn (rc 9: sim-long sl-1)

## r1

Brief `briefs/review-mesh-churn-r1.md`. Branch `claude/mesh-churn` head
`dc6a266d`, base `96e5c804`, 3 files +109 -3. Read-only on code; one
round. Step 0: `git fetch --unshallow`, `cargo check --workspace` green.
No merge gate run (the lane gate was green, 49/49).

**Verdict: MERGE-OK.** 0 BLOCK, 0 MAJOR, 1 MEDIUM, 3 MINOR. The
Done-when lines of `briefs/mesh-churn-r1.md` hold: the red-first test is
green, the two-scale test is green, repro-sl1 numbers are pasted, the
gate is green (per the lane report, not rerun), no file over 1,500
production lines (mesh.rs 1191 in all).

### Findings

- mch-1, MEDIUM, crates/chora/src/mesh.rs:383: `drop_left` runs only when
  the new key is vacant, so the second view of the same scale (a layer on
  two outputs of one scale, or an output and its capture) finds the new
  size already made, never leaves the old size, and the old sizes stay
  `UNUSED_FOR` as before the fix. Confirmed with a scratch test (not
  committed): two 160x120 outputs at scale 1, a radius-4 rect across the
  seam, 10 resizes, one frame each: `sizes("r")` holds all 11 sizes. Not
  a regression (the pre-fix lifetime), and repro-sl1's tiling windows
  stay on one output each. Fix: run the leave when a view first joins a
  slot's users (the `None` arm of `Slot::use_in`), not only on a vacant
  key, and add the same-scale test.
- mch-2, MINOR, crates/chora/src/mesh.rs:532: the `unsafe` block of
  `trim_heap` has no `// SAFETY:` line (the crate's convention, gl.rs:32,
  43, 175); the reason on the allow is not one. Fix: add `// SAFETY:
  malloc_trim takes a pad size and only touches allocator state under its
  own locks; any thread may call it.`
- mch-3, MINOR, crates/chora/src/render/cost.rs:602: `made <= 2 * 10`
  where `made` is exactly 20 (checked with a temporary `assert_eq!`), and
  the red commit had `<= 10`, changed in the fix commit. The bound is
  exact, not loosened to hide churn, but `<=` would not see fewer slots
  either. Fix: `assert_eq!(made, 2 * 10)`.
- mch-4, MINOR, crates/chora/src/mesh.rs:530 with core-diag
  `crates/chora/src/main.rs` (788605f4): two unsafe glibc calls in two
  places with two cfg stories; core-diag's `mallopt` has no
  `cfg(target_env = "gnu")` gate, so a musl build of the merge would not
  find `libc::mallopt`/`M_MMAP_THRESHOLD`. Fix: one `heap` module with
  `tune()` (mallopt at start) and `trim()` behind one allow and one cfg
  (a follow-up after both merge; the ratchet stays 361).

### 1) drop_left (Layers::slot)

`cargo test -p chora --lib mesh::`: 4 passed. `cargo test -p chora --lib
render::`: 123 passed (includes both new tests).

- Two views, two scales: each view keeps its own size; the new test
  covers it and is green.
- Same `(node, slot)` at two sizes in one frame of a view: kept, since
  `drop_left` keeps a user whose frame is the current one (`*at ==
  frame`).
- A view that skips frames: its old frame number differs from the
  current one, so it leaves the old size on its next new size; correct.
- Two views of the same scale: the gap of mch-1.
- Use after drop: none. `GlesTexture` is reference counted, the elements
  of the frame hold clones, and `Passed` (mesh.rs:139) keeps a slot and
  a version, no texture; a dropped slot makes `existing()` miss and the
  passes rerun. `versions` is one counter for all slots, so a remade
  slot never repeats a version.
- Frame counter: u64, one step per `set_view` (once per view per frame,
  render.rs:416; virtual outputs call it too); it cannot wrap. `users`
  holds one entry per view name (an output, or its capture), so it is
  bounded by the outputs; the entry of a removed output or an ended
  capture lingers on a slot until it is dropped, which can keep one old
  size `UNUSED_FOR` after the next resize (bounded, not a finding).
- Cost: `drop_left` walks all textures once per new key; with 120
  textures and about 160 new keys per relayout that is about 20k cheap
  steps, no finding.

### 2) trim_heap

- Called from `Layers::retain` only when `drop_left` set `dropped` or
  the retain/cap removed textures; `dropped` is taken, so at most once
  per retain (retain runs once per view, render.rs:359, before
  `begin_view`; a drop in one view's frame trims at the start of the
  next view).
- `cfg(all(target_os = "linux", target_env = "gnu"))` is inside the fn,
  so musl and other targets build an empty fn; `libc` is a plain
  dependency of `chora` (Cargo.toml:7). Only x86_64-unknown-linux-gnu is
  installed here, so a musl build was not tried.
- The allow is local to the fn and has a reason; no SAFETY line (mch-2).
  The call itself is sound: no pointers, any thread.
- Worst case per frame: a resize drag of a large layer under llvmpipe
  drops two slot textures each frame, so `retain` trims each frame. A
  C microbench (glibc, dynamic threshold raised, two 1920x1080x4
  textures freed and remade a little larger each frame, 60 frames, 3
  runs): the `malloc_trim(0)` call costs 0.1 to 3 us, but the pages it
  gives back are faulted in again by the next textures: alloc and touch
  6.4 to 6.9 ms per frame without trim, 6.5 to 8.2 ms with it, so about
  +0.3 to +1.5 ms per frame for a 1080p layer, more toward a 30 MB layer
  (above 32 MB glibc maps the block anyway). On a GPU the textures are
  not heap and the call costs microseconds. With core-diag's fixed 64 KiB
  threshold every texture is mapped and unmapped on free, so the trim
  adds nearly nothing for textures and only catches small holes.

### 3) Red first

`ed1bc1b2` holds only the test (cost.rs +23); checked out, it fails:
`old sizes stay: [(20, 20), (21, 21), ..., (30, 30)]`. `0d9e7de8` holds
the fix, the bound change and the two-scale test. The bound 2 x 10 is
exact: `made` is 20 with the fix (a temporary `assert_eq!(made, 20)`
passed), the same as before the fix (the fix changes how long old sizes
live, not how many are made), so the line is a ceiling against thrash,
not the red assertion (mch-3).

### 4) Ratchet

`dc6a266d` raises `allow` 360 to 361 with a `ratchet-raise:` line in its
message. The brief's premise does not hold: core-diag `788605f4` ("one
allow on main, the lint ratchet stays at 360") folds `unsafe_code` into
the existing allow on `main` and does not change `tools/lint-ratchet.list`.
A trial merge of `dc6a266d` with `origin/claude/core-diag` merges clean,
the list says 361, and `tools/lint-ratchet` prints `ok: lint-ratchet
(allow 361, no-reason 0, eprintln 131)`. The merged value both need is
**361**, not 362. One shared helper would be simpler (mch-4).

### 5) sl-1 after the fix

The lane measured 243 MB after the last close (+89 MB over 154 MB,
anon 128 MB), red by 29 MB; the peak with 12 windows fell from 376 to
266 MB. core-diag measured repro-sl1 green with the tunable alone
(`GLIBC_TUNABLES=glibc.malloc.mmap_threshold=65536`: 176 MB after the
last close, +26 MB), and its `mallopt` in `main` sets the same fixed
threshold. The rest is the brk-heap holes that blocks under the raised
dynamic threshold leave; a fixed threshold stops the raise for all
arenas, so the rest is expected to close with both merged.

The sim-long recheck must measure, on the train with both merged (same
debug build as the reports): repro-sl1 RSS after the last close against
start + 60 MB, with the anon split; the peak with 12 windows (expect at
most about 266 MB); the full `w9.sh` end RSS against core-diag's 288 MB;
the frame time (p99) during a relayout or a resize drag, since every new
size is now an mmap, a munmap and fresh page faults; and, if the stat is
added, textures made per 12-window cycle (unchanged by this fix, about
1280 create_buffer calls).

Known, not findings: the hint-click act flake (lane flake-hint); co-41,
co-42.
