# security a r4 (rc 6): close the injected-input class in the type

Branch `claude/security-a` at 250b3716 (the session branch was merged into it already, at 85476db0). Pushed.

## The type

Events of `crates/chora-scene/src/event.rs` that a person causes, with `injected` after this round:
`input.key`, `node.key`, `pointer.pressed`, `pointer.moved`, `pointer.released` (before), and
`input.button`, `input.moved`, `pointer.entered`, `pointer.scroll` (new; field absent in JSON when false).
Without the flag, on purpose: `pointer.left` (it only clears a hover mark; a dropped leave would leave a
stale mark), and the events that no person causes (surface.*, focus.changed, node.requested, ...).
Core: `input.button` is flagged by the press of its grab (`ButtonGrab.real`), also for the release, `move` from
`xdg_toplevel.move` and a stray release; `input.moved`, `pointer.entered` and `pointer.scroll` by `state.injecting`.
Plugins: tiling and xp through `hears` (kit `from_person` knows the four new events); construct, mc, notify and
the assistant through the new `chora_kit::runtime::hears_pointer` at the top of `event`.
The compiler named only tiling (`PointerEntered { node, moved }`); the other sites match with `..`, so the
optout check listed them (28 lines, each annotated or turned into a drop).

## Items, earliest gate

| item | red | fix | earliest gate |
|---|---|---|---|
| 1 core: injected Super + click | 4d917c6a: `an_injected_click_with_an_injected_modifier_reaches_the_button_grab_flagged` left `[(input.button,false),(input.moved,false),(input.button,false)]`; `an_injected_move_and_an_injected_scroll_reach_the_owner_flagged` left `[(pointer.entered,false),(pointer.scroll,false)]` | 0a3224b8 | chora unit test |
| 1 tiling follow_mouse | de8fcbbe: `an_injected_pointer_move_does_not_move_the_focus_with_follow_mouse` focused s2, wanted s1; `an_injected_button_grab_drags_no_window_in_tiling` | 0a3224b8 | tiling unit test, then gate cell |
| 2 type, plugins | (as above) | 0a3224b8 | compiler for tiling, optout check for the rest |
| 3 optout.py | no red commit: the hook runs guards.sh on a commit, so red could not be committed. Red run before the fix: 3 FAIL in guards.sh (`injected: _inj`, `text: Some(injected)` twice), the 10 new cases otherwise green | cbdac78f (fields read at their own depth; `_`-names refused; `..` counts only when no field `injected` is named) | guards.sh, pre-commit |
| 4 wiring: xp | n/a (guards exist); mutation: deleting `hears` in `Wm::event` fails 2 tests | 2121057b (`Wm::start_on`, fake core) | xp unit test |
| 4 wiring: tiling | mutation: deleting `hears` in `Tiling::event` fails 3 tests | 0fa52f90 | tiling unit test |
| 4 wiring: construct | mutation: `walk_key(.., true)` or no `hears_pointer` fails the test | 793458e6 (`Construct::new`) | construct unit test |
| 4 notify | mutation: dropping the guard fails | e465c5f8 (private dbus-daemon + fake core; returns early without dbus-daemon, it ran here) | notify unit test |
| 4 assistant ROW, CLOSE, helper rows, figure | failed before the fix: `every_button_of_the_chat_refuses_a_press_of_a_program` (hud-row-chat), `a_press_of_a_program_on_the_figure_neither_opens_nor_hides_the_chat` | 70f8d1f2 | assistant unit test |
| 4 gate script | | 13f8737b: Tab once, retry only the wait; new cells "tiling with follow_mouse" off/on, green (with `hears` removed from tiling the click cell fails: "an injected click focused s1") | matrix step (gate-injected-presses.sh) |
| docs | | 4dea8a7e configuration.md, api.md | |
| hud gate | | 250b3716: see deviation 2 | hud step |

## Checks

- clippy --workspace --all-targets --locked -D warnings clean; fmt clean; injected-optout.py and guards.sh green.
- cargo test --workspace --locked with the env unset and with CHORA_TEST_ACCEPT_INJECTED_KEYS=1: all pass
  (the two host-bound failures of r3 did not occur in this container).
- tests/merge-gate.sh, second run (after the hud fix) passed every step, 250b3716, log dir /tmp/merge-gate.Y0Uyjt;
  the first run stopped at `hud` (deviation 2). Skips inside e2e, all listed in tests/e2e-skips.txt:
  desktop-file-validate, fkeys option, pipewire, xwayland-satellite, thunar. shellcheck is not installed here
  (the pre-commit hook ran it in its own env).

## Deviations and open

1. The red commit of the assistant item was refused by commit-message-length (subject 73) and I had pushed the
   next commit before I saw it; the test and the fix are in 70f8d1f2 (red-first accepts a test in the same
   commit). The red output is above.
2. `refuses_injected` now covers ROW, CLOSE and helper rows, as the brief says, and the hud gate clicked the row
   and the figure with injected clicks (chat-row-click-opens, typed-yes-held, xp-one-balloon failed). The cell
   `chat-row-click-opens` is now `chat-row-injected-click-refused`; the other two open the chat with chat.show.
   No gate cell opens the chat by a person's click on the row any more; the hud gate has no real press
   (a second hud session with --test-injected-press-is-real would add it). Backlog.
3. mc and the assistant `event` have no driven wiring test (no harness for the struct): their guard is
   `hears_pointer` (predicate covered through the kit test `from_person_knows_every_event_with_an_injected_flag`
   for the flag, not for the call). Backlog if the reviewer wants it.
4. Not in scope, untouched: review point 9 (COSMETIC, xp last_move coalescing).
