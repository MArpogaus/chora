# ci r2 (rc 4, rc 5, rc 7)

Lane: sonnet, cloud, new branch claude/ci-r2 from the session branch
claude/autonomous-dev-session-3w-i33pg7 (de55f346). One round. Stop at 250 k context and hand over.
Read dev-rules.md and conventions.md (with the skills it names; git-conventions "CI workflows" and
rust-dev "Check pipeline") first. Setup as briefs/ci-r1.md.
Items:
1. rc 5 (owner 11-14 16:47: packaging smoke in GitHub Actions): the rpm, arch and distrobox jobs of
   .github/workflows/release.yaml also run on pull requests to dev (build and install smoke only,
   no publish, no release), so the CI PR proves the packages. Keep one definition (a reusable
   workflow or a shared job file), no copy.
2. rust-dev hard rule: tests/merge-gate.sh runs cargo build, test and clippy with --locked.
3. fuzzel: flavors/tiling/fuzzel.ini uses selection-radius, which fuzzel 1.9 rejects (Debian and the
   cloud container ship 1.9.2; gate-tiling-power and gate-tiling-programs fail there). Make the
   config work on 1.9 and newer (drop the key or move it to an optional include); both gates green.
4. Every action pinned to a SHA (pinact; the grep check of git-conventions prints nothing).
Method: red first where a check failed; one commit per fix.
Done when: tests/merge-gate.sh passes; actionlint and pre-commit pass. Report to the PM
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 12 lines): commits, what only GitHub can prove.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution
lines; never --no-verify; stage by name; no force push; push only claude/ci-r2; no pull request
(the PM opens it); never check out another branch in an existing worktree.
