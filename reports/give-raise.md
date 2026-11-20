# give-raise

## r1

Branch claude/give-raise, head 4b561e79. Red f0ab3136 (kit test `a_given_window_is_raised_and_focused_in_the_receiver` fails on dcb77733: the confirm of a given adopt returns no raise or focus; plus the gate assert on_top in tests/gate-handover-matrix.sh). Green 277dd835: `Book::confirm` of an `Origin::Given` adopt now also returns `Change::Apply` of Move (look group to top, no index) and Focus on the window, in the shared book, so all four looks get it; xp's own stacking is untouched. Gate assert follow-ups 01bb6d3f, 4b561e79.

Gate (--jobs 2, pre-commit,ci-check,gate-lib,build,test,clippy,matrix,mc-handover,tiling-audit,xp,doors,e2e): all PASS (matrix 306 s, e2e 295 s, doors 372 s).

Notes: the matrix assert (given window's root group is the last normal-order root group) covers xp and tiling receivers only. For mc and construct the check failed in every cell because their windows live in scene or pane groups, not as root-group siblings, so it does not apply there; the kit raise still applies to them. A first version with one query per root child took the matrix over its 387 s cap (about 400 s against 294 s base); the final one takes two queries. No captures.
