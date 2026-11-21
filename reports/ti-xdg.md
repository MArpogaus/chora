
## r1

- Red: 490c4e84 (tests/runtime-dir-selftest.sh, fails on a84effda). Green: 2024358f; head 2024358f (merge with origin/claude/autonomous-dev-session-3w-i33pg7 was already up to date).
- Files: flavors/shared/runtime.sh (the one helper, sourced), wallpaper.sh, waybar/waybar.sh, screenshot.sh (mkdir-ok comment: user pictures), dist/build-arch.sh (installs runtime.sh), tests/mkdir-runtime.sh (guard: flavors mkdir -p needs -m or mkdir-ok), .pre-commit-config.yaml (hook now covers tests/*.sh runtime paths), tests/guards.sh (red/green cases), tests/no-inline-comments.py (allows mkdir-ok:), tests/gate-tiling-bar-early-signal.sh (-m 700), tests/merge-gate.sh (runs the selftest in ci-check).
- Rule: without XDG_RUNTIME_DIR, ${TMPDIR:-/tmp}/chora-UID via mkdir -m 700; foreign owner, symlink or non-700 prints one line on stderr, exit 1.
- Gate: tests/merge-gate.sh --jobs 1, 49 cells PASS, no red cell.
