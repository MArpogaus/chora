# mc-helper

## r1

Lane claude/mc-helper from ac538e82, head 42585079 (pushed). Already
contains origin/claude/autonomous-dev-session-3w-i33pg7 (merge: up to date).

### mc-57 (cause, red, green)

Cause: not a product fault and not an injected-key fault. The fake voice
agent answers `agent_list` with one helper and the one project it runs
("projects: demo; agents: atlas (demo, working)"). parse_list gives one pick;
`only_helper` (plugins/chora-assistant/src/chat.rs:795) opens that chat, as
docs/hud.md:370 says ("the only helper that runs, not a list of one"). So
Super+a opens the chat and `hud-chat-pick-1` never exists. The fixture is
wrong against docs/hud.md, not the product; docs/api.md has no rule on it.
- red 9eaee852: `known-red-mc-host` is the plain step `mc-host`;
  gate-mc-host.sh: "FAIL: light: the chat shows no list of helpers".
- fix 00406cd8: tests/fake-voice.py line of `agent_list` answers
  `FAKE_VOICE_LIST` when set; gate-mc-host.sh sets a second project.
  Two checks that the red had hidden were stale too and follow the product:
  the hint is part of the card (cf78bef2), so the "card covers the hint"
  check (null node `hud-chat-hint`) is now "the card names its keys, no
  second hint node, card inside the box also at two lines"; the dark dot
  is 216 152 138 (hud.yaml:27, state-dark.svg), not 204 120 102.
- green 5/5: 5 runs of tests/gate-mc-host.sh, each exit 0, 30 ok, 0 FAIL.

### mc-55 (I10 "AGENTS")

Cause: plugins/chora-mc/scenes/agents.yaml `chapter: {title: Agents}` (drawn
as 'AGENTS', '3  AGENTS', '→ 3  Agents') and the running head
"Agents  ·  N running" in agent.rs.
- red 5ed59654: unit tests scene::no_shipped_chapter_title_says_agent
  ("agents: agents") and the_host_draws_the_voice_status (head says
  "Agents  ·  1 running"); live: acts `mc I10 red`, three lines, e.g.
  "mc-s-agents-chapter-title says 'AGENTS', a word of the builders".
- fix c5acd4d0: title and head say Helpers; manage.rs tests and
  gate-mc-paths.sh:374 ("→ 3  Helpers") follow; the 3 mc I10 rows removed
  from known-red.toml. The check keys on text, not a region index (the
  unit test and I10 read texts).
- green: unit tests, acts `cell mc I10 pass`, 5/5 acts runs.

### mc-56 (doors under the chat)

Cause: `door_items` (region/items.rs) kept the five doors and their
`door:*` hit areas in the scene while the chat was open, under the host's
chat image (I6 1.00:1).
- red 61098a37: unit test the_doors_of_the_column_go_while_the_chat_is_open
  ("no door under the chat"); live: acts `mc I6 red` with
  mc-r0-right-doors-programs/lock/power 1.00:1 and volume 1.69:1.
- fix 42585079: `door_items` returns nothing while `chat == Open`, as
  `key_items` does; the items, with their hits, are back at the close
  (POLS). The 4 mc-r0-right-doors-* rows removed from known-red.toml.
  Not touched, other findings: I6 rows mc-r1-right-agents-0-sign,
  mc-r1-right-chat-title, mc-r1-right-chat-door stay known red.
- green: unit test 5/5; acts mc cell: 42 pass, 1 known red (I6, those 3
  rows), 0 blocking, 5/5 runs; the same 5 runs show `cell mc I10 pass`.

### Counts

gate-mc-host 5/5; the 3 new unit tests 5/5; `tests/acts/run.py --no-selftest
mc` 5/5 (exit 0, "42 pass, 1 known red, 0 blocking" each).
Captures: none kept. The acts harness removes its evidence directory on a
green run (run.py, "evidence kept" only on a block); the red evidence dir
of the first run was /tmp/acts-run-6tte_cfr (container, gone with it).

### Gate

`tests/merge-gate.sh --jobs 2`, full, detached, on 42585079 (= merged with
origin/claude/autonomous-dev-session-3w-i33pg7): 58 steps PASS, "all steps
passed", EXIT 0. `mc-host` is now one of them. No foreign red.
