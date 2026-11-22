# Review split-b r1 (PM, pure-move check)

Base..head: 3b14c825...2af7801c; services/voice only, 6 files, +568 -517.
Builder: session_016iS3kQcGNziaNKrScfwCsx (sonnet); reviewer: PM (not the builder).
Method: `git diff --color-moved=zebra 3b14c825...2af7801c -- services/voice`, non-moved +/- lines listed.

## Checks
1. Done criteria (brief split-b-r1.md): core.rs 1420 to 1136, agents.rs 1410 to 1189 production lines; both under 1,500. PASS.
2. Pure moves: every non-moved line is a mod/use line, a module doc line, an `impl Core {` wrapper, `pub(super)` on moved items, or rustfmt line breaks of signatures that grew by `pub(super)`. No body line changed. PASS.
   New files: agents/permission.rs (Permission, question, outcome, is_physical, unshowable, detail, asked, plan_line), agents/text.rs (cut, redact, redact_texts, redact_line, title, summary; `pub use text::{summary, title}` keeps the public path), core/confirm.rs (ask .. answered), core/frames.rs (played_level .. interim).
3. `cargo test -p chora-voice` green (worker report r1, 81bbe7c7).
4. Gate: incomplete (2 h tool cap inside `keys`): 39 PASS, none red; not run: keys, pixels, mc-sofa, mc-column, mc-handover, launcher-click. A voice-only pure move cannot change those cells; train 109 runs the full gate on the merge. Open until t109 green.

## Findings
- sb-1 NOTE: redact_texts and redact_line are pub(super) but used only inside text.rs; could stay private. No action for the RC.
- sb-2 friction: the lane gate hit the 2 h tool cap; briefs now say setsid nohup for the gate (log-down-mc brief already does).

Verdict (computed): 0 MAJOR open: MERGE-OK; joins train 109.
