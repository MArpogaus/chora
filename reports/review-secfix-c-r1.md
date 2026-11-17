# Review secfix c r1 (cd7a8946), opus medium, 06:03 UTC 11-17

Verdict: FIX (3 MAJOR open). Sanitize and the 400 ms button guard are correct; red tests fail on base,
pass on head (voice 5/5, chora-mc 1/1, construct 1/1, one build).

## Findings
1. MAJOR services/voice/src/client.rs:225-238 ask_key (hold_security :726): a security settings.set or
   settings.reload is asked from chat::detail(patch) with no hides/too_long check (chat.rs:1146 returns
   early). sanitize drops a whole OSC body, so "Be nice.\x1b]<rules>" shows "Be nice." and applies all;
   U+009D in array keys (agent_cmd) too. Fix: too_long || hides -> answer no, tell the caller (as chat.rs:1182).
2. MAJOR mc.rs:351-370 deviation: without a claimed indicator (main.rs:533) a 4 KB-64 KiB detail shows only
   "(the command is too long to show here)" and Alt+y confirms blind. main.rs owns self.core and
   self.claimed, so it can answer no before reporting. Restore red test a_question_too_long_for_the_hud_is_refused.
3. MAJOR regression chat.rs:219 hides() = sanitize(d) != d (chat.rs:201, agents.rs:881) refuses normal text:
   U+FE0F emoji (⚠️), ZWJ, ZWNJ (Persian), LRM/RLM, NBSP, soft hyphen, \r, >2 combining marks. Fix: refuse a
   narrow set only (C0 but \n\t, C1, ESC, U+202A-202E, U+2066-2069, U+200B, U+2060-2064, U+FEFF), or escape
   controls visibly (\u{202e}) instead of stripping.
4. MINOR config.rs:599-625, agents.rs:836: prompt {args}/{name} raw scalars only sanitized for display;
   spoken prompt (Effect::Ask, core.rs:635) raw. Known issue.
5. MINOR core.rs:1162-1171: BUTTON_DELAY counts from show(), not the plugin draw; document it.

## Refactor proposals
- One showable gate chora_kit::chat::unshowable(detail) -> Option<Reason> for chat.rs, agents.rs, client.rs, main.rs (S-M).
- Split kit sanitize into strip_dangerous (refusal) and normalize (display), or escape_visible (S).
- One detail builder for chat.rs:168 and agents.rs:906; refusal phrases in config, not literals (M).
- Input::ConfirmButton answers an enum (Taken, NotOpen, TooSoon) (S).
- mc agent.rs:97-133, construct hud.rs:956-1000 to kit ask::Question and Card (M, DB6).
