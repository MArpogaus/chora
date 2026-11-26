# tiling-chip

## r1

Item: F7, rv9kb-4 (cosmetic): the word of the layout chip sat at the left of its chip, about 75 px of empty well on its right, like an input field.

- Base: a93bc75b. Head: 8add6eca (branch claude/tiling-chip-r1).
- Commits: red 47c28164 (test: word centred in the chip), fix e6afbf97 (`"align": 0.5` on `custom/layout`; the chip keeps `min-width: 100px`, no new option), a130b7bf (the check moved into the existing layout loop so the sleep ratchet stays flat), 8add6eca (the spacing gate expected the word left aligned at 10 px; now it expects it centred; docs/configuration.md sentence updated).
- Diffstat: 5 files, +61 -10 (config.jsonc 1, docs/configuration.md 5, tests/chip-gaps.py 30 new, tests/gate-tiling-bar.sh 16, tests/gate-tiling-spacing.sh 18).
- Red on the base: `chip tile: left gap 8, right gap 85, width 116` then `FAIL: the word of tile sits off centre in its chip: gaps 8 and 85`.
- Gaps before and after (ink, 1280x800, chip 116 px wide): tile 8/85 before, 46/47 after. After: tile_left 34/34, fair 47/46, spiral 39/40, max 44/44; the width is 116 for every word.
- Captures: media/2026-11-26/tiling-chip-r1/bar-{before,after}-{ink,paper}.png (bar crop, 2x, each about 15 KB).
- Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --push), all steps passed; last lines: `tiling-bar PASS 183s`, `e2e PASS 280s`, `all steps passed; no --push, nothing is pushed`. A first run had stopped at the sleep ratchet (my extra sleep) and at the spacing gate (`the layout chip to its word: 50 px, not 10`); both fixed by the commits above, then the full gate ran again clean.

## Review r1 (PM, 71 lines)
Base a93bc75b, head 8add6eca (4 commits, 5 files, +61 -10). Red first: 47c28164 ("chip tile: left gap 8, right gap 85") before the fix e6afbf97 (`"align": 0.5` on custom/layout, one line). a130b7bf puts the check in the existing layout loop (no new sleep); 8add6eca turns gate-tiling-spacing.sh from "left aligned, 10 px" to "both gaps alike" (the spec change, docs/configuration.md with it). Gate: all steps passed (worker). Rule 6: COSMETIC on tiling while ti-57 reopened; the lane started before r12 reopened it, so it finishes (PM 11:2x). Findings: none. **Verdict: MERGE-OK 8add6eca: train 154.**
