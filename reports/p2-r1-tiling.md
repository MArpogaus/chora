# p2-r1-tiling (RC 2, P2 item 1)

Lane: cloud, branch claude/p2-tiling at b97369d1 (no code changes). Binaries of the round: sha b97369d1cd9a3c9837ceaf756cc115c0dc5c113b, snapshot /tmp/persona-round/bin. Step 0: unshallow, cargo check and build green. Sessions tiling 1280x800, PERSONA_CHECK=1, 2 slots (Kemal and beginner together, the ricer after), helper transcript seeded (user + agent entry). Personas: haiku Agents with act.py only; caps 45 actions, 20 looks. Breaches (breaches.jsonl): 0 in all three runs. Driver slips: none; one deviation: T5 not checked (see harness).

## r1
Harness (not issues): (1) L1 compares toplevels at `check.py start` (none) with the end, so windows opened and closed in the run are never "gone"; (2) T5 needs a probe window, not started (it would add a toplevel to L1); (3) the session has an empty $XDG_CONFIG_HOME, so L2 (change a gap, reload) cannot start; the contract L2 only holds focus, so its pass is vacuous; (4) "expected open [], got [tiling-keys]" is the key overlay the persona left open; (5) T7 tiling -> xp not run (needs `tiling+xp`).

### Kemal (29 actions, 15 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| T1 Super+Space | pass | 0 | yes |
| L1 two terminals, focus, close one | harness red (gone) | 0 | yes (focus on 3rd try via Super+Tab, close via Ctrl+d on 2nd; Super+Right, Super+l, Super+q did not do it) |
| T5 no stray keys | not checked (harness) | 0 | partial: chords tried |
| T3 helper by Super+a | pass | 0 | yes |

### beginner (23 actions, 11 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| T1 start by a visible door | FAIL: clicking Foot in Apps opened the helper chat (3 times) | 0 | no |
| T2 key list | red only for the open overlay (harness 4) | 0 | yes (Keys button) |
| T4 power cancel | red only for the open overlay | 0 | yes |
| T6 unknown program | FAIL: no notice (box says Nothing found) | 0 | yes by eye |

### ricer (18 actions, 8 looks)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| L2 gap and reload | vacuous pass; harness 3 | 0 | no (no settings folder) |
| L3 workspace and back | pass | 0 | yes |
| T2 key list | red only for the open overlay | 0 | yes |
| T7 tiling -> xp | not run | - | - |

Real candidates for the PM: the Apps launcher click opens the first row (helper) instead of the clicked row (tiling-T1, beginner); Super+q does not close a window; Super+Right switches workspace. Files: feedback/2026-11-20-p2r1-{kemal,beginner,ricer}.md; captures media/2026-11-20/persona-tiling-{kemal,beginner,ricer}/. No ledger edits.

### triage.py (tiling section; older swarm6 rows left out)
```
== tiling / contract T1 (1)
  2026-11-20-p2r1-beginner.md: Clicking Foot in the Apps menu opens the helper chat instead of a terminal (0 days)
== tiling / contract T4:power (1)
  2026-11-20-p2r1-beginner.md: Key list stays open after the power menu is cancelled (0 days)
== tiling / contract T6 (1)
  2026-11-20-p2r1-beginner.md: Unknown program in the Apps box gives "Nothing found", no start and no notice (0 days)
== tiling / none (11)
  2026-11-20-p2r1-kemal.md: Super+Right switches workspace instead of moving focus (0 days)
  2026-11-20-p2r1-kemal.md: Super+q does nothing visible; the terminal closed only with Ctrl+d (0 days)
  2026-11-20-p2r1-kemal.md: Launcher lists "Chat with a helper" first and the helper panel covers the right tile (0 days)
  2026-11-20-p2r1-ricer.md: The key list offers a reload key but there is no settings file to edit (0 days)
  2026-11-20-p2r1-ricer.md: Yellow square at the top-left corner after switching workspace (0 days)
```
