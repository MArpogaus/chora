# review co-close r1 (rc 1: co-75 BLOCK, co-76 MAJOR): seeded review of claude/co-close-r1 before its merge

Model: opus, effort high, because the diff changes construct close and focus rules, xp's answer to another client's close (D31) and a gate's pixel compare; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/co-close-r1 head 8517022f, base 280b9dcf (14 files, +349 -34). One round.
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-co-close-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, EXIT 0, 64 PASS).
Read briefs/co-close-r1.md (done criteria), reports/co-close.md "## r1", reports/rc1-pass.md "## r1 construct" (rpc1-1, rpc1-2), findings/construct.md co-75, co-76 on claude/session-log; `git diff 280b9dcf..8517022f`.
Checks, each with its command and files:
1) Red first: f5570063, 96927adc, 5e42478a (tests only) before 272940fc; 090d17cf, 8517022f test only (`git log --oneline --stat 280b9dcf..8517022f`). A fix with no red test before it is MAJOR. Does 8517022f (png-same.py --outside) weaken K1, or only exclude the resting chip?
2) co-75 (plugins/chora-construct/src/adopt.rs close_focused, keys.rs): Super+x closes exactly the window seen focused: in a face the core focus, never a pane, nothing with no focus; elsewhere the focus-ring pane, never one behind the stood-back cube. Any path left where Super+x closes a window not shown focused, or closes nothing while one is shown focused? `cargo test -p chora-construct`.
3) xp side (plugins/chora-xp/src/wm/dispatch.rs): answers node.requested close only for its own windows, as tiling answer_request does; no other look's close changes; xp's own close button and Alt+F4 unchanged. `cargo test -p chora-xp`.
4) co-76 (hints.rs resting_set): a flown-in pane keeps Step back · All keys after 3 s; both chips click; the chip never covers the pane's work area more than a face's chip does (vision: minimal HUD); still hidden while the key list shows.
5) Beyond the brief: docs/configuration.md, construct-style.md, construct.yaml match the code (POLS); gate-construct-close.sh has no new sleep beyond the ratchet and asserts both rpc1 paths; merge-gate.sh change only adds the cell.
Known, not findings: co-77, co-78 (MINOR), co-79 (MAJOR, next round), co-44, co-45.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rcc1-N, severity,
file:line, text, fix in one sentence). Append to reports/co-close.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
