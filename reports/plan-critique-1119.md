# Plan critique 2026-11-19 06:16 UTC (two opus critics, read-only; owner 06:12)

## Waste (critic 1)
1. Opus review on every handback costs 8-12 %; sonnet reviews found the real port defects (kit2b rk-1). Opus only for NET3, WM2, WM7a, security.
2. The gate runs three times per slice (lane, reviewer, train). The reviewer runs only `cargo test -p` for the touched crates.
3. Unmerged bases: WM2 pilots mc on book without wm1b; WM1c starts on wm1b and merges mv3 itself. Shared files: tools/copies.toml (WM1c, KIT6, WM2), the known-red list (NET3, WM2), merge-gate.sh (CUT2).
4. The budget sums did not add up. tokens.py cannot see cloud lanes. No report records a lane's actual cost.
5. Trains ran about 1 per 3 h (48-56), which allows 18-22 merges by Sat against 28 planned. Cap the unmerged backlog at 4; spend the slack on flakes and tests.
6. Seeded review: base and head SHA with diffstat; the brief's done criteria as checks; known findings by id; a command and files per check; no gate; cap 120 k and 0.5 %; at most 8 findings; the verdict rule.
7. Fable pays once, on the flake classes F21, F29 and the hud flake (train throughput), and maybe on the WM7 cube design.

## Alignment (critic 2)
1. KIT6 (rank 13) started ahead of the KIT2c/d and WM1d ports; it touches xp and tiling, so expect rework. CUT2 is justified (F6).
2. WM4, WM6 and NET4 move no F criterion this week: cut. F7 cannot be met by Sat (tour next week); NET3 is mislabelled F7.
3. F5 has no lane (F17, coverage on the merged head). For F4, MV4 must name region.rs and chat.rs, with cube.rs after WM1c, plus a debt-<date>.md. F8b must name CONTRIBUTING and dev-rules.
4. Merge order: KIT2b r2, WM1c, NET3, WM2, KIT6, CUT2 last.
5. A Saturday opus check lane measures each F on the final head.

## PM decisions (06:16)
- Adopted: 1, 2, 3 (merge order), 5, 6, 7 of the waste critique; 1-5 of the alignment critique.
- KIT6 keeps running (rule 5: no scope message); its successors wait.
- The owner preferred opus (06:12), but port reviews go to sonnet high because the evidence shows sonnet finds the port defects. Opus stays on core and design reviews.
- F7 is declared open for Sat (owner 11-17 14:20: tour after the refactor and the user test); no question to the owner.
