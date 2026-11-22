# review tour-fix r1 (F7, rc 11): seeded review of claude/tour-fix before its merge

Model: sonnet, effort high (test and tour harness only, no product code; design judgment on check faults)
Lane: cloud, read-only on code: claude/tour-fix head faf958fe, base 892d035c (10 files, +485 -730; demo/tour/{chapters/mc.py,
run.py, tour.py}, tests/acts/{harness.py, invariants.py, known-red.toml}, tests/{test_acts.py, test_tour.py, tiling-audit.py,
tour-diff.py}). One round. Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure:
send_message the PM "review-tour-fix BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/tour-fix-r1.md (done criteria) and reports/tour-fix.md "## r1" on claude/session-log.
Checks, each with its command and files:
1) Dumps: tour.py writes `<act>.<capture>.png` and `.json` in the layout tests/tour-diff.py reads; red 0b14978a before
   green 82afbbc4; `python3 -m pytest tests/test_tour.py -q`.
2) Old build: red 40fdb206, green 21777e46; the degrade path (no scene.boxes, no notify state.get) runs the act and marks
   `baseline: none` with a reason (design/tour-v9.md section 6), never a silent pass. Read the code; no replay needed.
3) Each check fault is real, not a weakened check, with its red commit first (rules.md 8): regex a222e45b/db187a87; xp 11 px
   floor and tray exemption 4b27cf2f/65560447 (justified by design/product-briefs.md:49? the exemption scoped to xp only?);
   boxes a4e57870/b1317554; tiling inner gap 10 b86e3ffb/98cee862 (design/looks/tiling-v2.md:17 vs the 4 px grid of
   design/looks/tiling.md, see ti-31); mc notify assets 0d26d8f6/e21d21f4; card xp-sized d1a26eb6/8222044d.
4) known-red.toml lost 73 rows (and faf958fe a stale row): for a sample of at least 15 removed rows, the breach is gone
   because the check was wrong or the product is fixed, not hidden by a wider match. Name any removed row that hid a real breach.
5) Remaining 66 reds (xp 30, tiling 5, mc 2, construct 18, handover 11): each has a findings row (findings/<look>.md:
   xp-39..41, mc-55, mc-56, ti-30, co-44, co-45, sh-31) or a known-red row naming one; tiling t26/t28 (fixed waybar pixels)
   named as not caused by the lane: confirm.
6) `python3 -m pytest tests/test_acts.py tests/test_tour.py -q`; ruff via `pre-commit run --files <changed files>`; no file
   over 1,500 lines.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity, file:line, text, fix
in one sentence). Append to reports/tour-fix.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and
MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines;
never --no-verify; no system packages.
