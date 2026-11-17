# Plan after the foundation week (draft 2026-11-17 12:5x, PM; critic below)

Owner 11-17 11:54: "Sharpen this plan further in reflection round. Have unnecessary detailed and
expensive desktop effects land before 1.0 so we can have som cachy sisuals to get attention."
Earlier the same day: foundation first, then UX/UI polish towards perfection, then the collected
features (no new demo sprints), to get "a usable and hackable desktop environment"; the demo is a
real-world workflow in one take (design/demo-playbook.md).

## Phases (each ends at a merge boundary with a tour against the last baseline)
| Phase | Weeks (budget) | Done when (measured, independently checked) |
|---|---|---|
| P0 rc tag | 11-17 | design/rc.md with the 10:47 waiver; tag pushed |
| P1 foundation | to Sat 11-21 03:59 (~40 %) | design/foundation.md F1-F8 met; tour v9 = v8 |
| P2 UX polish | week 2 (~35 %) | per look: no BLOCK, MAJOR or MINOR open in findings/; fresh-eyes tour v10 finds none; v7/v8 MINOR rows closed |
| P3 collected features | weeks 3-4 | each item below merged with its gate cell and a tour beat |
| P4 effects (S3) | week 4, before 1.0 | one plugin on a scene effect API; each effect inside its frame budget (gate) and off by default outside the demo |
| P5 one-take demo | after P3, P4 | demo-playbook.md runs as one e2e test, 9 beats green 3 times in a row; recorded |
| 1.0 | after P5 | rc.md in full (no waiver), persona round 4, CI and packaging green |

## P3 order (ranked by what it unblocks; cost from backlog.md estimates)
1. N1 widget host and N2 settings service in chora-kit: every later item sits on them (N2 12-16 %).
2. DB12 one cube service: construct owns the cube; xp cube is a view of it.
3. X10 XP tray with working popups and chevron, X11 taskbar (Quick Launch, lock, resize, reorder), X12 Start menu pinned and frequent: on N1/N2.
4. A14 cylinder and slider geometries (demo beats 7, 8).
5. Voice tools and the mail/calendar sources for mc (demo beat 1).
Items not on this list stay in backlog until 1.0.

## Rules that keep it honest
- One phase at a time; a phase opens only when the previous one's criteria are met (no partial credit, owner 11-17 10:59).
- Every slice: red test first, merge gate, independent review; budget at most 12.5 % a UTC day.
- Effects never block P2: S3 lands only after P2 is met, and it ships off by default.
- The demo is a test, not a sprint: beats only use features already merged.

## Open questions for the critic
- Is P2 before P3 right when X10-X12 rewrite the xp panel anyway (polish lost)?
- Is "no MINOR open" a reachable P2 bar in one week?
- Budget: P3 alone looks larger than two weeks at 12.5 % a day.
