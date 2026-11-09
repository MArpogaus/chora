# H4c: decision of the design check (2026-11-09, opus, read-only at 1e824a9)

## Recommendation: act-stamped tokens (option b, made exact)

What the core can promise: main-socket clients and unnamed Wayland clients are trusted (D33);
they can take the keyboard today (`focus`, an `exclusive` layer, `input.inject`). The rule must
add no new path for anyone else, and a token must prove a real act. Plain (b) proves nothing:
"bound to the last real press" is always true at mint time. Three additions make it a proof.

Core (lane R):
1. `State.last_act: Option<Act {serial, at, stamped}>`: the serial and fake-clock time of the
   last real key or button press, set in `input::key` and `press_button` only when
   `!injected`, grabbed keys and buttons included (Super+a is a grab). `last_press`,
   `last_input` and popup or move grabs stay as they are.
2. `activation.token` (ipc.rs:894) stamps the new token with `last_act.serial` (in
   `XdgActivationTokenData::user_data`) only if the caller is not confined
   (`!ipc.confined(conn)`), the act is younger than `ACT_AGE` = 2 s and not stamped yet (one
   act, one stamped token). Otherwise it still mints, unstamped: D42/D72 placement and
   `choractl run` from scripts and voice keep working. That closes #102: no token without a
   press claims a person's act.
3. `request_activation` (protocols.rs:258), before the toplevel lookup: for a layer surface
   that takes the keyboard on demand, of an unnamed client, not while locked: give it the
   keyboard (`set_layer_focus`) only if the token is stamped, `stamp == last_act.serial` (no
   real press since: the person did not start typing elsewhere) and its age is at most
   `MAX_TOKEN_AGE`. Otherwise nothing; never push a layer to `pending_activations`.

Plugin side (H4b, H7; no core): the host passes a token to the field only (1) after a
`pointer.pressed {injected: false}` on its own node (it mints at once), or (2) from
`chat.show {token}`: the opener `chora-chat` forwards its `XDG_ACTIVATION_TOKEN`. The field
calls `activate` on every show. `chat.show` without a token opens without the keyboard.

## The three paths

- **Mouse.** The real click on "Chat ..." or a row is the act; the host mints at once; the
  field takes the keyboard with no second click (UX F1 M says "with the field focused").
  A click on the field still works as today. Answer to facts question 1: no second click.
- **Keyboard.** Super+a (tiling grab, real) is the act. `chora_kit::run` → `choractl run`
  mints within ms: stamped. The opener hands the token to the host, the host to the field.
  If the person types before the field maps, the token is stale and the keys stay with their
  window; Super+a again focuses. The host keeps the field process alive between opens
  (it keeps the draft anyway), so the map comes fast.
- **Voice.** A spoken command opens the chat view and the field without the keyboard; the
  field shows "Click here or press Super+a to type." Reasons: any voice the mic hears starts a
  turn, also a video (voice README); D78 already says "a voice ... is no proof that the user
  agrees, a key or a click is"; and the harm is concrete: a field that takes the keyboard
  while the person types a command or a sudo password, and Enter sends it to an agent. A
  voice-only person (the nurse) does not type, so P2 holds: open, read, speak, close all work
  by voice. P11 lists a spoken command as a cause the person may use, not one the core must
  honour. `voice-mcp` is confined, so its mints are never stamped; the push-to-talk key is
  real but the host passes no token to a voice call, so the result does not depend on timing.
  If a speak-then-type persona later needs it: chora-voice vouches, stamped only when no real
  press came in the last 3 s and none since (a D row of its own). Not now.

## Options rejected
- (a) Host-only scene command `focus_layer`, gated on the name: main-socket names are chosen
  in `hello`, so the gate is no gate; layers have no scene id (new API in chora-scene); it
  proves no act and leaves #102 open. More lines, less safety.
- (c1) Map `exclusive`, then commit `on_demand` and let the core keep the keyboard: zero
  proof, a frame in which a click cannot take the keyboard back, and #102 open.
- Refusing `activation.token` without a press: breaks `choractl run` from scripts and voice
  `run_program` (D42, D72). Stamping keeps both.

## 2. The injected-press fix: same step, yes

It is the same rule ("an injected press is not a person's act") and the step adds `last_act`
in input.rs anyway. Two lines more, each with a test: `press_node` sets `state.pressed` only
if `!injected` (input.rs:793, focus_node and focus rule of clients.rs:331), and
`token_allowed` (protocols.rs:291) also needs `serial == last_act.serial`. Do not stop
injected presses from setting `last_press`: popup grabs and interactive moves of injected
clicks (persona, gates) need it, and it proves nothing once tokens and `pressed` use the act.
Keep `click_focus` for injected clicks: it is direct input of a client granted
`input.inject`, not a deferred claim, and persona tests click the field.
H4c's files become `input.rs`, `ipc.rs`, `protocols.rs`, `state.rs` (`layers.rs` unchanged).

## 3. Tests

Change the four tests of ec39ba3 (`state.rs`):
- `token()` becomes `act_token(core)`: a real key `crate::input::key(.., Keycode::new(38),
  Pressed/Released, .., false)`, then mint with `core.named("tiling")`. Plain `token()` stays
  for the guard tests below.
- `an_on_demand_layer_with_a_valid_token_gets_the_keyboard` → rename
  `..._with_a_token_of_a_real_press_gets_the_keyboard`, uses `act_token`. Red first.
- `a_layer_without_a_token_gets_no_keyboard_on_map`: unchanged (green today).
- `a_named_client_gets_no_keyboard`: mint with `act_token`, so the name rule refuses, not a
  missing stamp. Must stay green for that reason.
- `a_click_on_a_window_takes_the_keyboard_back`: `act_token`, and replace
  `set_layer_focus(None)` by a real click on s1 (`move_pointer` + `press_button(.., false)`).
  Red first.

New guard tests (green today, red under a naive "valid token" rule, green after the stamp):
- `a_token_minted_without_a_press_gives_no_keyboard` (#102)
- `a_token_minted_after_an_injected_press_gives_no_keyboard` (agent with `input.inject`)
- `a_press_after_the_token_keeps_the_keyboard_with_the_window` (stale)
- `a_token_minted_late_after_the_press_gives_no_keyboard` (`core.advance(3 s)`)
- `one_press_stamps_one_token` (mint twice, the second gives nothing)
- `a_confined_mint_carries_no_act` (client socket `agent` with `activation.token`)
- `a_locked_session_gives_the_layer_no_keyboard`
Injected-press tests (red today), `ipc/clients/tests.rs`: `an_injected_click_allows_no_focus_node`,
`an_injected_press_gives_a_client_socket_no_token`; protocols.rs: extend the `token_allowed` test.
Order: write the two red rule tests and the two injected tests; the naive branch turns the
first green and the guards red; the stamp turns all green.

Consequences for other steps: `tests/chat.sh` (H4b) cannot make a real press, so it checks
"an injected open gives no keyboard; an injected click on the field gives it"; the token path
is these unit tests, like the answer path of questions. Persona L2 and A-checks clicking
"Chat ..." by injection need one click on the field; UX word table gets the field line
"Click here to type." Docs: architecture.md focus list (+3 lines), D80 draft row, the doc
comment of `activation.token`, hud-sprint row H4c (files, tests, HELD removed).

## 4. Size and risk

Code about 60 lines (state.rs 10, input.rs 14, ipc.rs 12, protocols.rs 22), tests about
190, docs about 15. No render path, no scene, no window path changes.
- `tests/2d-guard.sh`: no risk; it compares pixels of scenes, and nothing here draws.
- Window handover gates (`gate-handover.sh`, `-matrix.sh`): low. They inject clicks and keys
  for main-socket window managers, whose `focus` is not gated by `state.pressed`;
  `voice-mcp` only calls `window.give`; `last_press` and popup or move grabs are unchanged;
  unstamped tokens still place windows (D72). Check: run both gates once on the branch.
- Behaviour: a fast typist after Super+a loses the auto-focus (by design, P11); a client
  socket that used an injected press for `focus_node` now gets code 4 (none in the tree).
