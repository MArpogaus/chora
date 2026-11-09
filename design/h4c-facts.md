# H4c: facts for the design check (2026-11-09 evening)

Found by the H4c developer before the stop; branch `claude/hud-h4c` at
ec39ba3 holds only the four tests (red: valid token, click on window,
named client; `a_layer_without_a_token_gets_no_keyboard_on_map` already
passes today).

- A click on an `on_demand` layer already gives it the keyboard:
  `click_focus` (input.rs:876) calls `keyboard_layer` and
  `set_layer_focus`; a click on a window takes it back
  (`set_layer_focus(None)`). Tests exist.
- No scene command gives the keyboard to a layer surface: `focus` and
  `focus_node` reach only toplevels and nodes. A main-socket plugin can
  do it today only by injecting a click.
- `token_created` keeps every token of an unnamed client, and a token
  of a named client only with the serial of its last press.
  `activation.token` (ipc.rs:888-897) mints a token with no client and
  no serial (#102), so a valid token proves no act of a person.

Questions for the opus design check (11-10 run 1):

1. Mouse path: a click on the field already works. Is the field shown
   under the pointer of the click on the "Chat ..." row enough, or must
   it take the keyboard without a second click?
2. Keyboard and voice paths (Super+a, "open the chat"): what proves the
   person's act? Candidate: the core records the serial of the last
   real (not injected) press or key, and a token is valid for focus only
   with such a serial; `activation.token` then mints only press-bound
   tokens (fixes #102). Voice: a spoken command is not a press; decide
   whether the voice service, a trusted client, may vouch for it.
3. The smallest core change that serves all three paths, with a test
   that a minted token without a press gives no keyboard.
