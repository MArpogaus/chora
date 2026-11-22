# tour-fix r2 (F7, rc 11): fix round after review r1 (FIX, 3 MAJOR)

Model: sonnet, effort high (tour harness fixes with red tests; scene-id mapping needs care)
Lane: cloud, claude/tour-fix at faf958fe (r1 head). Cost at most 0.8 %; stop at 250 k context. One area: demo/tour and tests/tour-*.py, tests/acts (harness only); no product code.
Why: reports/tour-fix.md "## Review r1" on claude/session-log (bf0a2d18): verdict FIX, MAJOR rw-1, rw-2, rw-3; MINOR rw-4..rw-7.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "tour-fix-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Each fix: red test commit first (it fails alone), then the fix commit (rules.md 8).
1) rw-1 (tour.py:569): a new take never keeps a stale png: clear the capture dir at take start or always overwrite; red test: a run dir with an old png gets the new one.
2) rw-2 (tour.py:68,451; intended.toml:10,15,20): dumps and intended.toml use one id: the scene id of design/tour-v9.md; red test: the h2 Send To row matches the Send To act (h5), not "Click Start".
3) rw-3 (tour.py:226,246-247): OldCore and the "missing boxes" filter apply only with TOUR_BASELINE; red test: on the candidate, an act with I5, I6, I8 or I9 breach prints it. Name the new red count per look and invariant; each new red gets a findings row (never a known-red row to hide it).
4) rw-4 (run.py:82-95): catch only the old-build errors; others fail loud; baseline-none.json holds the `<act>.<capture>` names.
5) rw-6: findings rows for tiling t26 and t28 (fixed waybar pixels) in findings/tiling.md on claude/session-log.
6) rw-7 (harness.py:708): settled_scene logs or raises after its last try, never silent.
7) rw-5 (tiling-audit.py:89-110, gap 10 vs ti-31 4 px grid): do not decide; revert to the design value of design/looks/tiling.md if the check changed it, and note the question in the report.
Gate: `python3 -m pytest tests/test_acts.py tests/test_tour.py -q`, `python3 demo/tour/run.py --check` (red count before/after), full `tests/merge-gate.sh --jobs 1` (no --only) green on claude/tour-fix merged with origin/claude/autonomous-dev-session-3w-i33pg7. A red cell you did not cause: name it with its log line; do not rerun to green. Run long steps in the background and poll in the same turn; never end a turn to wait.
Done when: rw-1..rw-4, rw-6, rw-7 closed with red and fix commits, rw-5 noted, the gate green.
Report: reports/tour-fix.md "## r2" on claude/session-log (shas, each rw line closed, red counts, findings rows); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/tour-fix and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report and findings/ rows); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
