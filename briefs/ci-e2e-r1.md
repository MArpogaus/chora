# ci e2e r1 (rc 4): the e2e job of PR #106

Lane: sonnet, cloud, new branch claude/ci-e2e from the session branch
claude/autonomous-dev-session-3w-i33pg7 (4e3c019e). One round. Stop at 250 k context and hand over.
Read dev-rules.md and conventions.md (with the skills it names) first. Setup as briefs/ci-r1.md.
Failure: the e2e job of PR #106 (run 43643290788, job 132595355197, Fedora 44 container, headless
sway host): step "2. three foot windows appear without overlap" prints
"FAIL: windows overlap: e2e/tiled.json". Read the full log and the e2e-logs artifact with the
GitHub tools; run the same e2e script locally (it lives in .github/workflows/ci.yaml and the e2e
scripts it calls). Find the root cause: a real tiling regression (e.g. gaps, borders or shadows of
tiling v2 that make geometries overlap) or a test that reads the wrong geometry. Fix the cause with
a red test first; never loosen the overlap check without a reason that holds for users.
Another lane (ci r3) fixes the font tests of the same run; do not touch those.
Done when: the e2e script passes locally on sway; tests/merge-gate.sh passes. Report to the PM
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 10 lines): root cause with evidence, commits.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution
lines; never --no-verify; stage by name; no force push; push only claude/ci-e2e; never check out
another branch in an existing worktree.
