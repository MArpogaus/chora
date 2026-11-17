# Review secfix c r2 (diff cd7a8946..75bc62e6, branch claude/secfix-c, 2026-11-17)

Verdict: MERGE-OK (0 BLOCK, 0 open MAJOR, 4 MINOR).
Tests run: the 3 touched tests at 0295f661 all fail (chat.rs:2577, client.rs:1127, main.rs:903),
at 75bc62e6 all 3 pass (`cargo test -p chora-voice --bins`). Merge gate not rerun (builder: passed).

## r1 MAJOR findings
1. FIXED. client.rs:228-236 ask_key builds the detail and returns Err(chat::unshowable) (too_long or
   hides) before any Input::Agent Ask; hold_security client.rs:750-757 answers the call REJECTED with
   the reason, logs it, drops the patch. Proof: client.rs:1135 a_security_change_that_hides_text_is_refused
   (OSC in record_dir, C1 in agent_cmd, >64 KiB). Deviation (b) accepted: a nested object goes through
   Value::to_string, which escapes C0 as visible `\u001b` (honest display); C1/bidi stay raw and hides()
   catches them, so the string field is the real attack surface.
2. FIXED. main.rs:530-534 hud(): no claimer and !mc::fits(prompt, detail, keys) (mc.rs:358, the same sum
   as question_text mc.rs:370) sends Input::Unshowable(id) instead of the report; core.rs:1167-1175
   answers no with the error earcon, only for the shown id while confirming. Proof: main.rs:879
   a_question_too_long_for_the_hud_is_refused (5 KB detail -> Some(false); short detail waits).
   Deviation (a) accepted: only Agent knows `claimed`, so main.rs is the right home.
3. FIXED. chora_kit::chat::hides (kit chat.rs:100) is exactly C0 minus \t\n\r, C1 80-9F, 200B,
   202A-202E, 2060-2064, 2066-2069, FEFF; chat.rs and agents.rs:889 use it, the old sanitize!=text
   check is gone. Proof: chat.rs:2561 hides_refuses_only_what_hides_or_reorders (FE0F emoji, ZWJ,
   ZWNJ, NBSP, LRM, \r\n pass; RLO, LRI, ESC, C1, ZWSP, BOM, NUL, U+2062 refused).
   Deviation (c) \r passes: accepted; r1 listed \r as a false positive, sanitize drops it on screen
   and it neither hides nor reorders visible text on the HUD (not a terminal).

## New findings
- MINOR deviation (d) main.rs:532/core.rs:1173: a chat question refused by the own HUD reaches the LLM
  as "The user declined"; the model may tell the user they said no, or retry the same long command.
  Not a confirm path (fail-closed), so not MAJOR. Fix with an answer enum (refactor 1).
- MINOR core.rs:1168 race: Unshowable(id) is dropped when show() put a newer question on the screen in
  between (chat confirm over a sub-agent note). The older sub question then stays pending without a
  screen; a later key answers it blind. Pre-existing (show() replaces a pending question), but the new
  guard does not close it: answer the sub question no when its screen is replaced.
- MINOR mc.rs:358 fits() counts bytes, not lines: 4000 bytes can hold hundreds of short lines; whether
  the HUD scene shows them all was not checked. Consider DETAIL_LINES-like cap for the own HUD.
- MINOR client.rs:1135 test asserts only "no question sent", not that ask_key returned Err nor that the
  RPC answer is REJECTED; a test through hold_security would pin the refusal path.
- No bypass found: every Effect::Confirm passes hud(); claimer path still gets the 64 KiB/hides gate
  upstream (chat.rs:202, agents.rs:881, client.rs:234); refusal paths answer false, never true.

## Refactor proposals
1. Answer channel Option<bool> -> enum Answer { Yes, No, Unclear, Refused(reason) } so the LLM and
   sub-agents hear why (S).
2. One gate chora_kit::chat::unshowable(detail) -> Option<Reason>; chat.rs:202 and agents.rs:881 now
   duplicate the refusal phrases (S-M).
3. Move the fits() check into the core (it knows the shown id and could know `claimed`) and drop the
   TOO_LONG fallback in mc.rs:375, which is now reachable only in a claim race (S).
