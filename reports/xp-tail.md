
## r1

rv9kb-2, the balloon tail ends over the paperclip head.

- Base 62a04be03, head e4157dbd2 (claude/xp-tail-r1).
- Red 522a8cd15: two unit asserts (greeting `place`, chat `Frame::tail`) at 1280x800 and 1280x720: "the tail tip 1174 is not over the paperclip head" (clip x 1213..1256).
- Fix cbd7d9ebc, e4157dbd2: the tail is no longer clamped into the balloon; a tip right of the box leans out of the corner as a node of its own (`hud-line-tail`, `hud-chat-tail`, input pass), so the balloon keeps its place and its node does not cover the bin (the first try as one wider picture went red in xp-readable bin-clear). docs/xp-style.md row 11 updated.
- Diffstat: 5 files, +224 -36 (bubble.rs, assistant.rs, chat.rs, chat/view.rs, xp-style.md).
- Captures: media/2026-11-25/xp-tail-r1/ x1-greeting and x5-chat, before and after.
- Gate: `tests/merge-gate.sh --jobs 2` ... `e2e PASS 290s`, "all steps passed". An earlier run went red once in `cube` (co: "first probe window has no keyboard"), which passed twice alone and in the second full run (flake, not in the balloon code).
