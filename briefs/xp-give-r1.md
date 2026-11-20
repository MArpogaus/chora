# xp-give r1 (RC 2, P2 xp-T7): a visible mouse way in xp to hand a window to another look

Model: sonnet, effort medium (one small menu entry on an existing action, with cells)
Lane: cloud, branch claude/xp-give at ce2c436b (session branch after train 89b). Stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "xp-give BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read: backlog.md xp-T7 and reports/p2-r5-mc.md (nurse T7: Control Panel, Desktop, Display grey, 3 rounds) on claude/session-log; on the branch plugins/chora-xp/src/context.rs (window menu Restore/Minimize/Maximize/Close, the disabled "Send To" of file rows), plugins/chora-xp/src/help.rs and keysheet give_row (Super+Alt+1-4), tests/gate-handover.sh, docs/api.md `window.give`.
Problem: xp gives a window to another look only by the give keys; a mouse user finds no way (T7 passes vacuously three rounds running). The other looks have their own doors; xp has none.
Items:
1. Red first (own commit, cell fails on ce2c436b): a cell (unit in context.rs plus one step in tests/gate-handover.sh or an xp gate) that opens the window menu of a window (title-bar right click and Alt+Space), picks "Send To" > a look, and asserts the window is owned by that look (I1), inside an output (I2), and gone from the xp taskbar.
2. Fix: a "Send To" submenu in the xp window menu, XP style (same menu art as the file "Send To"), one row per running look other than xp, by its plain name (Tiling desktop, Media center, 3D space; take the names the key list uses), calling the same give as Super+Alt+N. A look that is not running: row greyed, no error. No new API; no change to other looks.
3. Docs: docs/configuration.md or the xp section where the window menu is described; the key list row stays.
Poll the gate in the same turn; never end a turn to wait.
Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,gate-lib,xp,xp-fields,xp-question,doors` plus the handover cell you touch; all PASS.
Done when: red sha and fix sha on claude/xp-give; gate lines pasted in reports/xp-give.md "## r1" with a capture of the open submenu (path under media/2026-11-20/); report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Not in scope: the volume slider drag, the XP-MAJORS rv-1..4 test gaps, Control Panel rows (backlog).
Rules: push claude/xp-give and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
