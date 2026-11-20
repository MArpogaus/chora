# flakes-l

## r2

Brief: flakes-l r2 (F5, no flaky tests): the six gate flakes of trains 82-87. Lane: cloud, branch claude/flakes-l from adf71956 (train 86), merged with origin/claude/autonomous-dev-session-3w-i33pg7 (fee9c695) before the final gate. Step 0 passed (`cargo check --workspace` exit 0; foot, python3, sccache present).

Load for every count: the cell alone, 3 copies at once (the gate lock serialises the HUD gate), beside repeated `cargo build --workspace` in a second target dir, later also 4 to 5 busy loops (load average 8 to 11 on 4 cores). No cell was retried, no timeout lengthened, nothing skipped.

| # | flake | red before | cause | fix | red after |
|---|---|---|---|---|---|
| 1 | tiling-look ink-2560x1600@2 "the focus border starts in 0 0 0 255" | 0 of 20 in the loop; hit again in the gate re-run at load 9 (also with my first fix, a wait for the border) | the watchdog of the core drops all effects when a frame costs more than its budget (F23); gate-tiling-bar-palette.sh did not take the 500 ms budget of the audit and the look gates. `CHORA_FRAME_BUDGET_US=1` alone gives the same line | cc1282db: the palette gate exports CHORA_FRAME_BUDGET_US (default 500000); ca5d99ee: the wait loop also waits for both ends of the border | 0 of 2 gate re-runs under load (tiling-look green twice at load 8 to 11); the cell alone is not re-counted to 20 |
| 2 | acts xp act:session xp.py:167 ValueError | 1 of 20 (run 15, same line) | the wpctl stand-in in tests/acts/looks/xp.py truncated `fake-state` before it wrote, so a read of the mixer in that gap is empty | 0b10637f: the stand-in writes a copy and moves it over the state | 0 of 20 |
| 3a | gate-hud context-picks-folder (FLK-HUD1) | 1 of 20, and 2 of 80 more with a trace | the cell read the figure at once and waited for its place to move. The question look and the listening look rest within 4 px of each other; the cell passed only when its read fell in the hop of the look before. The trace of a red run: the figure read at rest, 64 px at (1200,686), then (1198,688) for 5 s | c3fd74c5: the cell reads the look (the image of the figure) once it is unchanged for 2.5 s, asks the question, waits for a new look, ends the question and waits for a new look | 0 of 60 (load 8 to 11) |
| 3b | gate-hud voice-state-rate (FLK-HUD2) | 0 of about 160 (every HUD run above includes the cell) | not reproduced; the read `node_has hud-state Listening` is already a state wait (5 s) on a word that the host sets at once | no change | not applicable |
| 4 | layout FLK-LAY1 light-1280x720 Super+T, no third terminal | 0 of 20 | not reproduced; the backlog line says the hints were empty, which is the step back: Super+T came before the walk hints stood | a549da57: wait for the hints of the walk (not step-back) before the key (tests/gate-construct-layout.sh, before the third terminal) | 0 of 3 gate runs (1 full, 2 loaded) |
| 5 | tiling-audit fullscreen paper-2560x1600@2 bar does not hide | 0 of 11 (CELLS=fullscreen, 4 rounds each) | not reproduced; `fullscreen_rests` (tests/gate-tiling-audit.sh:84) is already a 20 s wait on the window rect and the work area | no change | not applicable |
| 6 | acts selftest persona sound, I5 overlap on the mc header | 24 of 100 (tests/persona session, planted.persona_check) | a product finding: mc answered `busy: []` in `state.get` while the set before still faded out under the new one (cross-fade, 250 ms), so the check read two sets, `mc-r0-header-*` and `mc-r1-header-*`, the same words twice | red test ec4f1a7b `mc_is_busy_while_its_regions_cross_fade`, fix 8d0645cc: mc says `fade` until the fade and 300 ms have passed | 1 of 100; the one is the same overprint, a render that lands between the last quiet read and the read of the nodes. Not closed |

Notes:

- Flake 1 shows what a count under load can miss: 0 of 20 in the loop, and the gate re-run at load 9 reproduced it. The cause is read from the line that fails with the budget set to 1 us.
- Flake 6 is open at 1 of 100. A full fix reads `busy` again after the nodes; that is a change of the harness (Run.snapshot, tests/persona/check.py, and the Snapshot of the acts) and I left it.
- Flakes 3b, 4 and 5 were not reproduced in 20 (3b: about 160, 5: 11). The brief's rule applied: the state wait added only for 4, where the backlog line names a state (empty hints); 5 and 3b already wait on state.
- Known, not mine: launcher-click is red with fuzzel 1.9.2.

### Gate

`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib,tiling-look,acts,hud,layout,tiling-audit` on 19862dfb (after the merge): all eleven steps PASS (pre-commit 5 s, gate-lib 72 s, ci-check 92 s, build 29 s, test 217 s, clippy 21 s, hud 78 s, layout 200 s, tiling-audit 288 s, acts 270 s, tiling-look 596 s). The palette gate then changed (cc1282db, test only; pre-commit ran on the commit). The touched cells (`--only tiling-look,acts,hud,layout`) ran twice more under load: run 1 green (hud 89 s, layout 268 s, acts 338 s, tiling-look 804 s), run 2 green (hud 106 s, layout 279 s, acts 343 s, tiling-look 957 s), load average 8 to 11.

Commits on claude/flakes-l (author and committer Marcel Arpogaus): ec4f1a7b, 8d0645cc, 0b10637f, ca5d99ee, c3fd74c5, a549da57, merge 19862dfb, cc1282db. Head: cc1282dbaf58eea7ef938a8ee9176b92fd815eae.
