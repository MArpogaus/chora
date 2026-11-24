## r1

Lane: cloud, branch claude/co-repro-r1 at 280b9dc (binaries of p2-r12-co, snapshot sha 280b9dc). Hand runs only, no code change. Step 0: `git fetch --unshallow`, `cargo check --workspace` (rc 0), `cargo build --workspace` (rc 0). Each run: fresh compositor, `construct 1280x800`, state dump (`cube.list`, `state construct`, `state xp`, `query kind:surface`), one `act.py look` at the end. Scripts and logs: media/2026-11-24/co-repro/ (co77.sh, co78.sh, logs). Start of each run: cube row click, Return (step into the start face, front 0, mode use). The brief's "empty face" (b) is the same face without Super+t. Programs was opened with Super+Space ("All programs", the fuzzel list) and Foot picked by typing `foot` and Return; a click on the Foot row did not select it in a trial.

### co-77 (5 runs each)
| Path | Foot holder (dump) | core focus | xp focus | front | `echo rN` in focused Foot (capture) | Foot full size |
|---|---|---|---|---|---|---|
| (a) Super+t, then Foot from Programs | 5/5 `xp`: terminal s1 and Foot s2, both owner xp | `holder null` 5/5 | Foot s2 5/5 | 0 | **5/5** (blue focus frame on the second window, text visible) | 0/5 (windowed, about 700x525) |
| (b) empty face, Foot from Programs | 5/5 `xp`: Foot s1, owner xp | `holder null` 5/5 | Foot s1 5/5 | 0 | **5/5** (the only window, text visible) | 0/5 (windowed) |

Answer: the person gets a focused, usable Foot: keys reach it in 10/10. What differs from the contract is the holder: the Foot is owned by xp (the xp-managed face window), not construct, and it is a window, not full size. This matches the C3 FAIL dump of r12 (holder xp, construct none). The keys do not go elsewhere in these runs (the list closes when Foot is chosen). Not isolated: a single manual run (session x1, no cube step, no sheet) gave a construct-owned, full-size Foot (rect 0,0,1280,800); co-78's Foot, which follows Super+F1, is construct-owned as well (see below). Why the owner differs between these paths is open.

### co-78 (5 runs each)
| Variant | sheet open before Super+t | after Super+t | after Escape: `open` | core focus after Escape | capture |
|---|---|---|---|---|---|
| sheet (cube row, Return, Super+F1, Super+t, Escape) | 5/5 `construct-help` | Foot s1, owner construct, focus construct s1 | **still open 5/5** | construct s1 (Foot) 5/5 | co78-sheet-run1..5.png |
| control (same without Super+t) | 5/5 | none | closed 5/5 (`open` empty) | construct 5/5 | co78-control-run1..5.png |

Answer: Escape goes to the Foot that holds the keyboard after Super+t, so it does not close the key sheet (the sheet keeps the modal state and the focus stays on Foot).

### Summary
- co-77: keys reach Foot 5/5 (path a), 5/5 (path b); Foot holder xp in 10/10, full size 0/10.
- co-78: Esc closes 0/5 (control 5/5).

Captures (20, under 2 MB each, 3.4 MB total): media/2026-11-24/co-repro/co77-{a,b}-run1..5.png, co78-{sheet,control}-run1..5.png. Dumps and actions in the logs (R1-co77-a.log, R1-co77-b.log, R1-co78-sheet.log, R1-co78-control.log).

Not run: other rows; co-77 owner cause; the earlier trial sessions (t1, t2) are not in the media.
