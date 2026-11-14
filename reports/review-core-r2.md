# Review core r2 (claude/core-r2 eff84702..78f28b2d), opus, read-only, 11-14 19:1x UTC

Verdict: FIX. C05, C14, C13, CR-1, CR-6, CR-4/5 match; red tests before each fix.
| id | sev | file:line | text |
|---|---|---|---|
| V02 | MAJOR | services/voice/src/core.rs:1202, :1211; asr.rs:84; chat.rs:1000 | info still logs the person's text ("over the reply: {text}", "queued: {text}", "asr: the prompt came back, dropped: {text}"); the info line per tool call logs arguments, so code_task tasks and sub-agent prompts land there. docs/voice.md (~780) claims none. No test for barge-in or queued paths. |
| C13 | MINOR | crates/chora/src/backend/tty.rs:~814 | cursor elements draw in front of the mark; a client cursor surface can cover it. docs/architecture.md:483 overclaims. Draw the mark in front of the cursor or force the named cursor while it shows. |
| C05 | MINOR | 81fbea47 | breaking for clients calling output.reserve/configure and the voice tools until granted; no BREAKING CHANGE footer, so the generated CHANGELOG misses it. |
