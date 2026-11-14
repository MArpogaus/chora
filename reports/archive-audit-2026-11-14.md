# Archive audit, 2026-11-14 (owner request 17:2x UTC)

Three read-only opus audits: the prune commits since 11-12 (77309d5d and smaller), archive/
against the active set, answers.md against the active set. About 120 archive items: about 70
covered, 25 superseded, 15 missing, 7 conflicts. The PM decided each point (owner 17:17).

## Restored
| Item | Source | Now in |
|---|---|---|
| Resume at every limit reset; pre-stop routine | charter 996, routines 14/54, owner 11-14 10:03 | routines.md |
| Status page after every merge, with reviewed captures | old rules 9, owner 11-10, 11-12 01:26 | routines.md |
| Week report every Friday, critical, no overselling | owner 11-09, charter 11b | routines.md |
| Daily independent opus critic, answered the same day | owner 11-09, charter 14 | routines.md, self-check trigger |
| Split review (5,000 lines) and extraction check, weekly | charter 17, 18; owner 11-09 | routines.md; rc.md 6 |
| Security fix 11 (a)-(c) | charter 11 | rc.md 6, backlog A2 |
| Persona round 5 scope | rc.md before 77309d5d | rc.md "After the tag" |
| Visual acceptance screencast | rc.md before 77309d5d, owner 17:08 | rc.md 11 |
| 6-min start check, PENDING follow-up, archive at hand-back | routines, friction 41, owner 11-11 11:43 | routines.md |
| Persona triage, rule broken twice gets a guard | guards 36-37, routines 52 | routines.md |
| Session end 11-30 with HANDOFF.md | charter 861-869 | routines.md |
| 85 % cap for haiku swarms | owner 11-11 17:29 | conventions.md 10 |
| Security policy and licensing stay the owner's; outward text a draft | charter 722, 61 | conventions.md 7 |
| xp helper balloon behaviour | owner 11-11 18:26 | product-briefs.md xp |
| Bindings overlay as a shared plugin; users ship own HUDs | owner 11-11 19:45, 11-09 | backlog H5, H6 |
| Marketing framing; release questions Q4, Q5, Q12, Q13 | owner 11-10, questions.md | backlog M1 |

## Decided (conflicts)
1. "Clean" (charter: no MEDIUM or HIGH) vs rc.md: the rc needs no BLOCK or MAJOR, every HIGH and
   security MEDIUM fixed; other MEDIUM to the known issues and worked before 1.0. An rc is not 1.0.
2. "Polished to perfection" vs two pixel rounds a week: the rc bar is no BLOCK or MAJOR; polishing
   goes on after the tag until 1.0 (vision-card.md). Each round is a full review, not confirm-only.
3. Demos as features land vs rc only: the demos come from the criterion 11 recording.
4. Daily critic: restored (cheap read-only opus pass), the self-check alone was too weak.
5. Architecture A1-A22: the PM decides (17:17); every change of docs/decisions.md goes into the week
   report for a veto.
6. Time zone: logs stay UTC (tools); chat and status page also give Bangkok time.
7. Spec pointer in dev-rules.md: design/looks/ and product-briefs.md.
8. mc name stays until after 1.0; the 8 spec-gap defaults of product-briefs.md stand.
9. Q2 (agent.set and notify.push privileged): keep, the default.
10. Fixed: decisions-now "After ui ux clean" time 10:30 -> 10:10 UTC; product-briefs stale pointers.

## Kept cut (superseded)
Full pixel review until COSMETIC before the rc; assets in all tests; per-face managers (D4); XP
split first; rc 11-18; owner tags; week-1 budget orders, three trains a day, the 4 h 30 run cap;
English chat; deleting stale branches.
