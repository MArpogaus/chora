# docs-f8 r1 (F8)

Branch claude/docs-f8 on 27bd60c7, head ba9dd020 (pushed). Docs only.
- docs/decisions.md: new section "Decisions of the shared core", D96-D105 (FakeCore, acts and invariants, kit::actions, kit::wm, kit::ask, paths, Language, size ratchet and cut gate, D87 revised, D92 kept). Accepted only where merged (merges named per row); D87 revised is "proposed" (no kit::launch on the branch); D92 gate against hard-coded strings is not built, said so.
- CONTRIBUTING.md: "Where shared code lives", the rule "a look uses kit, no copy", how to add a copies.toml row. docs/project-structure.md: kit and scene lines.
- Open: no separate dev-rules doc exists in the repo; CONTRIBUTING.md and project-structure.md carry it. Decisions 12-14 of the plan (D89, D93, D95 amended) not filed: brief did not list them.
- Gate: pre-commit lychee and vale pass; tests/merge-gate.sh --jobs 3 on ba9dd020: all steps passed (run /tmp/merge-gate.XsPA7i, lane container).

## PM review 04:16 (independent; builder was the lane)
Diff 27bd60c7..ba9dd020: CONTRIBUTING.md +18, docs/decisions.md +17 (D96-D105), project-structure.md 2 lines.
Checked: every cited merge SHA is the named merge on the session branch (f31ff880 net1, de057ab3 net2-adapters,
8b6b8d25 net2-harness, bdf84f41 kit2, 2afec150 wm1, 5990b001 kit4, 27bd60c7 kit1); D104 (D87 revised) marked
proposed, correct since kit::launch is not built; D105 says the string gate is not built. No dev-rules doc
exists, CONTRIBUTING carries the rule. Open (F8 not met until): D89, D93, D95-amended records of the 11-17 plan.
Verdict MERGE-OK (0 BLOCK, 0 MAJOR). Train 55 started 04:16.
