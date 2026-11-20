# p2-r2-xp (RC 2 persona round 4, P2 item 1): xp persona round 2

## r2
Setup: branch claude/p2-r2-xp at 79c8d2b5; `cargo check` and `cargo build --workspace` green; round.sh snapshot once, binaries of 79c8d2b5 in /tmp/persona-round/bin; sessions xp 1280x800, PERSONA_CHECK=1, 2 slots (Erika and Murat together, the pixel-watcher after); helper transcript seeded (a user and an agent entry) after each start; for T5 a chora-probe window (40 120 200 --report) started by hand with the display of the headless output. Haiku personas, act.py only, caps 45 actions and 20 looks (used: Erika 32/15, Murat 37/16, pixel 33/16). No code changed. Driver slip: my first probe start survived beside the second (two probe windows in the taskbar of the pixel run); T5 data is from the second.
Caveat as in r1: "expected open [], got [xp-dlg-help]" comes from the persona leaving Help open, not a defect. It is the only reason for the red of T2 (Murat, pixel), X2, T4:lock and T4:volume. Harness changes since r1 are in: mousepad stand-in, atlas answers, lock/screenshot scripts, persona-probe.

### Erika (check.py end T1 X1 T4:clock T4:power T3)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| T1 Notepad from Start | pass | 0 | partial: window opens, typed note never shows (stand-in window?) |
| X1 minimize and find again | pass | 0 | yes (taskbar button brought it back) |
| T4 clock | FAIL: no clock node (checker id, as r1) | 0 | yes (2:37 AM) |
| T4 power cancel | pass | 0 | yes |
| T3 helper | pass | 0 | yes ("Hello") |

### Murat (T1 T2 T4:volume T4:screenshot T6)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| T1 | FAIL: open left by Help; window s1 not focused after Help | 4 x I5 (balloon over the 25% row of the volume menu) | yes |
| T2 Help and Support | FAIL: open left by Help only | see T1 row | yes (table with Super+Print, Vol keys) |
| T4 volume | FAIL: open left by Help only; set-volume 75% x3 in calls.log | see T1 row | click reaches the stand-in; persona saw no tick change (stand-in answers 0.50) |
| T4 screenshot | FAIL: no grim call | see T1 row | not tried |
| T6 honest failure | pass | see T1 row | yes ("Nothing found"; persona wanted an error box) |

### pixel-watcher (X2 T2 T5 T4:lock; --probe)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| X2 Start menu, drag | FAIL: open left by Help; the dragged shape was the chora-probe window without a title bar (harness) | 0 | no |
| T2 key list | FAIL: open left by Help only | 0 | yes via Help; F1 shows nothing, Super+F1 not tried |
| T5 no stray keys | pass | 0 | yes (Super+D, Super+E, Super+Shift+Esc; probe got no Super chord key) |
| T4 lock | FAIL: open left by Help only; swaylock -f x3 in calls.log | 0 | lock reached the stand-in; nothing drawn, persona says no |

### Fix check (xp findings fixed since r1)
| Finding | act or cell | Reached | Result | Capture |
|---|---|---|---|---|
| xp-23 paperclip | cell gate-xp-paperclip | Erika clicked it | pass (chat opens, answer Hello) | erika shots 013-015 |
| xp-31 Super+D toggle, task button restores | acts second Super+d, task button | pixel Super+D twice; Erika minimize + taskbar button | pass | pixel shots 010-011, erika 008-009 |
| xp-32 speaker click opens volume popup | act click on speaker | Murat | pass (popup, set-volume 75% called) | murat shots, calls.log |
| xp-26 tooltip and tick of the level | act tooltip, popup ticks | Murat clicked 75%, no hover | not reached (tick cannot move: the wpctl stand-in always answers 0.50) | calls.log |
| xp-27 question card | cell gate-xp-question | no persona met a question | not reached | none |
| xp-33 keys key opens the key list | act keys key | pixel pressed F1, not Super+F1 | not reached | pixel shot 007 |

### triage.py (the p2r2 lines of its output; whole run: waiting 86, overdue 27, missing ux 0, FORMAT OK)
```
== xp / I5 (1)
  2026-11-20-p2r2-murat.md: Starting Notepad balloon covers the 25% row of the volume menu (0 days)
== xp / contract T4:clock (2)
  2026-11-20-p2r2-erika.md: Clock is read at a small size (contract red is the checker, not the look) (0 days)
== xp / contract T4:lock (2)
  2026-11-20-p2r2-pixel.md: Lock gives no visible sign (0 days)
== xp / contract T4:volume (2)
  2026-11-20-p2r2-murat.md: Volume popup: level does not tick after a click (0 days)
== xp / contract T6 (2)
  2026-11-20-p2r2-murat.md: Run box: "Nothing found" for an unknown program, once a silent close (0 days)
== xp / contract X2 (2)
  2026-11-20-p2r2-pixel.md: The small blue window does not follow a drag (0 days)
== xp / none (17)
  2026-11-20-p2r2-erika.md: Typed note does not appear in Notepad (0 days)
  2026-11-20-p2r2-erika.md: "Starting Notepad / Please wait" balloon stays up after Notepad has opened (0 days)
  2026-11-20-p2r2-murat.md: Starting Notepad balloon stays after Help opens (0 days)
  2026-11-20-p2r2-pixel.md: Help window stays above the desktop after Super+D (0 days)
  2026-11-20-p2r2-pixel.md: F1 on the desktop does nothing; Super+F1 not tried (0 days)
waiting 86, overdue 27, missing ux 0
OVERDUE
```

Files: feedback/2026-11-20-p2r2-{erika,murat,pixel}.md; captures and logs media/2026-11-20-r2/{erika,murat,pixel}/ (shots, actions.jsonl, calls.log, evidence). No ledger edits.
Round binaries: sha 79c8d2b5 (/tmp/persona-round/bin, one snapshot).
Stop rule (p2-plan 3): xp does NOT meet it. One new check-backed finding: I5 on the volume path (balloon over the 25% row, 4 lines in Murat's breaches.jsonl), and the fix check leaves xp-26, xp-27, xp-33 not reached. Contracts T2/T4:volume/T4:lock/X2 are red only through the open Help window; the checker or brief should close Help (or the contract ignore it) before r3. Harness candidates: Notepad stand-in takes no text, probe window drag, volume stand-in constant 0.50, lock draws nothing. Rounds so far on xp: 2 of 3.
