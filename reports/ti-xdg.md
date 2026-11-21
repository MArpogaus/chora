
## r1

- Red: 490c4e84 (tests/runtime-dir-selftest.sh, fails on a84effda). Green: 2024358f; head 2024358f (merge with origin/claude/autonomous-dev-session-3w-i33pg7 was already up to date).
- Files: flavors/shared/runtime.sh (the one helper, sourced), wallpaper.sh, waybar/waybar.sh, screenshot.sh (mkdir-ok comment: user pictures), dist/build-arch.sh (installs runtime.sh), tests/mkdir-runtime.sh (guard: flavors mkdir -p needs -m or mkdir-ok), .pre-commit-config.yaml (hook now covers tests/*.sh runtime paths), tests/guards.sh (red/green cases), tests/no-inline-comments.py (allows mkdir-ok:), tests/gate-tiling-bar-early-signal.sh (-m 700), tests/merge-gate.sh (runs the selftest in ci-check).
- Rule: without XDG_RUNTIME_DIR, ${TMPDIR:-/tmp}/chora-UID via mkdir -m 700; foreign owner, symlink or non-700 prints one line on stderr, exit 1.
- Gate: tests/merge-gate.sh --jobs 1, 49 cells PASS, no red cell.

### PM review r1 (17:52 UTC; a84effda..2024358f, 12 files +137 -10)
Checks against the brief:
- One helper, flavors/shared/runtime.sh, is sourced by wallpaper.sh and waybar.sh. It is installed by dist/build-arch.sh. Closed.
- The rule: ${TMPDIR:-/tmp}/chora-UID, made with mkdir -m 700. A symlink, a directory owned by someone else or a mode other than 700 exits 1 with one stderr line. Pinned by tests/runtime-dir-selftest.sh in ci-check (red 490c4e84). Closed.
- screenshot.sh has a mkdir-ok marker with its reason. The hook covers flavors/ and tests/ runtime paths. guards.sh has 3 cases: fail, pass with the marker, pass with -m 700. Closed.
- The full gate was green: 49 cells at --jobs 1, as the lane reported.
- tx-1 NIT runtime.sh:17: stat -c is GNU only. That is fine on Linux, the only target. Note only.
- tx-2 NIT: a set XDG_RUNTIME_DIR is trusted and not checked. This was true before the change, and pam/logind own that directory.
Open MAJOR: 0. Verdict: MERGE-OK. It touches .pre-commit-config.yaml, so it is a harness branch and goes into t103.
