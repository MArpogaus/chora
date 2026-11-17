# decisions r5 (rc 8): proposed decision D95, packaging and repo cut

Model: sonnet, effort low (one entry in one file)
Lane: cloud, branch claude/decisions, continue from its head (bbf1264). One round, at most 1 hour.
Read answers.md lines "2026-11-17 04:43 UTC" and "04:46 UTC" on claude/session-log.
Add one entry D95, status "proposed (owner, 2026-11-17)", in the format of D87-D94, with the
revise-or-revert sentence. Content: after the tag each runnable plugin and service is its own package,
built as split packages from one PKGBUILD and one RPM spec; library code (chora-kit, chora-scene,
chora-draw) stays statically linked, not a package per module; flavor meta packages bundle a look with
its default plugins and config; one joint meta package holds all looks; until the API is stable (1.0)
each plugin package requires the exact core version. Repos: one repository until 1.0, then one plugin
repository, later a split per look or user scenario. The cut is kept now: a plugin uses only chora-kit,
chora-scene, chora-draw and the live API, ships only its own assets, and a gate checks both (to build;
known breach: chora-assistant installs huds/tiling, crates/chora/Cargo.toml:49). Check that line in the
code before you cite it. Nothing else. Docs only. Done when pre-commit and mkdocs --strict pass.
Report: lines appended to reports/decisions-r1.md, one send_message to the PM.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines;
never --no-verify; stage by name; no force push; no pull request. Push only claude/decisions.
