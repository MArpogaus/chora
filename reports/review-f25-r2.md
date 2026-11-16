MERGE-OK
# Review of claude/f25 (cfb4e269, df101c14) over 66f13d68

| Sev | Where | Finding | Evidence |
|---|---|---|---|
| none | cfb4e269 | Red-first holds: test commit precedes fix, touches only render.rs + render/damage.rs | at cfb4e269: FAILED, damage.rs:56 "the veil blended again outside the damage left: 192 right: 128"; at df101c14: 1 passed |
| none | mesh.rs:1004-1008 | Damage is relative to dst; fix adds dst.loc, then window_rect applies projection + viewport (GL window coords, y flipped by projection, cut to viewport, None when empty). Same pattern as BackdropElement (backdrop.rs ~463) | code read |
| none | gl.rs:230-246 | Scissor reset to the viewport after the loop, also on error; empty damage draws nothing. Overlap: the tracker hands over non-overlapping rects, so one blend per pixel | code read |
| MINOR | gl.rs:238-245 | After a failed draw the loop keeps drawing (result.and); harmless, could break early | code read |
| MINOR | gl.rs:246 | "Restore" sets the scissor to the full viewport, it does not read the previous scissor or disable the test. chora never enables SCISSOR_TEST (grep: none); it relies on Smithay's frame having it on and setting its own scissor per draw, as backdrop does. Correct today, relies on Smithay behaviour | grep SCISSOR crates/chora/src: empty |
| MINOR | damage.rs | Test only covers scale 1, no output transform, one damage rect; the scissor-space and multi-rect paths have no direct test (CI e2e and merge-gate cover the real case) | code read |
| none | impl RenderElement | Only 3 impls: MeshElement (fixed), BackdropElement (scissors per damage window), CellElement (images.rs:683, delegates to Smithay texture draw) | grep -rn "impl.*RenderElement" crates/ |
| none | tolerance | No change to tests/agent.sh or thresholds; diff touches 5 files, all under crates/chora/src | git diff --stat |
| none | commits | Both conventional, author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>, no attribution lines, body lines at most 72 chars | git log, line widths read |
| none | clippy | `cargo clippy -p chora --all-targets --locked -- -D warnings` finished clean at HEAD | exit ok |

Not re-run: the CI-image e2e loop (30/30) and merge-gate claimed in reports/f25-r2.md.
No open BLOCK or MAJOR.
