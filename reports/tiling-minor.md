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

## Review r1

Reviewer lane, read-only on code: claude/tiling-minor-r1 head 31af60c5 on 84f66d43
(13 files, +363 -50, 15 commits). Step 0 green (`git fetch --unshallow`,
`cargo check --workspace`). No merge gate run (lane gate /tmp/merge-gate.rTgTF5).

Checks:
1) rth-1 core press_layer: PASS. Claim at the point (claimant_at, one owner),
   else live claim owners overlapping the shown rect of the layer's output, else
   every live claim owner. One claim at the point gives one look; with one look
   the result equals the old `first_manager` (which also needs a claim; the new
   code adds the `live` filter). Injected press: no `last_real_press`, no act;
   locked: no act. Escape hatch, grab path and `note_act` lock guard untouched.
   `cargo test -p chora input::` 65 passed; red confirmed at 8e2052a7 (both rth-1
   tests FAILED, lock test ok).
2) Red first: PASS. Every fix sha follows its test sha (ab0738de; 8e2052a7 ->
   6b199a70; 9fe4649a -> bcc8cb4f; b8b2d3b5 -> 87aa0d1e; 2c928a8b -> 47465ea4;
   6a20aba7 -> 58270886; f8a15eb7 -> ea5ab55e). rth-8 hardened in b8b2d3b5 with
   the rth-5 red run pasted: accepted.
3) rth-4: PASS. The chip shows "title: reason" (70 chars, 5 s) only when
   `show_or_log` gives None; the only `eprintln` in the diff is the moved line of
   main.rs; `tools/lint-ratchet`: eprintln 46.
4) rth-5 gate step: PASS with rtm1-1. A poll (`until_ok no_notice 16`), no new
   sleep; `tools/sleep-ratchet`: ok, sleep 778, sleep-all 929 (list 779 -> 778 true).
5) rtd1-2..4: PASS with rtm1-2. en and de for KeyUnboundBare, the theme titles and
   the Looks notice; chora-tiling 208 unit tests and chora-kit words tests green.
6) Overlap with claude/tiling-doors-r2 (still at 721be592, nothing pushed): no
   behaviour conflict. HINT_TIME only feeds `hint_unbound`; the reload chip has its
   own CHIP_MS. If r2 injects a clock into `show_words`/`chip_until` (its test b),
   the new call in reload.rs:81 fails to compile on merge, a visible conflict, not
   a silent one. Word overlap is text only (rtm1-5).

Findings:
- rtm1-1 MINOR tests/gate-tiling-reload-notice.sh:67: the card expires 12 s after the failed reload and the step waits 8 s after `card_shows`, so when the earlier waits take more than about 4 s the step passes without the fix; fix: wait 2 s at most (`until_ok no_notice 4`), since the dismiss is synchronous.
- rtm1-2 MINOR plugins/chora-tiling/src/main.rs:375: the rtd1-4 test pins only the words of `unreachable_notice`; nothing checks that a failed clients.list in `launch_looks` shows the notice; fix: a unit or doors cell that runs the Looks launch against a core whose clients.list errors and sees one notice.
- rtm1-3 NIT crates/chora/src/input.rs:3117: the lock test still passes with the lock guard of `note_act` removed (the lock hides the bar from `surface_under`), so it pins the hit test, not the guard; fix: say so in its doc comment or add a `note_act` unit test under a lock.
- rtm1-4 NIT crates/chora-kit/src/runtime.rs:568: a good reload through the method `settings.reload` (not the key) leaves the failure card up until it expires; fix: dismiss in a plugin hook on Ok of the method too, or note it in rth-5.
- rtm1-5 NIT plugins/chora-tiling/src/looks.rs:60: the notice says "Other desktops" / "Andere Desktops" while ti-55 (TILING-DOORS r2) makes "look" the word on screen; fix: align the title with the bar word when r2 lands.

Verdict: MERGE-OK (0 BLOCK, 0 MAJOR, 2 MINOR, 3 NIT).
