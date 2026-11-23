# docs-rc

## r1

Head 54c30d16 on claude/docs-rc (one commit on 6b01c976, which is already origin/claude/autonomous-dev-session-3w-i33pg7: the merge was a no-op). Tools: vale and lychee present; pre-commit passed (vale, lychee, typos).

- CHANGELOG: Unreleased now has Feat (5), Fix (about 23 incl. the 6 security lines), Perf (2), Refactor (1) lines with finding ids or shas, drawn from `git log v0.9.0-rc.3..6b01c976` (curated: scopes xp, tiling, mc, construct, core, kit; no harness lines). No version heading. It is a curated pick, not all 900 fix commits; the rest are harness and test work.
- known-issues: 7 "verify" groups (xp, ti-8..13, ti-18, mc-9..12, co-8..11, co-14, sh-13/17) all kept as "unverified since 2026-11-22" (the ledgers in findings/ hold no newer check). ti-20 removed (verified 5/5, verify-maj r3, gate cell launcher-click). Added mc-47 (fuzzel < 1.11 ignores row clicks) and the mesh-churn MEDIUM (two views of one scale). rd-1 to rd-4 are closed (debt-down r2), rd-5 is in the mc-focus lane, rd-6 is a NIT: none added. The id mch-1 cannot sit in the doc (typos hook reads it as "mch"). Diff: 0 verify lines left unresolved; +2 entries, -1 section (VERIFY, ti-20).
- mc keys: docs/configuration.md mc paragraph now lists Super+Right/Left, Super+o, Super+F12, plain Right/Left/Home/Escape on a page, XF86Back, XF86HomePage, Super+BTN_LEFT, and the R4 changes since rc.3 (Super+a chat was agenda, Super+Return terminal was drawer, Super+Home agenda). Checked against plugins/chora-mc/mc.yaml bindings.
- decisions: D95 "Status: accepted ... amends D77; not built yet (PK4 after the tag, dependency gate to build)"; D116 "Status: accepted and built (kit7-a2, keys_for_new)".
- Gate: tests/merge-gate.sh --jobs 2 full, 57 cells PASS, EXIT 0, no red.

## PM review r1

PM review (docs only, 4 files, +48 -15, under 200 lines; the PM is not the builder). claude/docs-rc 54c30d16 against 6b01c976; the lane's full gate --jobs 2 EXIT 0.
- pr-1, MINOR, docs/known-issues.md (9 rows): "unverified since 2026-11-22 (the ledger holds no newer check)" reads as if checked on 11-22; the date is the review date, not a check date, and the parentheses nest. Fix: "unverified (no check in the ledger since rc 3)" or the date of the last check per row.
- pr-2, MINOR, docs/known-issues.md shared: the mesh-churn line is MEDIUM, a severity the file does not use, and sits after COSMETIC. Fix: MINOR, under MINOR.
- pr-3, MINOR, docs/decisions.md D116: "merged before rc 8" names no tag that exists. Fix: "merged in train 70".
- pr-4, NIT, D95: the Status line repeats "accepted (owner, 2026-11-19 05:59), amends D77" from the next sentence. Fix: "Status: accepted; not built yet (PK4 after the tag; dependency gate to build)."
- pr-5, NIT, CHANGELOG: "a 1 px line hairline"; docs/configuration.md: the new sentences are one 700-char line where the paragraph wraps at 78.
Verdict: 0 BLOCK, 0 MAJOR: MERGE-OK; joins the next train. pr-1..5 go to docs-rc r2 (on the Thu head).

## r2

Head 37730187 on claude/docs-rc-r2 (7 commits on bc153929). vale and lychee present; cargo check passed; pre-commit passed on every commit.

- pr-1 4d4f1c3f: "unverified: no check in the ledger since rc 3" (8 rows, no nesting; the intro says since rc 3).
- pr-2 8006a31a: the mesh-churn row is MINOR, under MINOR in the shared section.
- pr-3 a7dfa755: D116 "merged in train 70".
- pr-4 5e5a427f: D95 "Status: accepted; not built yet (PK4 after the tag; dependency gate to build)."
- pr-5 13535fb6: CHANGELOG "a 1 px hairline"; the mc key sentences in docs/configuration.md wrap at 78.
- CHANGELOG d52eb615 is the known-issues commit; 37730187 adds 16 lines for merges since 6b01c976 (xp 5, tiling 3, mc 4, construct 1, kit 1, core 3 Fix lines; 2 Perf lines), each with a finding id or sha; no harness lines.
- known-issues d52eb615: 28 rows removed, 1 reworded (ti-11: foreign programs only); empty headings dropped. Closed in findings/*.md with a sha that is an ancestor of bc153929: xp-23 65f4510b, xp-24 9a6506d6, xp-25 21799660, xp-26 5ac45053, xp-27 cf78bef2, ti-5 03e06230, ti-6 and ti-19 c1adc900, ti-7 6092aee3, ti-21 bbb105ba+3a9a426f, ti-22 c9ed1cfd, ti-23 c9ed1cfd, ti-24 c9ed1cfd, mc-13 and mc-14 6fa86c16, mc-32 cbdbc34f, mc-33 dda31e6a, mc-34 0fbb751e, mc-35 3c19f217, mc-36 1eb04373, mc-37 309ae998, co-16 and co-18 cbdbc34f, co-19 and co-20 929b64e7, co-21 db4d703b, sh-23 and sd-h1 (not red), sh-24 cc940fec, sd-h2 20e4ad6e. Kept: ti-1 and mc-47 (fixed or closed in the ledger, but a user with fuzzel < 1.11 still hits them; the PKGBUILD pin 5dd34fcd covers Arch only), the Notepad note, the open and verify rows, and mesh-churn (it has no row in findings/; reports/review-mesh-churn.md says it stays).
- Note: the brief names reports/findings.md; the ledgers are findings/*.md on claude/session-log.
- Gate: tests/merge-gate.sh --jobs 2 (full) on the branch merged with fresh origin/claude/autonomous-dev-session-3w-i33pg7 (bc153929, already contained: no-op merge), EXIT 0, every cell PASS, no red.

## PM review r2
- 11-23 14:58 PM (seeded rule: docs only, 39+/55-, PM not the builder): MERGE-OK 37730187, 0 BLOCK, 0 MAJOR.
- pr-1..pr-5: one commit each with its id (13535fb6, a7dfa755, 5e5a427f, 4d4f1c3f, 8006a31a); each matches its Fix line.
- known-issues: 15 removed ids sampled against findings/*.md (xp-26, xp-27, ti-24, ti-5, ti-7, mc-32, mc-34..37, mc-13, co-21, co-16, sd-h2, sh-23): each fixed or closed there with a cell; none removed while open.
- CHANGELOG: 16 Fix + 2 Perf lines name shas or ids; no harness lines; no version heading.
- Lane gate EXIT 0 full on the bc153929 merge. Joins the next train (train118).
