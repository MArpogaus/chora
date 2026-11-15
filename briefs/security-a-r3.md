# security a r3 (rc 6): close the injected-input class in every look

Model: sonnet, effort high (security class fix across four plugins, findings named)
Lane: cloud, branch claude/security-a (0a3d0040); merge the session branch
claude/autonomous-dev-session-3w-i33pg7 first. One round. Stop at 250 k context and hand
over. Read dev-rules.md and conventions.md first, then reports/review-security-a-r2.md (all)
and reports/security-a-r2.md. Setup as briefs/flakes-r2.md.

Two rounds closed single paths and missed others of the same class. This round closes the
class: with test_accept_injected_keys off, an injected key or press changes nothing a person
would see (no focus, no keyboard to a window, no program, no scene turn, no close).
1. Gate first (compile/lint time): a check in the pre-commit set (tools/ or a clippy-style
   grep step, as the harness already does) that refuses `injected: _` and `..` in every
   pattern on NodeKey, PointerPressed, PointerReleased (and any event with `injected`) unless
   the line carries a `// injected: <reason>` comment. Show it red on today's tree.
2. Review r2 MAJOR 1: construct walk arm (input.rs:241-260) and the cube keys of key_act
   (modes.rs:596-603): pure `walk_key(event, accepts)` like list_key, red unit test first.
   Docs: configuration.md 192-199 lists what the switch covers.
3. MINOR 4: mc page_key (main.rs:882-891, 1216). MINOR 5: chora-xp wm.rs:789-796, 1798 and
   chora-tiling main.rs:693 (each rated MAJOR alone). Red unit test per plugin first.
4. MINOR 6: chora-kit runtime.rs:881 pin_accepts_injected. COSMETIC 7: input.rs:636.
5. MINOR 2, 3: tests/gate-injected-presses.sh opens the list by a method (not an injected
   Tab), no `sleep` as a wait (until_ok, or a later real event as barrier), a switch-on
   control for every negative cell, xp and tiling cells added.
Done when: each item has its red commit and fix; the new check passes on the branch;
tests/merge-gate.sh and tests/ci-local.sh pass once (say skips).
Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 15 lines) and as
reports/security-a-r3.md on claude/session-log with the earliest gate per item.
Rules as briefs/flakes-r2.md, with push only claude/security-a and that report; never end a
turn to wait for a long background job.
