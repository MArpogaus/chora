# Review: core-diag r2 (rc 9: sl-3, sl-1), claude/core-diag at 788605f4

Reviewer lane: opus, read-only on code. Brief: briefs/review-core-diag-r2.md. Diff:
`git diff fc779a76..788605f4` (3 commits, 4 files, +108 -12). Read: briefs/core-diag-r2.md,
reports/core-diag.md "## r1" and "## r2". No merge gate run.

**Verdict: MERGE-OK.** 0 BLOCK, 0 MAJOR, 1 MINOR, 2 NIT. The Done-when line "full gate green" fails
(acts red), but check 5 shows the red is foreign to this diff.

Step 0: `git fetch --unshallow`, `cargo check --workspace` on 788605f4 green (1 m 27 s).

## Checks

1. **Scene::band** (crates/chora-scene/src/scene.rs:925-936): same band as the old `take_while` walks,
   because the root children are always sorted by `Order`. `partition_point` equals the `take_while`
   count on any slice that is partitioned by the predicate, and a slice sorted by order is. Every writer of
   the root children keeps that order:
   - `link` (scene.rs:904) inserts at `index.clamp(low, high)` or at `high`, always inside the band. Its
     callers are `create` (tree.rs:41), `attach` (tree.rs:64-65, unlink then link), `move` (tree.rs:93-94,
     unlink then link), `place_free` (scene.rs:492) and `set_order` (scene.rs:953-954, unlink then link
     after the new order is set).
   - `set_order` is the only place that changes the order of a linked node: `update` calls it
     (scene.rs:753) before `props.apply`, and an animation takes `from` after it (scene.rs:761), so
     `advance` never writes a new order. Every update drops the running animation first (scene.rs:752).
   - The reset (tree.rs:187-204) sets the root children to the surfaces and every props to default:
     all Normal, sorted.
   - `place_free` links a surface with parent None: such a surface has default props (detach resets
     them, scene.rs:893), so it links as Normal and stays Normal after the reset of its props.
   - Other writes of `children`: `stack.append(&mut node.children)` in detach (scene.rs:890, a removed
     subtree), the overlay clear (tree.rs:182) and the injection push (scene.rs:562): neither is the
     root. `nodes` is private; no code in crates/chora or plugins writes `children` (grep).
   `cargo test -p chora-scene`: 160 passed (0.50 s).
2. **Scene::unlink** (scene.rs:958-967): `Vec::remove` shifts the tail, so sibling order stays. A node
   with no parent returns as before; a node missing from its parent's list is a no-op as `retain` was.
   The one difference, only the first of several equal ids goes, cannot happen: every `link` follows an
   `unlink` or a new id, and `show_injection` calls `remove_injection` (an unlink) before its push.
3. **mallopt** (crates/chora/src/main.rs:10-12): the first statement of `main`; no `ctor` crate in
   Cargo.lock and std starts no thread before `main`, so the SAFETY line is true. `unsafe_code` sits in
   the `#[allow]` of `main` only (not crate level); the other unsafe allows in crates/chora are unchanged.
   libc is a direct dependency; no non-glibc target in the repository (no musl/BSD in CI or packaging).
   `cargo test -p chora`: 649 lib tests passed (14.4 s), `a_big_transaction_answers_before_the_stall_line` ok.
4. **Red first:** `git log --oneline fc779a76..788605f4`: 722d5baa (tests only: tests.rs +55,
   transaction.rs +36), then d845a7f5 (fix), then 788605f4 (allow). d845a7f5 also loosens the ratio
   bound of the create test from 8 to 10; still red before the fix (r1 ratio 17, and 51 s against the
   2 s bound). Margins from the r2 report: absolute bounds 4 to 5 times (0.45-0.53 s of 2 s, 0.15-0.20 s
   of 1 s, 0.62-0.77 s of 3 s). The ratio bound has only 1.8 times (5.5 measured, bound 10): cd-1.
5. **Acts red: could this diff cause it? No.** By checks 1 and 2 the diff changes no scene result for
   any reachable scene, only its cost: the band and the unlink give the same index as before. The
   construct plugin sets no `Order` (no `Order::` or `order:` in plugins/chora-construct/src), and its hint
   nodes sit under groups (`parent: crate::NAME`, `HITS`, `HINTS` in hints.rs), where `band` returns
   `[0, len]` on both sides of the diff. The failing act reads the keycap texts
   `construct-hints-keys-keys-<k>-text` once (tests/acts/looks/construct.py:511-523, `press_hinted`)
   right after `hint_shown` waited only for the rect of `construct-hint-keys` (construct.py:529-533):
   a wait race in the act when the keycap texts land a moment after the hint rect. The I6 head-6
   11 px breach is the key list layout that known-red.toml already lists under the sibling act. The
   mallopt changes allocation latency only, not scene content; it can shift timing but does not create
   the race. Consistent with the suspected flake (ldm-2, lm-2, lane flake-hint r1).
6. **Ratchets:** `tools/lint-ratchet`: ok (allow 360, no-reason 0, eprintln 131); the list is untouched
   by the diff. `tools/size-ratchet`: ok (0 files over 1500); scene.rs 1272 lines.

## Findings (id | severity | file:line | text | fix)

- cd-1 | MINOR | crates/chora-scene/src/tests.rs:2051 | The ratio assertion `big < 10 * floor` has 1.8
  times margin (5.5 measured under the parallel suite) and compares two runs at different moments, so a
  load spike during the 10000 run on a busy debug host fails it, while the 2 s absolute bound already
  catches the quadratic case (57 s) with 4 times margin | drop the ratio assertion, or take the smaller
  of three runs of each size before dividing.
- cd-2 | NIT | crates/chora/src/main.rs:12 | A fixed 64 KiB threshold makes every block of 64 KiB or more an
  mmap and munmap with fresh zeroed pages (shm import, frame buffers), and it overrides a user's
  `GLIBC_TUNABLES` mmap threshold; frame time was not measured with it | measure the frame time of one
  battle run (w9 or a pixels cell) with and without the call, and note it in the report.
- cd-3 | NIT | crates/chora-scene/src/scene.rs:921 | The sorted root band is now an invariant that
  `partition_point` relies on, stated in a comment only; a future writer of the root children that
  skips `link` would misplace nodes silently | add a scene test that runs create, move, attach,
  set_order up and down, place_free and reset, and asserts the root children are sorted by order after each.
