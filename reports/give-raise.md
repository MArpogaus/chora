# give-raise

## r1

Branch claude/give-raise, head 4b561e79. Red f0ab3136 (kit test `a_given_window_is_raised_and_focused_in_the_receiver` fails on dcb77733: the confirm of a given adopt returns no raise or focus; plus the gate assert on_top in tests/gate-handover-matrix.sh). Green 277dd835: `Book::confirm` of an `Origin::Given` adopt now also returns `Change::Apply` of Move (look group to top, no index) and Focus on the window, in the shared book, so all four looks get it; xp's own stacking is untouched. Gate assert follow-ups 01bb6d3f, 4b561e79.

Gate (--jobs 2, pre-commit,ci-check,gate-lib,build,test,clippy,matrix,mc-handover,tiling-audit,xp,doors,e2e): all PASS (matrix 306 s, e2e 295 s, doors 372 s).

Notes: the matrix assert (given window's root group is the last normal-order root group) covers xp and tiling receivers only. For mc and construct the check failed in every cell because their windows live in scene or pane groups, not as root-group siblings, so it does not apply there; the kit raise still applies to them. A first version with one query per root child took the matrix over its 387 s cap (about 400 s against 294 s base); the final one takes two queries. No captures.

## PM check r1 (2026-11-20 23:25 UTC; PM is not the builder; diff 3 files +90 -3, dcb77733..4b561e79)
- Red first: f0ab3136 (kit test + matrix on_top) before the fix 277dd835. Pass.
- Fix in the shared path (kit Book::confirm), not per look: Origin::Given only; a new window gets no raise (test a_new_window_gets_no_raise_from_the_book). Book.name is the id of the look's root group (book.rs:164), so Move raises the look's group in its band. Pass.
- Gate: full list at --jobs 2 green (lane report). Train 95 reruns it.
- rv-1 MINOR: on_top checks xp and tiling receivers only; mc and construct get the same Move and Focus but no gate assert (their windows sit in scene or pane groups). Backlog.
- rv-2 MINOR: on_top checks the look's group against the root; the order of the given window inside the receiving group (xp: under another xp window?) is not asserted. Backlog: a matrix assert on the receiver's own top window.
Verdict (computed): no BLOCK or MAJOR open: MERGE-OK. Joins the train after t94b.
