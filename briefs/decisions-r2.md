# decisions r2 (rc 8): record the owner's answers to D1-D8

Model: sonnet, effort low (docs only, the decisions are given below)
Lane: cloud, branch claude/decisions, continue from its head (3cc1450f of r1, stopped at 04:39 because
the owner decided the open questions). One round, at most 2 hours.
Read dev-rules.md, conventions.md, briefs/decisions-r1.md and answers.md (line "2026-11-17 04:38 UTC")
on claude/session-log once. Do everything r1 asked that 3cc1450f does not yet do; then turn the
entries for the open questions D1-D8 of reports/debt-rc.md 3b from "open" into decisions dated
2026-11-17, owner, with the chosen option and its consequences:
D1 launch tracking: the plugin owns the child with pidfd and waitid; the window is matched by token or
app id; lives in chora-kit; replaces the 30 s timer and exit-status file; after the tag.
D2 cube panes: face or float stays derived each frame; `placed` is saved in the existing state file.
D3 z-order: groups get an explicit layer/order in the scene API (background, normal, overlay, lock);
the core sorts; replaces the implicit node order and mc re-ordering; after the tag.
D4 environment: setup.sh checks tool versions; the merge gate refuses to start in a drifted container
instead of failing red.
D5 test scripts: each unrun script is run once; it joins the gate or a profile if it passes and covers
something unique, else it is deleted.
D6 localization: chora-kit word tables (words.rs) per plugin, a gate against hard-coded UI strings, a
pseudo-locale test; no Fluent.
D7 config: one schema of shared sections (style, effects, bindings, programs, clock) in chora-kit;
looks add their own sections.
D8 render budget: 4000 x (output pixels / 1920x1080 pixels), floor 4000; documented in docs/api.md
only when implemented (record the decision now, not the doc).
Write only what is given or what the code shows. Docs only. Done when pre-commit and the docs build pass.
Report (reports/decisions-r1.md, extend it, at most 30 lines) with a section "Refactor proposals".
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines;
never --no-verify; stage by name; no force push; no pull request. Push only claude/decisions.
Stop at 250 k context and hand over.
