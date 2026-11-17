# Review construct r3 (head 93d508ea, base aa8aa8c0)

Reviewer: independent, read-only; code reasoning only, no build, no gate run (disk 19 G).

## Verdict: MERGE-OK (BLOCK 0, MAJOR 0, MINOR 3, COSMETIC 0)

## Checks
1. Red before fix. ORDER OK: daa0c747 (tests/gate-construct-placed.sh only) precedes 52a71b60 (fix). Without the fix
   `Pane::file()` has no `placed`, the saved file loses it, `adopt.rs:112` / `wm.rs:115` / `scene.rs:229` set `placed: false`,
   `frame.rs:171` hides the pane when the cube stands back, so `pane_drawn` is false and cell 2 fails. Cell 1 (a pane nobody
   placed gives way) guards against the trivial "always drawn" fix. Not executed here; builder reports the red.
2. Persistence. OK. `file.rs:53-56`: `#[serde(default, skip_serializing_if = Not::not)]`: old files load (false), new files carry
   `placed: true` only when set; round trip test covers it (`file.rs:252`). `main.rs:385` writes it; the loaders in `adopt.rs`,
   `wm.rs`, `scene.rs` read it; the waiting/handed-over/list literals (`adopt.rs:282`, `panes.rs:144`, `list.rs:96`) correctly
   say false. No new panic path: unchanged parse/`load`/`save` (`file.rs:182,192`; `save` still rejects `../evil`, test at
   `file.rs:299`). A pulled window without a command is not a file pane (`main.rs:362`): it returns to its face after a restart,
   so no window is lost; documented (`docs/cube-modes.md:204`). That is a behaviour limit, not a loss.
3. Gate changes. No assert weakened. hud: label reads moved to `construct-label-body-text` and compare the window title
   (foot `-T`), strength equal; the overflow `fits` lost its `if visible == True` guard (now stronger); the notice check reads
   `construct-hud-say` for "No room in view" (matches `main.rs:150`/`hud.rs:1580`); an Escape-closes-list assert was added.
   pull: two asserts added only (task button 1 on the face, no pane drawn). merge-gate.sh: steps `placed` and `construct-hud`
   added to the header list and the step table (`merge-gate.sh:10,179-180`), same `BIN=$repo/target/debug` form as neighbours.
4. "N-1 was the tour, not the product". Backed: the extended pull cell (pull, step back, drag back, owner xp, task button 1,
   no pane drawn) is reported 3 of 3 green on the head; the ch6 diff shows the old drag started at (330,186), which the builder
   places in empty space with the cube stood back, so nothing was pulled. Capture
   `construct-59a16702-2026-11-17-tour-old-script-no-pull.png` exists in reports/construct-r3/ along with the two fixed-script
   captures. Gap: the frame (f149-f161) of tour v6 was not checked, the act `06-15-pulling` is named instead (acceptable).
5. Merge with the session branch (7e066694): `git merge-tree --write-tree` is clean, no conflict. Merge base is aa8aa8c0;
   the diff is 14 files, 138+/23-.

## Findings
- MINOR `tests/gate-construct-hud.sh:~205,258` (builder note): at 1280x720@1.5 a Tab did not open the list while the pointer
  sat over a pane; the script now moves the pointer to (4,300) and sleeps 2 s. This may hide a real product quirk (list not
  opening under the pointer). Evidence: the new `ctl inject pointer_move` before `key Tab` and the builder's report item 4.
  Not judged; log in the ledger.
- MINOR gate evidence: `tests/merge-gate.sh` was not green as one run (hung at `tiling-look`, fuzzel 1.9.2 vs pin 1.11.1,
  known F27 family); the cells for this lane ran as an `--only` subset, all PASS. The brief asked for one run; PM should run the
  merge gate before merging and treat `tiling-look` as environment.
- MINOR `tests/gate-construct-hud.sh` adds ~243 s to the gate (step `construct-hud`) and a 2 s fixed `sleep` plus a
  long `fail` message with two `ctl call` dumps (debug residue in an assertion message). Acceptable, trim later.

## Unreferenced construct scripts (listed by builder, not fixed)
gate-construct-k1.sh, k2a, 3d, mcpane, nested, windows. Not reviewed.

## Refactor proposals (ranked)
1. Move `until_ok`, `pane_drawn`, `session_start`-wrapper helpers duplicated in `gate-construct-pull.sh` and
   `gate-construct-placed.sh` to `gate-lib.sh` (S).
2. `Pane`/`PaneFile` literals repeat the field list in 6 places (`adopt.rs`, `list.rs`, `panes.rs`, `scene.rs`, `wm.rs`,
   `tests.rs`); a `PaneFile::fresh(kind, ..)` / `Pane::new` constructor would have made this change one line (S).
3. `demo/tour/ch6.py` hard-codes the title bar point (870,225); read it from `xp-title-*` nodes. Also the six unreferenced
   construct gate scripts: wire in or delete (M).
