# mc-tidy

## r1

Branch claude/mc-tidy (from 5b34d4b1), head a3a9e8ef. No product code changed: three of the four rows were already fixed on the session head, one is a host transient.

| row | red sha | fix sha | cell | state |
|---|---|---|---|---|
| mc-55 ('AGENTS') | 5ed59654 (unit `no_shipped_chapter_title_says_agent`) | 31f53dc9 / c5acd4d0 (agents.yaml, agent.rs, manage.rs), already on 5b34d4b1 | unit + I10 | fixed on the session head; row stale, close it. mc-tidy changes nothing |
| mc-57 (gate-mc-host.sh:185) | 9eaee852 | 00406cd8 (fixture lists a second project) | gate-mc-host.sh light + dark | green here (all checks ok in both themes); no known-red-mc-host entry remains (mc-host is a plain step); close the row |
| mc-56 (doors under the chat) | 5a59b7ae (sofa cell `door`: no door text overlaps hud-chat) | none needed: MH-2 (bf23d49d, 97efc46a) keeps the doors visible and the chat box ends one baseline above them | sofa `door` (a3a9e8ef waits for the box), unit `the_chat_ends_above_the_doors_on_a_small_output` (1280x720) | settled layout is right: chat [888,196,336,420], Programs at y 644 (no overlap). The red was the first frame only: hud-chat is the host's full-area box [880,0,400,720] for about 0.2 s until mc.chat.slot arrives. That transient stays open (host should hold the draw until the box arrives, with a timeout when mc is absent; not done: touches the assistant host) |
| mc-50 | none | none | sofa `remote-back` green (one XF86Back turns it, 45fcc3f1) | back half closed. Yellow square after Return: no mc node, pointer marker of the headless capture; cannot be reproduced headless, stays open |

Captures: none taken (SHOTS unset); cells and logs in /tmp/merge-gate.OYLH6z/.

Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` on a3a9e8ef: 38 PASS, 1 FAIL (`keys`: gate-construct-keys-persona.sh:88 "the Foot row of Programs started no foot", reproduced with `--only keys`), xp-fields SKIP. The diff is a test script and a unit test in mc, so keys cannot be caused by it; not bisected on 5b34d4b1. Not green: the PM decides on the keys row.
