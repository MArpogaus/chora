# review tiling-focus r4 (rc 1, 2; ti-57): seeded review of claude/tiling-focus-r4 before its merge

Model: opus, effort high, because the diff adds a shared kit focus hook (Focus::refresh_node) used by tiling and mc under an exclusive layer, where a wrong rule sends keys into a client the person does not see; the builder never reviews itself (rules.md 7)
Red on head: none (a review: reads claude/tiling-focus-r4, writes no code)
Lane: cloud, read-only on code: claude/tiling-focus-r4 head 7ef97ab8, base 3be27e32 (5 files, +200 -49). One round. Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it: 63 PASS, 1 foreign FAIL gate-tiling-notice.sh:31 socket, rerun steps PASS).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-tiling-focus-r4 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/tiling-focus-r4.md (done criteria), reports/tiling-focus.md "## Review r3" and "## r4" on claude/session-log; `git diff 3be27e32..7ef97ab8`.
Checks, each with its command and files:
1) Red first: 21cb4101 before 7ef97ab8; the new gate-looks-move.sh step 2b (picker open, Escape on the empty desktop, then Super+2/1) fails on 3be27e32 by its logic (read, no gate run); `cargo test -p chora-tiling` names the unit that was red.
2) crates/chora-kit/src/wm/focus.rs Focus::refresh_node: one shared rule (no per-look copy left in mc or tiling); it never takes the keyboard while a layer is exclusive and a client the person cannot see would get it; xp's path (key_node = owner of each focus.node) is untouched and still right. `cargo test -p chora-kit`.
3) plugins/chora-mc/src/main.rs (56 lines changed): a pure move to the kit hook, no behaviour change in mc (mc refresh before and after; rc1-pass r2 mc results must still hold, esp. no stray keys, mc-60).
4) plugins/chora-tiling/src/main.rs: the "except while its key list shows" exception is right (Super+F1 list open, then the picker; Escape order); no new sleep or timer (tools/sleep-ratchet).
5) docs/api.md matches the code; D119 unchanged is correct (or name the sentence that should change).
6) ti-57 path from the P2 r12 persona (L3 run 2 holders [] I3): would it now hold, by the code? Name any other path into the picker (mouse on the tag bar, keys) not covered.
Known, not findings: FLAKE-NOTICE-SOCKET (new foreign flake, PM files it), sh-37..sh-42, rpm2-6.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtf4-N, severity, file:line, text, fix in one sentence). Append to reports/tiling-focus.md on claude/session-log under "## Review r4".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
