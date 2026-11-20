# kit-minor

## r1
Branch claude/kit-minor from e7f2410a, pushed at 939acf6a. Gate (pre-commit, ci-check, build, test, clippy, gate-lib, mc-paths, mc-handover, wm-pane, acts, doors): all PASS.
1. pr6-1 (code side was wrong, docs/api.md right): a real give (D55 remove with `to`) arrives as surface.detached and got `new`; a created surface reserved for the look got `given`. Now detached is `given`, created (reserved or not) is `new` with its token. Red 1d61e031, fix 861958ac, follow-up cell 939acf6a (detached adopt expects Given). Cells: wm_book a_window_given_in_has_the_origin_given (via detached), a_window_reserved_by_the_core_is_new_and_keeps_its_token; the mc, construct and xp origin cells now expect `new` for the created+to window.
2. helper.rs answered -> working: already covered at e7f2410a (cell the_answer_of_a_helper_holds_while_voice_listens_again asserts it; red 11267d2a, fix 2d6808f2). No change.
3. new utterance clears answered: already fixed at e7f2410a (cell a_new_utterance_clears_the_answer_of_a_helper; red 11267d2a, fix 2d6808f2). No change.
4. seed() report order: already fixed (cell a_late_reader_keeps_the_order_of_the_reports, fix 88f441e4). No change.
5. api.md rewrap: already done in 2d6808f2 (helper paragraph). No change.

## PM review r1 (PM not the builder)
MERGE-OK 939acf6a; red 1d61e031, fix 861958ac. Checks: book.rs claim_as keeps the guard order; detached -> Given, created (reserved or not) -> New{token}; mc windows.rs:59-62 matches New|Given on both arms, so mc-42 keeps its token path; xp, mc, construct origin cells moved to `new` with the token still out of the state. No open MAJOR.
- pr1-1 NIT: claim() doc comment above still describes the reserved case only through `to`; one line on the origin would help.
