# decisions r4 (rc 8): fix the review findings of r2

Model: sonnet, effort low (wording fixes in one file)
Lane: cloud, branch claude/decisions, continue from its head (5ed35a5). One round, at most 1 hour.
Read reports/review-decisions-r2.md on claude/session-log (verdict FIX). Fix in docs/decisions.md:
1. MAJOR D78: only construct, mc, tiling and xp read the setting test_accept_injected_keys
   (`.accept_injected(...)`); notify and the assistant follow only CHORA_TEST_ACCEPT_INJECTED_KEYS,
   and the notify tests pin it off. Check the code and state it exactly.
2. MINOR D78: name node.key among the flagged events (event.rs:179, kit from_person) or drop "every".
3. MINOR D78: name the gate: tests/gate-injected-presses.sh.
4. COSMETIC: say "proposed (owner, 2026-11-17)" once in the intro, not in each D87-D94 row; write the
   audit's question ids as "debt-rc 3b D3" so they cannot be read as decision D3.
Nothing else. Docs only. Done when pre-commit and mkdocs --strict pass. Report: lines appended to
reports/decisions-r1.md (each finding: fixed, with the line), one send_message to the PM.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines;
never --no-verify; stage by name; no force push; no pull request. Push only claude/decisions.
