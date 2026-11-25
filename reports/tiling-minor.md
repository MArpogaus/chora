# tiling-minor

## r1

Branch claude/tiling-minor-r1, head 31af60c5, on 84f66d43. Red commit before
each fix (the gate step for rth-5; the unit test for the others).

| finding | red sha | fix sha | test |
|---------|---------|---------|------|
| rth-1 | 8e2052a7 | 6b199a70 | input.rs `a_press_on_a_bar_outside_the_claims_goes_to_the_look_of_its_output`, `..._where_no_claim_lies_goes_to_every_look` (two managers; the far one connected first). The act goes to the claim at the point, else the managers whose claim overlaps the output of the layer surface, else every live manager. |
| rth-2 | docs only | 6a801c0c | api.md, architecture.md, state.rs `Act`/`note_act` name a real press on a layer surface |
| rth-3 | ab0738de | (tests only; they pass at once) | input.rs `a_real_press_on_a_bar_is_an_act_of_the_look_of_the_claim`, `an_injected_press_on_a_bar_is_no_act`, `a_press_on_a_bar_while_locked_is_no_act` |
| rth-4 | 9fe4649a | bcc8cb4f | main.rs `a_failed_reload_without_a_notice_server_shows_the_reason_on_the_chip`; reload.rs shows "title: reason" (70 chars) on the chip for 5 s when `show_or_log` gives None |
| rth-5 | b8b2d3b5 | 87aa0d1e | tests/gate-tiling-reload-notice.sh last step: a good reload leaves no notice within 8 s (the card expires after 12 s); red run showed "a good reload after a failed one leaves the card up" |
| rth-6 | open (backlog) | | |
| rth-7 | docs only | 6a801c0c | configuration.md `show_helper` row: "as `MOD+a` (`run: chora-chat`) does" |
| rth-8 | b8b2d3b5 (same commit as the rth-5 step) | b8b2d3b5 | the gate takes the pixel before the reload (`$base`) and reads it at the card centre; no separate red, the check was hardened |
| rtd1-2 | 2c928a8b | 47465ea4 | theme.rs `the_titles_of_every_missing_script_are_pinned_in_both_languages`; German "Design-Umschalter nicht installiert" |
| rtd1-3 | f8a15eb7 | ea5ab55e | main.rs `the_hint_names_no_help_key_when_none_is_bound_rtd1_3` (red) and `the_hint_names_a_rebound_help_key_rtd1_3` (passed at once); new word `KeyUnboundBare` in chora-kit (en, de) |
| rtd1-4 | 6a20aba7 | 58270886 | looks.rs `a_door_that_cannot_list_the_looks_says_so_in_both_languages` (red on a stub that answers empty strings); `launch_looks` shows the notice |
| rtd1-5 | docs only | 6a801c0c | configuration.md "The theme rows set `look.theme`" |

Not in the brief and not touched: rtd1-1 (the sleep 0.5 in gate-tiling-apps.sh).
The sleep list was lowered by one (31af60c5, `tools/sleep-ratchet --update`:
tree 778, list was 779); no sleep line was added.

### Ratchets (on head 31af60c5)

```
ok: sleep-ratchet (sleep 778, sleep-all 929)
ok: size-ratchet (0 files over 1500)
ok: lint-ratchet (allow 360, no-reason 0, eprintln 46)
```

The lint ratchet was at 47 after rth-4 (one `eprintln` for the chip); the last
commit removed it, so the count is the 46 of the base.

### Gate

`tests/merge-gate.sh` (no --only), one run, on head 31af60c5: all steps PASS,
no SKIP ("all steps passed"). Run directory /tmp/merge-gate.rTgTF5 (cloud
lane; red-first, identity, pre-commit, ci-check, build, test, clippy, layout,
acts, doors, matrix, e2e and the tiling-* steps among them).

One slip, corrected before the push: the first red commit of rth-1 failed the
subject-length hook, and the fix commit then held the tests; I split them
again (8e2052a7, then 6b199a70), so red-first holds in the history.
