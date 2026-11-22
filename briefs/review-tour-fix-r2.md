# review tour-fix r2 (F7, rc 11): seeded review of claude/tour-fix r2 before its merge

Model: sonnet, effort high (test and tour harness only, no product code; checks fixes of r1 findings)
Lane: cloud, read-only on code: claude/tour-fix head 87fdbeb9; lane diff = `git diff 1c9e64f7 87fdbeb9` (14 files,
+858 -840: demo/tour/{chapters/*.py, intended.toml, run.py, tour.py}, tests/acts/{harness.py, invariants.py,
known-red.toml}, tests/{test_acts.py, test_tour.py, tour-diff.py}); r2 commits faf958fe..6343c36e. One round. Stop at
120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the PM's train runs it).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure:
send_message the PM "review-tour-fix-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Read reports/tour-fix.md "## Review r1" (rw-1..rw-8) and "## r2" on claude/session-log.
Checks, each with its command and files:
1) rw-1 closed: red ba72f129 fails on its parent, fix a53c835f; tour.py `dump` takes the picture again except the H50 frame
   of this take; a failed capture leaves no file. `python3 -m pytest tests/test_tour.py -q -k dump`.
2) rw-2 closed: red e55aa1e9, fbbab3d7; fix 9478ea71; every act names a scene of design/tour-v9.md, Send To is h2, every
   `act` of demo/tour/intended.toml is a scene used by a chapter. Read tour.py and the five chapter files.
3) rw-3 closed: red 9262e8ba, fix 8e665374; OldCore and the drop of "missing boxes" only with TOUR_BASELINE; the
   candidate keeps "missing boxes" red. No other path that turns a box invariant green on nothing.
4) rw-4 closed: red 7d7fdaf7, fix dd8505d1; only a NO_METHOD RuntimeError is `baseline: none`; baseline-none.json
   `names` matches what tests/tour-diff.py reads (`baseline_none`).
5) rw-7 closed: red f245d0b9, fix 14f34409; tests/acts/harness.py `settled_scene` says so on stderr after the last try.
6) rw-5: 6343c36e reverts b86e3ffb and 98cee862 exactly (tests/tiling-audit.py back to the 4 px check). PM decision
   11-22: the inner gap 10 of design/looks/tiling-v2.md holds; a later round exempts it; not part of this verdict.
7) Counts: 66 -> 84 red; the +18 are tiling I2 (ti-31) only; the other new lines (tiling I6 +2, handover I3 +1) have rows.
   No known-red row widened. `python3 -m pytest tests/test_acts.py tests/test_tour.py -q`; ruff via
   `pre-commit run --files <changed files>`; no file over 1,500 lines.
8) ti-37 (tiling-look clock 120 wide): confirm no file of the lane diff touches tests/gate-tiling-bar-spec.sh,
   tests/clock-width.py, flavors/tiling or crates/; the PM fixes it in train 105 (clock-width).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv-N, severity, file:line, text, fix
in one sentence). Append to reports/tour-fix.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and
MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines;
never --no-verify; no system packages.
