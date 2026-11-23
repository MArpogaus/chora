# tiling-round

## r1

Lane: claude/tiling-round from 011db749. Head bb9fa6b3. Step 0 green.

| Item | Result |
|---|---|
| I2 | Cause: look.gap_inner 10 in the default look (tiling.yaml:22, config.rs Default). Red d85828ff (config unit test, I2: left (12, 10), right (12, 12); gate-tiling-gaps expects 12). Fix 573496e5: gap_inner 12 (tiling.yaml, config.rs, docs, two more unit tests). Harness test_acts ti-31 hard-coded 10: bb9fa6b3. gate-tiling-gaps 4/4 ok, tiling unit tests 182 ok, test_acts 178 ok. |
| TF-1 | Red 90b13505: "FAIL: without Inter the clock selftest did not print a skip line naming Inter (TF-1)". Fix badaecd2: skip lines name python3, PIL or Inter. Selftest ok. |
| T3 | Already holds, no change. Fresh tiling session: Super+a then `type hello`, no click: "hello" is in the chat input (capture). Cell: tests/gate-chat-keys.sh "ok: with one helper, typed text and Enter reach the helper" (6 ok lines, run on the head). The r4 red was the persona harness (helper never worked, persona never typed). |
| T1 | Not fixed, needs a decision. Plain first Super+Return in a fresh tiling+xp session: tiling-owned (s1 tiling), not reproduced. It is xp-owned when the pointer or keyboard focus sits outside the claim of tiling: xp start menu open and pointer on the start button, or pointer on the tiling bar (rows from my run: pointer 1275,5 and 640,12 gave xp windows; 5,5, 640,400, 5,400 gave tiling). Cause is the product rule of choose_manager_of (crates/chora/src/state.rs:991, D55): a key chooses its manager only when its claim holds the pointer or focus; the tests a_grabbed_key_of_the_user_chooses_its_manager_where_the_person_is (ipc.rs:2876) pin "the pointer is in b". The brief's rule ("the key's window belongs to the key's look") contradicts that pinned design, so I made no core change. r4 pointer position is not in its logs. |

Gate (full, --jobs 2, detached, merged state equal to head, autonomous branch already contained): 25 PASS, 1 FAIL: acts (test_acts ti-31, mine: AssertionError 12 != 10), fixed in bb9fa6b3 and checked locally (178 ok); the gate ended EXIT 1 after that cell and was not rerun. Cells after it did not report; tiling gap gate run alone, ok. Persona contracts T1/T3 for kemal and ricer not run (budget); T3 evidence above.
