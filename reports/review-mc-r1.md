# Review: mc r1 (rc 1), claude/mc-fix at 4feb4dc0

Reviewer lane: opus, read-only. Diff: `git diff claude/autonomous-dev-session-3w-i33pg7...4feb4dc0`
(5 commits, 13 files). Brief: briefs/mc-r1.md; ledger findings/mc.md (mc-1, mc-3 to mc-8).

Gates (worktree /home/user/wt/mc):
- `tests/merge-gate.sh --red-first-check`: passes (7f8f4f80 tests before the three fixes; 4feb4dc0 has its unit test).
- `tests/merge-gate.sh --worker ''`: build, test, clippy PASS (211 s, 80 s, 90 s), exit 0.
- `tests/gate-mc-paths.sh` on the branch binaries: all 8 cells and the window audit PASS ("every path of the mc gate").
- Probe of the chat focus (MC-1): scratch script (session as gate-mc-paths): MC-1 reproduced.

## Fixes against their findings

- mc-1: plain Right/Left and Escape on a page (main.rs:1166 `page_key`, :1180 `back`); key list names `← →` on a
  page, `Super+ ← →` in a program scene. Matches, except with `--output` (MC-3).
- mc-3: the footer of page 2 already had `← 1  Agenda`; cell mouse-back passed on the red commit too. Verified.
- mc-4: word `Mute` on the state line (mic.rs), privileged `voice.mute` is reached over the main socket. Matches.
- mc-5, mc-6, mc-7: verified by the cells first-page, terminal-chat, all-keys.
- mc-8: `Urgent:` in column and footer, footer line in `margin` (>= 14 px), a click clears only the footer line. Matches.

## Findings

| id | sev | file:line | text | evidence |
|---|---|---|---|---|
| MC-1 | MAJOR | plugins/chora-mc/src/main.rs:2669 | `keyboard_to` gives the page node the keyboard on every page turn of the user, also while the chat field of the HUD host has it: with the chat open, Super+Right moves the keyboard from the chat to the page, so typed text is lost and plain arrows turn pages. Before, `Focus {surface: None}` left the node focus alone. `take_free_keyboard` checks for a focused node; `keyboard_to` does not. The cell terminal-chat checks only the place of the chat, not its keyboard | chora-scene scene.rs:1478 (`focus_node` overwrites `focused_node` of another owner), scene.rs:1438 (`focus` None keeps it); probe: chat opened with Super+a, plain Right leaves agenda (chat has the keyboard); Super+Right shows morning with the chat node still there; plain Right then shows agents |
| MC-2 | MINOR | tests/merge-gate.sh:136 | tests/gate-mc-paths.sh is no step of the merge gate (as gate-mc-column, -host, -scale are not); the new cells guard nothing after the merge | grep of merge-gate.sh and xp.sh |
| MC-3 | MINOR | plugins/chora-mc/src/main.rs:430 | With `--output` (every manager pane of a construct, the battle and handover gates) a page takes no keyboard, so mc-1 stays open there: the same page and footer, but plain arrows and Escape do nothing. The flag stands in for "beside other window managers"; documented in mc-style.md 16 and mc.yaml, but it is a design gap of D55 (a node focus routes new windows) for the PM | keys::PageKeys::Bound; tests/battle/w12.sh runs mc without --output beside tiling |
| MC-4 | MINOR | plugins/chora-mc/src/mic.rs:47, 96 | After `Failed` or `Refused` the word has no hit until the next `voice.state`; a person cannot retry Mute with the mouse until the agent changes state | `word()` returns `None` as the press for both |
| MC-5 | COSMETIC | plugins/chora-mc/src/mic.rs:49 | "Use your own mouse" reads as a reproach; the line should say what happened (for example "Only a click of a person mutes") | copy |
| MC-6 | COSMETIC | plugins/chora-mc/src/main.rs:1101 | Each press computes the items of the scene twice (`click_mic` and `click_hit` both call `hit_token`) | region.rs:707 |

## Verdict

MC-1 is MAJOR and open: **FIX**. Fix: in `keyboard_to`, skip the `FocusNode` while `self.chat == Chat::Open`
(or while a node of another owner has the focus, as `take_free_keyboard` checks), and add to cell
terminal-chat a typed text that must reach the chat field after the page turn.
