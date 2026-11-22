# mesh-churn

## r1

Brief `briefs/mesh-churn-r1.md` (rc 9, sim-long sl-1, layer texture
churn). Branch `claude/mesh-churn` from `96e5c804` (split-c head); head
`dc6a266d`. The train head `3b14c825` is an ancestor of the branch, so
the merge with `origin/claude/autonomous-dev-session-3w-i33pg7` is the
branch itself. Step 0: `git fetch --unshallow` and `cargo check
--workspace` green.

### Commits

- `ed1bc1b2` test(mesh): a resized layer keeps one texture per slot
  (red). `render/cost.rs`
  `a_layer_that_changes_size_keeps_one_texture_per_slot`: one rect with
  radius 4, resized 10 times (21 to 30), one frame each. Red on
  `96e5c804`:

      old sizes stay: [(20, 20), (21, 21), ..., (29, 29), (30, 30)]

- `0d9e7de8` fix(mesh): drop the old size of a layer texture when a
  view resizes. Green: `sizes("r")` holds one size after the last frame.
- `dc6a266d` chore(mesh): count the unsafe allow of trim_heap in the
  lint ratchet (`tools/lint-ratchet.list` allow 360 to 361, with a
  `ratchet-raise:` line; outside the brief's area, but the gate's
  ci-check fails without it, see Gate).

### Fix

- `Slot` keeps the views that use it, each with the frame of its last
  use; `Layers::set_view` counts a frame per view that begins.
- `Layers::slot`, on a vacant key: `drop_left` removes the current view
  from the textures of other sizes of the same `(node, slot)` that it
  used in an earlier frame (not this frame); a texture that no view uses
  then goes at once instead of after `UNUSED_FOR`. A view of another
  scale stays a user of its own size, so that size is kept. A use of two
  sizes of one slot in the same frame of a view drops nothing.
- `Layers::retain` calls `libc::malloc_trim(0)` once (`trim_heap`, its
  doc comment names sl-1) when textures went since the last retain, by
  `drop_left`, by `UNUSED_FOR`, by a gone node or by the cap. glibc only
  (`cfg(target_env = "gnu")`).
- New test `a_resized_layer_on_two_outputs_of_other_scales_keeps_both_sizes`:
  outputs at scale 1 and 2, the rect resized 3 times; `sizes("r")` is
  `[(43, 43), (86, 86)]`, and 3 more frames allocate no texture and run
  no pass (a fix that drops the other view's size thrashes here). Green.

**Deviation from the brief:** the red test bounds `textures_allocated`
at 2 x 10, not 10. A rounded rect draws two slots per size (slot 0, its
content, and slot 1, the pass of its radius), so each new size makes two
textures with or without the fix; the fix changes how long the old ones
live, not how many are made. The create_buffer count per 12-window
cycle was not measured (`textures_allocated` is not in `render.stats`);
it stays about the same: every new tile size still makes its textures.

### repro-sl1

`tests/battle/repro-sl1.sh` from `claude/core-diag` (copied in for the
run, not committed), debug build, all three on this container:

    96e5c804 (before):
    repro-sl1: cycle 1: 12 windows    rss= 372 MB anon= 258 ...
    repro-sl1: cycle 3: 12 windows    rss= 376 MB anon= 262 ...
    repro-sl1: cycle 3: all closed    rss= 373 MB anon= 262 ...
    repro-sl1: core RSS 150 MB at start, 373 MB after the last close (bound +60 MB)
    repro-sl1: RED

    0d9e7de8 without malloc_trim (trim_heap a no-op):
    repro-sl1: cycle 1: 12 windows    rss= 277 MB anon= 164 ...
    repro-sl1: cycle 3: 12 windows    rss= 299 MB anon= 186 ...
    repro-sl1: cycle 3: all closed    rss= 273 MB anon= 163 ...
    repro-sl1: core RSS 150 MB at start, 273 MB after the last close (bound +60 MB)
    repro-sl1: RED

    0d9e7de8 (with malloc_trim):
    repro-sl1: cycle 1: 12 windows    rss= 263 MB anon= 146 ...
    repro-sl1: cycle 3: 12 windows    rss= 266 MB anon= 148 ...
    repro-sl1: cycle 3: all closed    rss= 243 MB anon= 128 ...
    repro-sl1: core RSS 154 MB at start, 243 MB after the last close (bound +60 MB)
    repro-sl1: RED

The peak with 12 windows falls from 376 to 266 MB and the RSS after the
last close from 373 to 243 MB (+89 MB over the start, bound +60 MB).
Still red by about 29 MB; the `mallopt(M_MMAP_THRESHOLD)` of lane
core-diag in `main.rs` (measured green there with the tunable) is the
remaining step.

### Gate

`tests/merge-gate.sh --jobs 1` (all steps, no `--only`) on `dc6a266d`
(= the branch merged with `origin/claude/autonomous-dev-session-3w-i33pg7`
at `3b14c825`): **green**, 49 steps PASS, run `/tmp/merge-gate.fB2vVA`,
`all steps passed; no --push, nothing is pushed`. Long steps: layout
1356 s, acts 1147 s, tiling-look 676 s, construct-hud 562 s.

The first run on `0d9e7de8` stopped at ci-check, caused by this lane:
`FAIL: lint-ratchet: allow is 361, the list holds 360` (the
`#[allow(unsafe_code)]` of `trim_heap`); fixed by `dc6a266d`.

### Size

`crates/chora/src/mesh.rs` 1191 lines in all (production and tests),
`mesh/passes.rs` 341; no file over 1,500 production lines.
