# security a r2 (rc 6): fix round of review-security-a-r1 (FIX)

Model: sonnet, effort high (security fixes, one touches a shared scene event type)
Lane: cloud, branch claude/security-a (443a1570); merge the session branch
claude/autonomous-dev-session-3w-i33pg7 first. One round. Stop at 250 k context and hand
over. Read dev-rules.md and conventions.md (with the skills it names) first, then
reports/review-security-a-r1.md (all of it) and reports/security-a-r1.md. Setup as
briefs/flakes-r2.md.

Items (numbers of the review), red first each:
1. MAJOR 1: injected keys bypass CON-B-10 in the construct window list. Compile-time gate:
   add `injected` to NodeKey (crates/chora-scene/src/event.rs:150), set it in
   crates/chora/src/input.rs:259, destructure it in every handler so each decides; the
   window list ignores injected keys unless test_accept_injected_keys is on. Red unit test
   of list_input with an injected Return first.
2. MAJOR 2: panel.set (chora-mc main.rs:646, 3115) rewrites the agents and overview panels.
   Refuse those scenes in set_panel (KISS); red unit test first.
3. MINOR 3, 4, 5: notify.clear privileged (pinned privileged-methods test); click passes
   injected to show and focus (test like an_injected_press_on_a_hit_does_nothing); pin
   inject_ok in the test harness so the env var does not change unit tests.
4. MINOR 6: one e2e cell in a session with the switch off: an injected press on an mc hit
   and on the window list does nothing.
5. COSMETIC 7, 8: the set_text doc line; docs/configuration.md 925 and "Injected keys".
Done when: each item has its red commit and fix; tests/merge-gate.sh and tests/ci-local.sh
pass once (say skips). Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines)
and as reports/security-a-r2.md on claude/session-log with the earliest gate per item.
Rules as briefs/flakes-r2.md, with push only claude/security-a and that report; never end a
turn to wait for a long background job.
