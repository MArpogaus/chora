# handover-mo r1 (rc 2, shared RC bar "no stray keys", "a visible way to start"): the open MAJOR rows of the two-output handover, each red first then fixed, or proven gone

Model: opus, effort medium because the rows cross core input routing, focus and two look plugins on two outputs; the causes are unknown and earlier sonnet rounds left them open
Lane: cloud, claude/handover-mo at a5b77488 (session head after train 96b). Cost at most 1.5 %; stop at 250 k context. One area: the handover on two outputs. Do not touch construct focus (lane co-keys owns co-41 and the construct half of I3) or the tour harness (lane chapters-a).
Rows (findings/shared.md, findings/shared-doors.md, reports/chapters-a.md "Known rows" on claude/session-log):
- sh-23 MAJOR: xp and tiling in two panes, the tiling pane stays black; after a click into it Super+Return opens the xp Start menu in the other pane (keyboard focus stays with xp).
- sd-h1 MAJOR: an empty tiling output in a two-screen setup is pure black, no bar, no start door (likely sh-23).
- sd-h2 MAJOR: the xp Start menu stays open through a drag and key gives, covering the window.
- ca-4 (tour row, xp "Super held, drags on the desktop"): the Start menu opens when Super is released after the drag. Same family as sd-h2.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "handover-mo BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) For each row: a gate cell (extend tests/gate-handover-matrix.sh or a new small gate script wired into merge-gate.sh) that is red on a5b77488, committed alone first. A row that is not red in 5 runs: say so with the runs and its evidence; no code for it.
2) Cause with file:line, then the smallest fix; KISS, no new dependency, no new config key. The Start menu opens on a Super release only when no other key or pointer motion came between press and release (XP behaviour); an output a look holds paints its bar and start door at once.
3) Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,ci-check,build,test,clippy,xp,matrix,virtual,doors,acts` green (gate-lib is left out: its selftest fails in lane containers). A red cell you did not cause: name it with its log line; do not rerun to green. Poll in the same turn; never end a turn to wait.
Done when: each row fixed (red cell commit, then green) or shown not red; the gate green; the finding rows updated (state, sha) on claude/session-log.
Report: reports/handover-mo.md "## r1" on claude/session-log (shas, cause with file:line, cell per row, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/handover-mo and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
