# PM review looks1-mc r1 (10:55 UTC, 7a6c0eb8..b0453e19, +42 -101, 3 commits)

Small diff, PM review (procedures: a PM review for small diffs; the PM is not the builder).
- Checks: region/items.rs chat_items is empty while a question is pending and the chat is closed (question_pending: voiced word, else the first helper's state); door_items empty while the chat is open; unit test the_doors_leave_while_the_chat_is_open; mc.yaml idle/asleep faint to muted; "Agents" to "Helpers" in scenes, agent.rs head, docs/configuration.md, manage.rs tests, gate-mc-paths.sh. Known-red rows 189 to 179 (I6 x7, I10 x3 removed, each now green in the lane gate).
- lm-1 MINOR: no unit test for question_pending (act rows cover it). Backlog.
- lm-2 NOTE: the lane gate's acts step was red once on foreign construct cells (I6 PANES 11 px vs 14 px; "the hint keys names no key", acts.log:475-477); run 1 green; suspected flake, the train proves it.
- docs/architecture.md "Agents: MCP clients" stays (technical term).
- Verdict (computed): 0 BLOCK, 0 MAJOR: MERGE-OK; b0453e19 joins t109.
