# kit4 r1 (F2): kit::ask the only question model in mc and construct

Branch claude/kit4 (from bdf84f41). Commits: dece31b3 (red: kit, mc, construct cells, copies rows at 0, kit stubs), aeb13ece (kit::ask is the only question model; docs).

## Done
- `chora_kit::ask`: `Question.buttons`, `Answer {Yes, No, Always}` with `Question::answer_params` (`always` adds `"always": true`; reserved, no card and no button offers it, the voice service does not take it yet: ButtonParams denies unknown fields), `Card::click(button, injected, now) -> Click {Answer(params), TooSoon, Ignored}`, `ask::answer_call(params)` and `ANSWER_METHOD`.
- mc: `Confirm`, `Keys`, `id_text`, the Ja/Nein pair and its key-cap code are gone; `Confirming` wraps a kit `Card`. A press goes through `Card::click`.
- construct: `Confirm`, `Keys`, `cap` gone; `Ask` wraps a kit `Card` and the wheel scroll. `Construct::answer` returns true when the click was spent (also TooSoon), so a refused click does not fall through to a focus.
- tools/copies.toml: three rows at 0 (own question event model, literal Ja/Nein in plugins, own key cap); `confirm.answer` max 10 to 8.
- docs/configuration.md says Yes and No (Ja and Nein in a German session) and the 400 ms; sh-26 and sh-27 dropped from docs/known-issues.md (the ledger is the PM's).

## Cells (red first)
- sh-26: kit `a_click_inside_400_ms_of_the_card_is_refused`; mc `a_press_inside_400_ms_of_the_card_is_refused` (counts `confirm.answer` calls in the fake core: 0 at 100 ms, 1 at 900 ms); construct `a_press_inside_400_ms_of_the_question_is_refused`.
- sh-27: mc and construct `the_buttons_follow_the_session_language` (En Yes/No, De from kit words).
- bidi/control characters: the existing mc `a_confirmation_shows_no_bidi_character` and construct `a_question_shows_no_bidi_character` stay green on the kit path; kit `a_hidden_character_in_a_key_never_reaches_the_card` is new (keys are sanitized too).
- `always_is_reserved_and_no_card_offers_it` (kit).
- Red 3 kit, 1+1 mc, 1+1 construct cells failed on dece31b3; green on aeb13ece. Note: non-numeric ids in old tests ("c1", "a") became numbers; the voice service sends numbers only (client.rs: "id: a number").

## Coverage
`cargo llvm-cov` is not installed in this container (no cargo-llvm-cov in ~/.cargo/bin): **blocker, not installed**. The kit `ask` cells cover from_event, keys, parts, buttons, press, click, answer_params by name (14 tests); no percentage measured.

## Merge gate (run on aeb13ece, --jobs 3, log /tmp/merge-gate.GiVqUd in the lane container)
All steps PASS (red-first, pre-commit, gate-lib, build, test, clippy, ci-check, 2d-guard, package-*, virtual, xp-*, hud, layout, chat-keys, tiling-audit, acts, last-words, cube, doors, placed, pull, out, winit, wm-pane, pane-keys, walk, keys, tiling-look, pixels, mc-sofa, mc-paths, mc-handover, construct-hud) except **e2e FAIL** at step 24 `gate-tiling-apps-click` "a click on the Zebra row did not start zebra": known F27 / ti-1 (fuzzel 1.11 pin vs the container's fuzzel, same as reports/construct-r2.md); tiling and fuzzel are untouched by this branch (no diff in plugins/chora-tiling, chora-xp). The gate stopped there.

## Behaviour
Door acts and NET2 acts PASS (acts, doors steps). Visible change: the buttons say Yes/No in an English session, a click in the first 400 ms of a card does nothing.

## Not done / left
- mc line `? {question}  — ja / nein` (agent column, agent.rs ~595) is a literal German hint, not a button; goes to kit words later.
- construct hint line "ANSWER YES OR NO" and label "QUESTION" are English literals (kit `Word::Question` exists); a words pass.
- `moved(now)` is not called when the drawer or panel re-places the card: the guard counts from the question's first appearance, as in construct's old redraw (it redraws only on a new question).

## Refactor proposals, ranked
1. Retire the two thin wrappers (`mc::Confirming`, `construct::Ask`) by letting kit `Card` carry a look-owned extra (scroll) and a `Card::node_button(node)` map; then both looks are pure drawers.
2. Move the `confirm.answer` call itself into kit (`Card::answer(&mut Conn, ...)`) so no look names the method (the copies row `confirm.answer` goes to the assistant and the voice service only).
3. kit words for the remaining literals: construct hint, label, mc ja/nein line.
4. `Answer::Always`: when the voice service takes `always` (grants.md 112-120), add a third `Button` and a grant-question flag on `Question`; the serialisation is already in place.
5. Call `Card::moved` from the one place that re-places a card, once more than the assistant has a movable card.

## Review r1 (opus, PM, 11-18 08:5x): FIX
MAJOR: construct sh-26 guard starts when the question arrives (voice.rs:43), not when draw_ask draws;
Card::moved never called; remade card on resize/restyle keeps the old time. MINOR: mc guard not reset
on re-place; construct hint literals (hud.rs:1259-1261) and mc "ja / nein" (agent.rs:564) not words;
mc 400 ms cell uses the wall clock; agent.question shown unsanitized in set_agent (pre-existing,
ledger). COSMETIC: show_answers doc, configuration.md wrap. REFACTOR: keys()/buttons() twice (into
kit::Question); side effect in a match guard (construct input.rs:315). Clean: no guard lost
(sanitize in ask.rs:91-103), Always unreachable, ids matched, red first.

## r2 (sonnet, 11-18)
Commits on claude/kit4: 1b671db4 (red: cells + stubs), 78c3ffbc (fix), 7b762ea0 (test: no Ja/Nein literal in the cell, copies row).
1. MAJOR sh-26: `Voice::drawn(now)` calls `Card::moved`; `draw_ask_at(now)` calls it after the apply that really drew (draw_ask returns early while hidden: no move). Cell `a_press_inside_400_ms_of_the_draw_is_refused`: asked at t, drawn t+5 s: press +100 ms TooSoon, +400 ms counts; a second draw restarts it.
2. mc: `show_answers(had, now)` calls `moved(now)` after the answers are placed; `confirm_at(data, now)` injects the clock. Cell `a_press_after_the_drawer_is_placed_again_is_refused`. (Red state of this cell lacked a scene, so it failed on the drawer assert too; fixed in the green commit.)
3. sh-27 hints: construct hint from Word::NoVoiceYes / Word::Yes+No (uppercased), mc `question_line` from the same words. Cells `the_hint_follows_the_session_language` (construct), `the_agent_question_line_follows_the_session_language` (mc).
4. `a_press_inside_400_ms_of_the_card_is_refused` now uses `confirm_at` with an injected `now`.
5. show_answers doc says Yes and No (Ja and Nein in German); configuration.md wrapped.
Red first: 2 construct + 2 mc cells failed on 1b671db4; all green on 78c3ffbc (mc 136, construct 190, kit 217).
## Merge gate (once, on 7b762ea0, --jobs 3, /tmp/merge-gate.u48pZW)
PASS red-first .. doors (incl. acts, last-words, xp-*, hud, layout, tiling); FAIL `pull` (8 s): "the pane of the pulled window is not in front of the face at 1:1 or nearer: 197 px wide for 200", a cube/pull timing cell (the F13/F14 flake family of flakes-r2), not touched by this diff. Steps after pull not run; e2e (F27) not reached. An earlier gate on 78c3ffbc stopped at cut-check (copies row: my new cell had a "Ja"/"Nein" literal), fixed in 7b762ea0.
Not done: coverage (llvm-cov still not installed).
