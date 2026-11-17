# xp-polish r6 (rc 1): Notepad notice only after the window shows; no second editor over a slow one

Branch claude/xp-polish at c7e4e59 (session branch already merged). Sonnet, one round, 2026-11-17.

## Done
| item | result | cell |
|---|---|---|
| MAJOR 1 "opened with X" only once X's window shows | fixed | gate-xp-notepad-fallback.sh, last cell; unit tests in launch.rs |
| MAJOR 2 no second editor over a slow hand-off editor | fixed | gate-xp-notepad-fallback.sh, hand-off cell (fake kate forks, exits 0, window after 4 s) |
| docs | docs/configuration.md `editor` text updated | - |

Behaviour: the fallback notice is kept in the start and said when the window of that start shows (or the start is still
running at 30 s). A failing chain gives only "Cannot open Notepad". The shell around an early (Notepad) start writes a
non-zero exit status to a small file in XDG_RUNTIME_DIR (removed with the start); only a non-zero status replaces the editor
at once (status 127 covers a missing binary). Status 0 or no status keeps the 30 s limit.

## Evidence
- Red (e23952b cells on the old chora-xp f40b2dd binary): hand-off cell FAIL "a second editor opened over the slow hand-off
  editor"; failing-fallback cell FAIL "a notice says Notepad opened with mousepad, which ended without a window".
- Green (c7e4e59): all 4 cells ok; cargo test -p chora-xp 186 ok; clippy -D warnings clean.
- No capture: no look changed (notices only), the r5 capture stands (xp, f40b2dd, 2026-11-17).

## Merge gate (one run on c7e4e59 after unshallowing the clone; the first start stopped at 2d-guard, which needs base 8742dee)
PASS: all steps through mc-sofa incl. xp, xp-notepad, hud, layout, tiling-audit, keys, pixels.
FAIL e2e: gate-tiling-apps-click "a click on the Zebra row did not start zebra" (fuzzel 1.9.2, also on base). Not chased.

## Open
- Only the latest failed editor is named in the success notice (not the whole chain).
- A start that replaces the shell (`exec`) leaves no status, so it keeps the 30 s limit.
- MINOR 1, 2 of the review go to known issues.
