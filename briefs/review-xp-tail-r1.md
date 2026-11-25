# review xp-tail r1 (F7, rc 1 known issues): seeded review of claude/xp-tail-r1 before its merge

Model: opus, effort high, because the diff draws a new node in a look (the balloon tail as its own node with an input pass) where a wrong rule covers the bin or eats clicks; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/xp-tail-r1 head e4157dbd, base 62a04be0 (5 files, +224 -36). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it; the train runs it whole).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-xp-tail-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/xp-tail-r1.md (done criteria), reports/xp-tail.md "## r1" and reports/tour-v9-judge.md "## r2b" (rv9kb-2) on claude/session-log; `git diff 62a04be0..e4157dbd`.
Checks, each with its command and files:
1) Red first: 522a8cd1 holds the failing asserts and comes before cbd7d9eb (`git log --oneline 62a04be0..e4157dbd`); run `cargo test -p chora-draw -p chora-assistant` at 522a8cd1 (red) and at e4157dbd (green).
2) Tail: tip over the paperclip head at 1280x800 and 1280x720 for greeting and chat (bubble.rs, assistant.rs, chat/view.rs); the balloon keeps its 62708d35 place; the tail node has no hit area that covers the bin, the clip or a window (input pass); it is removed with its balloon (no stray node after close, hide, look switch).
3) Other looks: the helper in tiling, mc, construct draws as before (bubble.rs callers; a tail left of or inside the box unchanged). docs/xp-style.md row 11 matches the code.
4) No new sleep, no new lint allow (`git diff --stat`); tools/sleep-ratchet, tools/size-ratchet, tools/lint-ratchet on e4157dbd.
5) Captures media/2026-11-25/xp-tail-r1/ x1-greeting and x5-chat before and after: tip on the clip head, no seam between balloon and tail, nothing else moved.
Known, not findings: rv9kb-1, rv9kb-3.., rv9j-1 pager, the cube probe-keyboard flake the lane named.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rxt-N, severity, file:line, text, fix in one sentence). Append to reports/xp-tail.md under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
