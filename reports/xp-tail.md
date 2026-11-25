
## r1

rv9kb-2, the balloon tail ends over the paperclip head.

- Base 62a04be03, head e4157dbd2 (claude/xp-tail-r1).
- Red 522a8cd15: two unit asserts (greeting `place`, chat `Frame::tail`) at 1280x800 and 1280x720: "the tail tip 1174 is not over the paperclip head" (clip x 1213..1256).
- Fix cbd7d9ebc, e4157dbd2: the tail is no longer clamped into the balloon; a tip right of the box leans out of the corner as a node of its own (`hud-line-tail`, `hud-chat-tail`, input pass), so the balloon keeps its place and its node does not cover the bin (the first try as one wider picture went red in xp-readable bin-clear). docs/xp-style.md row 11 updated.
- Diffstat: 5 files, +224 -36 (bubble.rs, assistant.rs, chat.rs, chat/view.rs, xp-style.md).
- Captures: media/2026-11-25/xp-tail-r1/ x1-greeting and x5-chat, before and after.
- Gate: `tests/merge-gate.sh --jobs 2` ... `e2e PASS 290s`, "all steps passed". An earlier run went red once in `cube` (co: "first probe window has no keyboard"), which passed twice alone and in the second full run (flake, not in the balloon code).

## Review r1

Seeded review of claude/xp-tail-r1, base 62a04be0, head e4157dbd (5 files, +224 -36). One round, read-only on code.

**Verdict: MERGE-OK** (0 BLOCK, 0 MAJOR, 2 MINOR, 1 COSMETIC).

Checks:
1. Red first: 522a8cd1 (test) before cbd7d9eb and e4157dbd (fix). `cargo test -p chora-draw -p chora-assistant` at 522a8cd1: 2 failed ("770: the tail tip 1174 is not over the paperclip head", greeting and chat); at e4157dbd: 260 + 37 passed, 0 failed. Pass.
2. Tail: `place` keeps `left` unchanged (the balloon stays at its 62708d35 place; the captures show the box unmoved), the tail is only unclamped; the tip lands at x about 1232, y about 657, on the clip head (1213..1256) in x1 and x5 at 1280x800; 1280x720 is covered by the unit asserts (work area 690) only, no capture. Both tail nodes are `InputMode::Pass` (no hit area over the bin, clip or a window). Lifecycle: `hud-line-tail` is a child of `hud` and is hidden by `hush`, faded with the balloon, hidden when a later balloon does not lean or is plain, and goes with `Assistant::remove` (look switch, hide); `hud-chat-tail` is a child of `hud-chat-view`, which `View::commands` removes on every redraw and on close. Pass, but see rxt-1 (seam).
3. Other looks: every other `Style::shape` caller passes 0 or `tail_x` (clamped to `width - radius`), so `tail_inside` holds and they draw as before; tiling and neutral greet with the tip inside the box (dot 24 px from the edge, area full width), mc and construct use a plain line and plain/glass chat (no `lean`). docs/xp-style.md row 11 matches the code. Pass.
4. No new sleep, no new lint allow in the diff; tools/sleep-ratchet ok (778/929), tools/size-ratchet ok (0 over 1500), tools/lint-ratchet ok (allow 360). Pass.
5. Captures (media/2026-11-25/xp-tail-r1/): before/after differ only in the box (1157,639)-(1252,789): the tail and the tray clock; nothing else moved. The chat (x5-after) has no seam: row 639 is fill from x 1161 to 1174. The greeting (x1-after) has one: row 639 is the balloon's black bottom border all across the tail base (rxt-1).

Findings:
- rxt-1, MINOR, plugins/chora-assistant/src/assistant.rs:1119, the greeting's leaning tail is created before `hud-line` and so drawn under it: the balloon's bottom border runs as a 1 px black line across the tail base (x1-after row 639, x 1161..1174), a seam the chat tail (created last in `panel`) does not have; z-order also depends on history (a tail first created after an existing balloon lies on top). Fix: push the `hud-line-tail` command after the `hud-line` Create/Update in `speech_node` (or raise it with a Move), so the tail fill always covers the border at its base.
- rxt-2, MINOR, plugins/chora-assistant/src/assistant.rs:1751, no test holds the new node: nothing asserts that `hud-line-tail` is created with input pass, hidden by `hush`, faded with the balloon, or hidden by a following non-leaning balloon, nor that `hud-chat-tail` comes out of `panel`; the red asserts check only the tip's x, not its y over the head. Fix: add one unit test over `say`/`hush`/`tick` commands for `hud-line-tail` (and one `created(.., TAIL_NODE)` in view.rs) and assert the tip y equals the head y.
- rxt-3, COSMETIC, crates/chora-draw/src/bubble.rs:131, the leaning tail has its 16 px base on the bottom edge and its tip about 57 px right and 18 px down, so it renders as a thin sliver (2 to 6 px of fill) that reads as a line rather than a balloon tail (v8: a full wedge). Fix: move the base to the right end of the bottom edge and widen it (or start the near edge on the corner arc) so the wedge keeps a visible body.

Known, not findings: rv9kb-1, rv9kb-3.., rv9j-1 pager, the cube probe-keyboard flake.
