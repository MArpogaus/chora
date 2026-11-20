# tour-plan (F7, RC 11)

## r1
Lane: cloud, read-only on code 9dc384b3 (detached; `git fetch --unshallow`, `cargo check --workspace` green, 1m31s). Wrote design/tour-v9.md (194 lines). No code changed.
Read: vision-card, foundation F7, rc 11, tour-v4, decisions-now 11-17 14:24Z and 10:59Z, review-tour-v7, media-r6 and the v8 timeline (claude/media-v8), proposal-glitch-gates, plan-after-foundation, audit-f (F7 rows), backlog DB11 and xp-T7, xp-give r1, persona p2-r4-xp, p2-r2-tiling, p2-r5-mc, p2-r3-co; on code: demo/tour (59 files), tests/acts (harness, invariants I1-I14, the acts per look), tests/persona/check.py (contracts T1-T7, X, L, M, C), tests/doors/lib.sh, gate-handover*, gate-give-holders.

### Plan in short
- Five chapters (xp Erika and Murat; tiling Kemal and a beginner; mc cook light and sofa dark; construct demo watcher and Max; handover nurse: Send To, Super+Alt+3, Super+Alt+4, a drop on the xp face, lock). 46 acts; each names its door, its persona contract or gate cells, and its capture frame (Q, Q+t, H50). Every door shown at least once; Launcher, Keys, Helper in every look.
- The tour acts call the act harness (`Expect.check`, invariants I1-I14) between segments; the same rows run without video as `run.py --check`.
- v8 comparison: a replay of the v9 rows on a build of 733967aa (same seed, size, stepped clock), scene diff always, pixel diff on quiet captures with masks (clocks, text boxes, pointer, caret); a channel over 24/255 differs; same = at most 0.1 % of unmasked pixels and no blob over 12x12 px. Change = named in demo/tour/intended.toml (incl. v8 open rows that are gone); regression = unnamed diff, or a check green on the replay and red on v9. v8 stills only as the reviewer's eye reference.
- Guidelines: 15 automatic rows (I2, I4-I10, I12, I13, video checker; 5 of them extend I2, I6 or I10: contrast, 14 px, spacing units, word table, still HUD in a camera move); fresh eyes (opus) for scaling, Luna fidelity, alignment, motion feel, theme, natural pace, and every change row.
- One set (DB11, F2): 4,291 of the 5,307 lines of demo/tour are copies (ch1-8, v4-v8, tour4-8, assemble4-8); folded into chapter rows; a tools/copies.toml row stops new copies.

### Cost
| Lane | Model | % |
|---|---|---|
| TOUR-ONE | sonnet medium | 1.5 |
| TOUR-DIFF (+ haiku tests) | sonnet medium | 1.3 |
| GUIDE-CHECKS | sonnet medium | 1 |
| CHAPTERS-A (xp, tiling, handover) | sonnet high | 1.2 |
| CHAPTERS-B (mc, construct) | sonnet high | 1 |
| RECORD (replay, take, diffs) | sonnet medium | 1 |
| REVIEW | opus high | 1 |
| reserve: one fix round, re-takes | - | 1.5 |
| **Total** | | **9.5 % (8 % without the reserve)** |
This lane: about 0.6 % (opus high, one round, ~110 k).

### Open questions (for the PM, the owner where marked)
1. Handover framing: follow the window (1600x900 main, four 320x180 thumbnails) or a 2x2 mosaic? Plan takes follow-cut (v8 two half-size screens read badly). Owner taste.
2. Baseline replay on 733967aa: acceptable that acts without a baseline (Send To, all `state.get` checks there) are judged by checks and review only? If more than about 30 % of acts have no baseline, fall back to a review against the v8 stills.
3. F-1: `choractl record` drew no notice where a live capture did (media-r6). A frame-level I9 on recorded frames needs this solved; it may be a core fix (record path) and then goes through the merge gate as its own slice. TOUR-ONE diagnoses it first.
4. 6 min cap of RC 11: the plan is ~5:45 with slides; keep the cap or let the handover run longer?
5. Owner called v8 "horrible": F7 asks only "no regression against v8"; v8 glitches that v9 still shows are known issues, not F7 findings. Confirm that reading.

### Risks
- The v9 rows may not run on 733967aa (scene ids moved in the refactor): the replay needs a selector map per build; cost can grow by about 0.5 %.
- Pixel noise between builds if the renderer changes in F3; the thresholds are a first guess, to be tuned on the replay of the candidate against itself (two runs must be "same").
- Stepped-clock frames align only if the cue timings match; captures are taken at quiet states and H50 is a held snapshot, so a different wait before quiet does not shift them.
- xp-give (22fead94) not yet in 9dc384b3; h2 falls back to Super+Alt+1.
- Persona contracts were vacuous in places (T7 r5); the tour T7 compares with the chapter's begin and one window with four owners.
- Natural pace (owner 11-17) and determinism pull apart; the seed of human.py keeps both.
