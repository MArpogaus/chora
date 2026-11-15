Verdict: MERGE-OK

# Review: claude/flakes-r5 at 366c1cb3 (base 98291088)

Reviewer: independent, read-only. Read: the brief, the lane report, the full diff (8 commits, 8 files,
+72/-15) and safety.rs, hang.rs, nested.sh step 14 and the xp test helpers.
Ran (on a `git archive` copy, CARGO_TARGET_DIR=/home/user/target-review, now deleted):
`cargo clippy -p chora-construct --bins -- -D warnings` with and without the construct allow.
I did not build voice (tract-onnx), run any GUI gate or run the merge gate.

No BLOCK and no MAJOR findings. Three MINOR and three NIT findings follow.

## Checks

1. F19 (1f1f5389 red, 975c43b7 fix): the red comes first and fails for a fixed reason. Before
   the fix, both `theme("one-name")` calls get the same folder, and the second `remove_dir_all`
   (and the `clear` of the second load) removes the first theme's PNG. The fix removes the cause:
   the `sheets::tests::theme` folder name now includes a per-process call counter, and
   `dialog::tests::theme` and `taskbar::tray_runs` both go through it. No retry, no timeout.
2. The clippy ban (2db659f0) works at compile time. I removed the allow at
   plugins/chora-construct/src/scene.rs:185-189 and clippy failed with
   "use of a disallowed method `chora_kit::runtime::Runtime::ungrab_all` ... (GR1)". With the
   allow in place, clippy is clean. The merge gate runs `cargo clippy --workspace --all-targets
   -- -D warnings` (tests/merge-gate.sh:143), so a new call fails the clippy step. There are
   three allows (construct leave, voice 688 and 709). Each sits on one statement and gives a
   reason. The two voice allows name GR2.
3. No test got weaker:
   - at_rest went from 2 reads to 4 equal reads.
   - cube.open gained a new 300 ms bound.
   - The audit now fails when no bar shows. Before, the probe loop fell through without a word.
   - The doors message now quotes the last capture instead of running a new one.
   - No assertion was removed. The one relaxation is the 500 ms budget (finding M1).
4. Diagnostics for the items that did not reproduce: F16 is enough (waybar log plus the last
   capture text). F7 is enough (the panic prints the hangs). F18 is not (finding M2).

## Findings

### M1 (MINOR, core; not this lane's regression): the watchdog blames the scene for host load under the software renderer
- Where: tests/gate-tiling-audit.sh:43 (`CHORA_FRAME_BUDGET_US` defaults to 500000);
  crates/chora/src/safety.rs:14-22 and 186-198 (`note`); safety.rs:336
  (`drop_all_effects`).
- Evidence: the watchdog subtracts only the run-queue wait of the main thread. The safety.rs
  doc says software-renderer threads that wait for a CPU look like a costly frame. Under load at
  2560x1600@2 that is enough for 2 s of "slow" frames, so all effects of all clients are
  stripped (`Scene::drop_effects` removes the props). They come back only when a client sets
  them again. Users see this: borders, glow and shadow disappear on a busy host without a GPU
  (VM, pixman/headless), and dropping effects does not lower the host load.
- The gate change itself is right. It is a look gate, the same budget is already used by the
  cube, HUD, 3D and K1 gates, and the watchdog keeps its own cells (nested step 26, HUD with
  budget 1).
- What is missing: the ledger entry for F23 calls this "gate only". The core cause needs its own
  backlog item, or the one gate that hit it now hides it. Possible fixes: sum the schedstat wait
  of the renderer threads (or read the process-wide wait once per frame), or skip
  `drop_all_effects` when the host is busy.
- Earliest gate: a red unit test in safety.rs on `note`, with a frame whose main thread did not
  wait but whose renderer threads did. Then tests/gate-watchdog-starved.sh at 2560x1600@2 with
  the default budget, asserting no `EffectsDropped`. Today it only shows that no client is
  disconnected.
- F21/F23 also has no red commit. The lane showed the cause with `CHORA_FRAME_BUDGET_US=1`
  (same colours at the same pixels), but the core log of the failing run was gone, so the
  cause is shown by its signature only. Acceptable for a gate-only change. Say so in the ledger.

### M2 (MINOR): F18 adds no diagnostics; the next hit will again say only "timeout: not_black"
- Where: tests/nested.sh:280-281, and `wait_for` at nested.sh:40-46 (50 x 0.2 s = 10 s, then
  `fail "timeout: $*"`).
- Evidence: the failure does not show the pixel value at 100,100, `top_color`, the tail of
  probe.log for `take-lock`, or whether the core still holds a lock. The next hit cannot tell
  "still locked" (black) from "drawn late" from "the probe failed after the takeover".
- Ask: print `pixel 100 100`, `top_color`, `tail probe.log` and the lock lines of the core log
  on failure.
- Earliest gate: tests/nested.sh step 14.

### M3 (MINOR, process): the lane's temp-folder scan missed a real clash, which this fix happens to remove
- Where: plugins/chora-xp/src/taskbar.rs:924 (`chora-xp-start-<pid>`) and, at base 98291088,
  sheets.rs `theme("start")`, which produced the same folder.
- Evidence: `the_start_button_draws_no_flag` and `the_start_button_is_green` both loaded a theme
  into `chora-xp-start-<pid>` and removed it. That is the same F19 race, from two different
  helpers. 975c43b7 removes it, because sheets names now end in `-<call>`.
- The report says "a scan found no clashing literal names". It should list this pair. It backs
  the lane's proposal for one counted temp-folder helper across crates (F19 class).
- Earliest gate: a unit test of the shared helper (unique folder by construction).

### N1 (NIT): use expect rather than allow for the disallowed_methods exemptions
- Where: services/voice/src/client.rs:684-688 and 705-709; scene.rs:185-188.
- Evidence: when GR2 removes the voice calls, an `#[allow]` stays silently stale. That is the
  class a71464e7 just cleaned up at wm.rs:688.
  `#[expect(clippy::disallowed_methods, reason = ...)]` would fail clippy once it is no longer
  needed. The repo uses no `#[expect]` today, so this is a convention question.
- Earliest gate: clippy (unfulfilled_lint_expectations under -D warnings).

### N2 (NIT): the ban does not catch the same pattern written with start_grabs and end_grabs
- Where: clippy.toml:13-15.
- Evidence: `start_grabs(); end_grabs();` with no grabs in between is `ungrab_all` under
  another name, and clippy cannot see it. The doc comment on `ungrab_all` / `start_grabs` in
  crates/chora-kit/src/runtime.rs:315-335 could say so. Review is the only gate here.

### N3 (NIT): the new 300 ms bound on cube.open is a fresh timing assertion under load
- Where: tests/gate-construct-cube.sh:641-648.
- Evidence: it subtracts one `construct.list` sample. If that call stalls, the bound gets
  looser, never stricter, so it cannot fail falsely from that. Only a stall inside cube.open
  itself can fail it. Measured 82 ms at load 11 and three loaded gates passed, so the margin is
  wide. Watch it in the flake ledger.

## GR2 (as asked)

voice client.rs:688 (`confirming` done) and 709 (`apply`) still run `ungrab_all()` and then
`grab()`. A key of both sets is briefly free and can reach a window (the GR1 class). Both carry
an allow that names backlog GR2. The ban now stops new cases. These two need the
start_grabs/grab/end_grabs shape and a red test on a fake core, like the wm.rs `on_fake_core`
tests.

## Gates the lane reports

Three full merge gates in a row under load (9.3-9.4) pass at 366c1cb3, plus one cold run, and
ci-local pre-commit passes. I did not re-run them (told not to). The diff merges cleanly as stated.
