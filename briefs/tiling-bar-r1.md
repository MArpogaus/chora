# tiling-bar r1 (RC look, vision "Hyprland-class looks"; ti-21 MAJOR): the bar in the tiling style

Model: sonnet, effort high (one look's bar to a written spec, with pixel asserts; the PM reviews)
Lane: cloud, branch claude/tiling-bar from 79c8d2b5 (session branch, train 81). One round, stop at 250 k. Cost about 1 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure or a missing tool (waybar, jq): send_message the PM "tiling-bar r1 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: findings/tiling.md ti-21 and the open rows; design/looks/tiling-v2.md (Grid, palette table, Type, Bar, the rows Keys, Apps, power); vision-card.md (chora-tiling: Hyprland-class looks, plain-text config). On the branch: flavors/tiling/waybar/ (config.jsonc, style.css, colors.css, waybar.sh), plugins/chora-tiling/src/theme.rs (palette files), tests/gate-tiling-look.sh and its pixel baselines.
Items:
1. Red first (own commit): pixel or geometry asserts in the tiling-look cell for the spec of tiling-v2.md "Bar": height 32, 8 from the top, 12 from the sides, radius 10, `surface` at 92 %, a 1 px `line` hairline, Inter 14 px weight 500, the words Apps and Keys and the power icon visible. Each assert fails on the head where the bar differs.
2. The fix in style.css and config.jsonc only (colours from colors.css, no literal colours; stays a hackable waybar rice); the palette of both themes (light and dark). Wobbly on for floating drags: only if it is a default in the tiling config already present; else name it in the report, no new feature.
3. Captures before and after (both themes, 1280x800 and 2560x1600@2) in reports/tiling-bar/; findings/tiling.md ti-21 row `fixed <fix sha> (red <sha>) cell:tiling-look` on claude/session-log.
Pixel baselines that change with the bar: update them in the fix commit and list each file in the report. Do not touch waybar.sh or wallpaper.sh (another branch changes them) or crates/chora.
Poll each run in the same turn; never end a turn to wait.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,tiling-look,tiling-audit,doors,layout,matrix`; all PASS. A red cell you did not cause: rerun it alone once and name it.
Done when: red commit before the fix, gate cells pass, captures, reports/tiling-bar.md "## r1" on claude/session-log (per assert: red sha, fix sha, cell, capture); head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/tiling-bar and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
