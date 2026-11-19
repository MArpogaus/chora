# ti-majors r1 (RC shared bar, vision card): three open tiling MAJORs ti-22, ti-23, ti-24

Model: sonnet, effort high (three product fixes in one look, each with a red cell; plan given)
Lane: cloud, branch claude/ti-majors from a61e8eaa (session head, train 76). One round, stop at 250 k. Cost about 1 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "ti-majors BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Read on claude/session-log: findings/tiling.md ti-22, ti-23, ti-24; reports/journey-1.md (T1, T2, T4); vision-card.md
("Shared RC bar": lock, power, screenshot reachable; honest failure states). The findings are from tour v4 (11-16):
reproduce each on a61e8eaa first. One already gone: close it with the cell that shows it, no code.
1. ti-22 Super+Print (and Print) give a notice: "Screenshot saved to <path>" or "Screenshot failed: <reason>"
   through chora-notify (shared/screenshot.sh, plugins/chora-tiling main.rs CHORA_SCREENSHOT). Red cell first:
   a tiling gate cell or act that presses Super+Print and waits for the notice; a second with a failing grim
   stand-in waits for the failure notice. Never run the host's grim outside the nested session.
2. ti-23 the bar tooltip "Run a program (Super+Space)" leaves when the pointer leaves the button or a key is
   pressed (POLS); red cell: hover, move away, the tooltip node is gone within 1 s.
3. ti-24 the bar shows the power and lock icon the key list names (keys.rs:739 "power icon: lock, log out,
   screenshot"); a click opens the power menu. The "Chat ..." pill sits in the bar, not over a client, and
   says the same helper state as the bar (no "No helper" beside a working chat). Red cell for each.
Each fix: its own red commit first, then the fix. KISS: the smallest change in the tiling plugin or shared/;
no core change; no new feature beyond the finding. Rows in findings/tiling.md: `fixed <sha> <cell>`.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,tiling-audit,tiling-look,keys,acts`
green. Before the final gate merge origin/claude/autonomous-dev-session-3w-i33pg7. Poll it in the same turn; never end a turn to wait.
Done when: 1-3 hold with the cells that show them; gate green; reports/ti-majors.md "## r1" on claude/session-log
(each claim with its command or cell and capture); head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/ti-majors and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
