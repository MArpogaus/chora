# Self-check 2026-11-24 (12:58 UTC, 19:58 Bangkok)

## Answers
- RC burn-down on track? No. F7 (tour v9) is still behind, F5 has no green series (r7 2/3), and rc 1 waits on co-68 (train129 running) and tiling persona r8.
- Reviewed branch unmerged > 12 h? No. co-pull be6e62ec has been MERGE-OK since 12:3x and is in train129. tiling-calm e9387fb2 went back to its lane after t128 went red.
- Running but not in the Queue? No. All 5 lanes are in Queue 1-4. The 6th lane is empty, which is a gap.
- Pace: no. The week is about 28 % used against 47 % time; the last days ran 4.9, 5.2 and 3.6 %.
- Models: every lane runs sonnet. Opus ran only for the critic and audit. The tour-base lanes are routine work and sonnet is right for them. But the look reviews ran on sonnet, and the critic is right that they should run on opus high.
- Vision drift: some. Four tour-base rounds went into tour-diff verdict logic, which is test tooling, while the v9 take itself is stale.

## Critic points and actions
1. Budget under pace. Fill the 6th lane now with TOUR-V9 final take on d40142e4. Run look reviews on opus high. Aim for at least 0.8 %/h.
2. The budget correction is a repeat (11-15, 11-19, 11-20). Treat it as an overdue owner order.
3. Four reds had no friction entry, and t128 had a known cause. Logged. Added a ratchet check before each train (procedures Train 2).
4. Red-first broke again with 689bff45. Logged. Backlog H-RF2. Reviews now check each fix commit's parent.
5. Effort went to test tooling. Tooling is frozen after tour-base r4, which is the last verdict fix. TOUR-V9 take is next.
6. State overstates. "76 greens" now sits next to the series count. The burn-down gets rerun after t129. The severity of xp-38 is being set by the xp-38 r2 lane.
7. Promises from 11-23. Opus reviews were missed; they are now in procedures. The rc.md table rewrite moves to after t129.
8. Compactions and the owner log. decisions-now.md is screened through 11-20 14:39, and the owner has sent no message since. answers.md is current. No action.

## Frictions resolved
- 11-23 13:07: resolved (foreign reds named).
- Three new entries from 11-24: resolved (see friction.md).

## Workflow change
procedures.md Train 2 now runs `tools/{sleep,size,lint}-ratchet` on the merged train worktree before the 2-hour gate. A breach goes back to its lane. This would have caught t128 in seconds.
Seeded review now uses opus high for look, core, design and security reviews.
