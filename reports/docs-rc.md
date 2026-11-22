# docs-rc

## r1

Head 54c30d16 on claude/docs-rc (one commit on 6b01c976, which is already origin/claude/autonomous-dev-session-3w-i33pg7: the merge was a no-op). Tools: vale and lychee present; pre-commit passed (vale, lychee, typos).

- CHANGELOG: Unreleased now has Feat (5), Fix (about 23 incl. the 6 security lines), Perf (2), Refactor (1) lines with finding ids or shas, drawn from `git log v0.9.0-rc.3..6b01c976` (curated: scopes xp, tiling, mc, construct, core, kit; no harness lines). No version heading. It is a curated pick, not all 900 fix commits; the rest are harness and test work.
- known-issues: 7 "verify" groups (xp, ti-8..13, ti-18, mc-9..12, co-8..11, co-14, sh-13/17) all kept as "unverified since 2026-11-22" (the ledgers in findings/ hold no newer check). ti-20 removed (verified 5/5, verify-maj r3, gate cell launcher-click). Added mc-47 (fuzzel < 1.11 ignores row clicks) and the mesh-churn MEDIUM (two views of one scale). rd-1 to rd-4 are closed (debt-down r2), rd-5 is in the mc-focus lane, rd-6 is a NIT: none added. The id mch-1 cannot sit in the doc (typos hook reads it as "mch"). Diff: 0 verify lines left unresolved; +2 entries, -1 section (VERIFY, ti-20).
- mc keys: docs/configuration.md mc paragraph now lists Super+Right/Left, Super+o, Super+F12, plain Right/Left/Home/Escape on a page, XF86Back, XF86HomePage, Super+BTN_LEFT, and the R4 changes since rc.3 (Super+a chat was agenda, Super+Return terminal was drawer, Super+Home agenda). Checked against plugins/chora-mc/mc.yaml bindings.
- decisions: D95 "Status: accepted ... amends D77; not built yet (PK4 after the tag, dependency gate to build)"; D116 "Status: accepted and built (kit7-a2, keys_for_new)".
- Gate: tests/merge-gate.sh --jobs 2 full, 57 cells PASS, EXIT 0, no red.
