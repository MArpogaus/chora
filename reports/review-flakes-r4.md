# Review: flakes r4 (claude/flakes-r4, ce8f7928 on d349a6da)

Read-only. Diff, brief, report, kit/core grab code (chora-kit runtime.rs:268-345, chora ipc.rs:1428-1486).
Ran at HEAD: `chora --lib words::` (10 ok), xp `cube_keys`, tiling `kept`, mc `bindings_that_stay` (ok).
Red-first checked by `git show`: 915f97b1, 0a3955d2, bdeb13b2, 02360013 each come before their fix.

## F17 (words alarm, D state)
Sound. A cold-cache backtrace faulting in debug info sits in `D`; futex/lock waits are `S`, so the
lock-in-malloc case the alarm exists for still ends the core. The bound remains: `goes_on` keeps
`ran_ms < GIVE_UP_AT_LAST` (60 s, words.rs:57,190), so a handler stuck in `D` forever (hung disk/NFS)
still ends at 60 s, the same ceiling an `R` spinner already had. SIGALRM is process-directed
(ITIMER_REAL, words.rs:254), so another thread runs `stuck`. Cost: 60 s, not 2 s, on a hung disk; OK.

## GR1 (start_grabs / end_grabs)
Correct, including bindings in both sets. In the kit, `start_grabs` moves the held strings to `old_grabs`
(it keeps give keys while `give_keys` is set). `grab` of a string in `old_grabs` moves it back with
the new action and makes **no** core call (runtime.rs:274-277). `end_grabs` ungrabs only what is still
in `old_grabs`. So a binding in both sets is never ungrabbed and never re-grabbed. That also avoids the
core's "is grabbed already" refusal of its own grab (ipc.rs:1457-1462); peak |old ∪ new| << 256.
- Error path: if `grab_all` fails (bad key in settings, INVALID_PARAMS), `end_grabs` is skipped. The
  kit's `apply` (runtime.rs:484) undoes the change and calls `settings_changed` again. The next
  `start_grabs` extends `old_grabs`, and the closing `end_grabs` drops the leftovers. This converges and
  is better than before (no window with no bindings).
- tiling `grab_all` checks `rt.action(..).is_none()` before the Swallow grabs (main.rs:581). It runs
  after `start_grabs`, finds nothing, and so reclaims from `old_grabs`. Correct.
- xp `sync_cube_keys` no longer calls `self.grab()`. `accept_injected`/`give_keys` were already set by
  the startup `grab()` (wm.rs:544), which always runs first. Equivalent.
- construct scene.rs:185 (`leave`): agreed, no second set to keep.

## Cube gate
- Start card (gate-construct-cube.sh:639-641): the card is built in the same `try_apply` as the cube
  (cube.rs:1183-1188), before `cube.open` answers. So "card present at answer" is a load-independent
  invariant and stronger on ordering. It no longer bounds the latency of `cube.open` itself, so
  docs/cube-modes.md:166's 300 ms is now asserted by nothing.
- `at_rest` (:380-386) fixes a real test bug (rest width read mid-animation).

## Findings
- MINOR gate-construct-cube.sh:639: the 300 ms promise is unasserted. Failure scenario: a change that
  makes `open_cube` block (for example a synchronous wait in `add_outputs` or `launch` for 2 s) still passes
  because the card exists at the answer. Fix: time only `call cube.open` (one program, no `prop`/`grep`)
  against a budget, or assert it in a construct unit test. Earliest gate: the cube gate step.
- MINOR gate-construct-cube.sh:380 `at_rest`: two equal reads 250 ms apart can still happen
  mid-animation if the construct misses frames for >250 ms under heavy load. That leaves the old flake
  possible, only rarer. Stronger: N equal reads, or a step-back-done prop.
- MINOR the xp red test covers the free functions `change_keys`/`grab_keys`, not `sync_cube_keys`.
  A future edit of `sync_cube_keys` back to `ungrab_all` would pass the unit test, though the new
  gate cell (Super+d on the faces) catches only the stray-key effect indirectly.
- MINOR 0a3955d2 (red) also refactors and drops the `self.grab()` call from `sync_cube_keys`. That is
  behaviour-neutral, but the red commit is not purely a test.
- COSMETIC wm.rs:688: `#[allow(clippy::expect_used)]` on `Wm::grab` is stale now that the `expect`
  moved to `grab_keys`.
- COSMETIC tests/doors/tiling.sh:36: the fail message takes a second, later capture.

## Not verified
- Full merge gate not run. Builder honestly reports 1 of 3 loaded gates (xp tray flake, F18): the
  brief's "done when" is unmet, but not because of this diff.
- F16: not reproduced; only diagnostics changed.

BLOCK/MAJOR: none.
Verdict: MERGE-OK
