# black-pane-r2 (RC shared bar "a visible way to start a program"): sd-h1 and sh-23 render half, option 1

Model: sonnet, effort high (one plugin's start-up path and one launcher; the PM reviews)
Lane: cloud, branch claude/black-pane at 2a1e2638 (r1: gate-tiling-empty-output.sh). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "black-pane BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/black-pane.md "## r1" (your finding and the three options), findings/shared-doors.md sd-h1, findings/shared.md sh-23, backlog.md KIT-SEC MINOR-1.
PM decision (r1 option 1): a bare tiling output or pane with no bar is a user path (pane.manager in a session without the tiling units), so it is a product fault, not only a tour fault. Fix:
1. Red first: tests/gate-tiling-empty-output.sh gains a mode that runs `chora-tiling --with-companions` with COMPANIONS=0 (no gate-started companions); it is red (pure black) before the fix. Commit it alone.
2. `chora-tiling --with-companions` starts flavors/tiling/waybar/waybar.sh and wallpaper.sh once, guarded by their pid files (no second bar when the session units or a gate already run them; a test shows one panel after two starts). No flag: behaviour unchanged, so the 15 gates and pixel baselines stay as they are.
3. pane.manager passes the flag when it starts tiling for a pane or a second output; docs/configuration.md (or the tiling doc) names the flag in one line.
4. Part of the guard: waybar.sh and wallpaper.sh use `: "${XDG_RUNTIME_DIR:?}"` instead of the mktemp fallback (KIT-SEC MINOR-1; a new dir per run defeats the pid guard).
5. Wire `tiling-empty` (outputs, panes and the flag mode) into tests/merge-gate.sh as a cell.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 first, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,layout,matrix,doors,tiling-look,tiling-audit,wm-pane,tiling-empty`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it. Do not touch crates/chora input.rs (stray-grab lane).
Done when: red commit before the fix, gate cells pass, captures before and after in reports/black-pane/r2/, reports/black-pane.md "## r2" on claude/session-log (per item: red sha, fix sha, test, gate cell, capture).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/black-pane and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
