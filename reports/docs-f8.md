# docs-f8 r1 (F8)

Branch claude/docs-f8 on 27bd60c7, head ba9dd020 (pushed). Docs only.
- docs/decisions.md: new section "Decisions of the shared core", D96-D105 (FakeCore, acts and invariants, kit::actions, kit::wm, kit::ask, paths, Language, size ratchet and cut gate, D87 revised, D92 kept). Accepted only where merged (merges named per row); D87 revised is "proposed" (no kit::launch on the branch); D92 gate against hard-coded strings is not built, said so.
- CONTRIBUTING.md: "Where shared code lives", the rule "a look uses kit, no copy", how to add a copies.toml row. docs/project-structure.md: kit and scene lines.
- Open: no separate dev-rules doc exists in the repo; CONTRIBUTING.md and project-structure.md carry it. Decisions 12-14 of the plan (D89, D93, D95 amended) not filed: brief did not list them.
- Gate: pre-commit lychee and vale pass; tests/merge-gate.sh --jobs 3 on ba9dd020: all steps passed (run /tmp/merge-gate.XsPA7i, lane container).
