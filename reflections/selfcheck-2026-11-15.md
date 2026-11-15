# Self-check 2026-11-15 (workday close, 2026-11-15 12:56 UTC)

Trigger: owner 11-15 09:35 / 09:32 "have a opus reflection and resolve frictions and sharpen our workflow".
Critic: one independent opus agent (effort medium, read-only) on the last 24 h of the log and both branches.

## Self-check questions
- RC burn-down on track? No. 1 of 11 criteria met (rc 7); rc 3 (three greens: F16, F17, F24, F25 open) is the
  highest risk; rc 6 had no Queue line. Action: Queue rewritten (rc 6 per-area sonnet reviews Tue; F25 and F16
  test-side fixes Mon on sonnet low, before the watchdog theory).
- Reviewed branch unmerged > 12 h? No (security a r3 handed back 12:50, review Mon 00:05).
- Running and not in the Queue? Yes before the fix: lanes.py flagged all three lanes without a criterion. Fixed (lanes tagged).

## Main-duty questions
- Pace kept? No. tokens.py: local opus alone 13.4 % this Bangkok day (cloud lanes not counted), over the
  10 %/day share. The PM's own turns are the largest cost (442 M cache-read tokens); 218 log commits in 24 h.
- Every lane on its Model line? Yes, but the lines broke the plan: 4 of 5 build briefs on sonnet high, not medium;
  no haiku lane; five opus reviews after the 09:28 "no new opus" line (my own rule, broken by me).
- Drift from vision or rc.md? Some: ti-1 (only BLOCK) waited 2 h after the owner's yes, and is now a costly bisection
  of a third-party launcher; fallback decision set for Mon.

## Critic points and actions
1. Opus after the stop line (5 reviews). Accepted. No opus until Mon 00:00 UTC; next rule change (merge boundary):
   a same-day re-review of one fix round runs on sonnet medium; opus only first reviews and hard causes.
2. Budget unmeasured 3.5 h. Accepted. Guard built: check_log.py refuses a state.md whose budget line has no
   'tokens.py HH:MM' measure or one older than 120 min (tests in tests/test_check_log.py).
3. Effort split ignored. Accepted. New briefs default to sonnet medium; high needs a reason (state.md models line).
   core watchdog r2 keeps high (no scope message to a running worker, rules.md 5).
4. PM burns opus on log commits. Accepted. Log once per hour plus once per merge (rules.md 9); the watchdog
   stays quiet without change.
5. Lanes without RC tags. Fixed now (lanes.py: ok).
6. rc 6 and rc 3 opt-in gates, rc 8 known issues unqueued. Queue lines 5 and 6.
7. ti-1 slow and costly. fuzzel r2 is one round only; Mon decision: fix, or a tiling Apps list without fuzzel.
8. "duties in every prompt hook" overstated: no UserPromptSubmit hook exists. decisions-now reading corrected;
   asked the owner (Needs owner, default no), since it changes harness settings.
9-11. Status page overstated (F25 cause, 2 of 11 met, stale Needs you). Corrected: 1 of 11, F25 not proven,
   Needs you matches state.md.
12. rc.md state column stale. Open: to drop at the next merge boundary (design/rc.md change needs owner-ok line).
13-15. Repeated corrections (models/budget 11-09, 11-12, 11-14; context/vision 11-12) failed because the guards were
   intentions or optional. This close adds a mechanical guard (2); the spawn guard (critic 18: PreToolUse
   hook on create_session and Agent checking the brief's Model line and the pace) is the next harness item, which
   needs the owner's yes because it changes settings (with the prompt hook question).
16-17. RC reachability: see Queue; the media lane moves before ledger r3 (screencast by Wed).

## Frictions
All open entries resolved today (18fea267): 26-51 each end with a Resolved line (fixed, guarded or closed);
1-25 closed as historical. 11-15 12:04 lanes PENDING 15 min: capacity, replaced sessions started in seconds (41).

## Workflow change
check_log.py budget-freshness guard (point 2): a state.md commit fails without a measured budget less than
2 h old. It removes the cause of "budget not known" (owner 09:27) mechanically.
