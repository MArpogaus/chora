
## r4
Setup: branch claude/p2-r4-xp at fee9c695 (xp-36 3290f1d4 in); `cargo check --workspace` and `cargo build --workspace` green; round.sh snapshot once, binaries of fee9c695 (full fee9c695eea10d53c1c2b3c6b4ddd2ebb6772da9) in /tmp/persona-round/bin; sessions xp 1280x800, PERSONA_CHECK=1, 2 slots (Erika and Murat together, the pixel-watcher after); helper transcript seeded (user and agent entry) after each start. For T5 a `round.sh probe` chora-probe window plus a `persona-probe` window (title bar) for the X2 drag. Haiku personas, act.py only, caps 45 actions / 20 looks (used: Erika 30/16 looks, Murat 35/18, pixel 22/12). Goals in user's words plus "close a window you only opened to look at". No code changed. Lane fuzzel is 1.11.1 here (not 1.9.2): the Run box opens. One extra Erika `look` (001) was the driver's seeding check, inside her 30.
Round binaries: sha fee9c695.

### Erika (check.py end T1 X1 T4:clock T4:power T3)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| T1 Notepad from Start | pass | 0 | yes ("Dear Anna") |
| X1 minimize, find again | pass | 0 | yes (taskbar button) |
| T4 clock | FAIL: no clock node (checker id, 4th run) | 0 | yes (3:27 PM) |
| T4 power cancel | pass | 0 | yes |
| T3 helper | pass | 0 | yes ("Hello") |

### Murat (T1 T2 T4:volume T4:screenshot T6)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| T1 | pass | 0 | yes |
| T2 Help and Support | pass | 0 | yes |
| T4 volume | pass (set-volume 75% in calls.log) | 0 | yes (75% ticked on reopen) |
| T4 screenshot | pass (grim + notify-send) | 0 | no: no visible sign, persona unsure (as r3) |
| T6 honest failure | pass | 0 | yes: Run box shows `Nothing found for "nosuchprogram"` (fuzzel opens here) |

### pixel-watcher (X2 T2 T5 T4:lock; --probe)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| X2 Start menu, drag | pass (real: dragged persona-probe by its title bar) | 0 | yes |
| T2 key list | pass | 0 | yes (Super+F1 opens Keyboard shortcuts) |
| T5 no stray keys | pass | 0 | yes (Super+d x2, Super+e) |
| T4 lock | pass (swaylock -f) | 0 | unclear: persona saw a dark box that vanished (harness swaylock stand-in) |

Breaches: no breaches.jsonl was written in any of the three runs (0 lines).

### Fix check
| Finding | Reached | Result | Capture |
|---|---|---|---|
| xp-36 Run... that fails says so | Murat: Run..., typed nosuchprogram | not reached as a failure: fuzzel 1.11.1 opens, so the box answers itself ("Nothing found") and no "Run could not start" path fires; the failed-start notice is only covered by the kit cells | murat 015-017 |
| xp-35 balloon over the volume menu | Murat: Notepad, then the speaker menu | pass (no balloon, menu clear, 0 I5) | murat 003, 008-010 |
| xp-24 text fields (Run box) | Murat: Run box rendered | reached, looks aligned (Open: field, rows); not pixel-measured | murat 015-017 |
| xp-26 tick of the level | Murat: 75%, reopen | pass (75% ticked) | murat 010 |
| xp-27 question card | no persona met a question | not reached (cell only) | none |
| xp-33 Super+F1 key list | pixel | pass | pixel 006 |
| xp-31 Super+D toggle | pixel | pass (second Super+d brings windows back) | pixel 007, 008 |

### triage.py (the p2r4 xp lines; whole run: waiting 153, overdue 27, missing ux 0, FORMAT OK)
```
== xp / contract T4:clock (4)
  2026-11-20-p2r4-erika.md: Clock contract red: no clock node (0 days)
== xp / none (31)
  2026-11-20-p2r4-erika.md: Notepad window is a dark terminal with a locale warning
  2026-11-20-p2r4-murat.md: Screenshot gives no sign that it worked
  2026-11-20-p2r4-murat.md: Notepad window is a dark terminal
  2026-11-20-p2r4-pixel.md: Lock gives a half-drawn dark box that vanishes on mouse move
  2026-11-20-p2r4-pixel.md: Super+D leaves the Help window visible
  2026-11-20-p2r4-pixel.md: Super+E names a config file in its notice
```
Files: feedback/2026-11-20-p2r4-{erika,murat,pixel}.md; captures media/2026-11-20-r4/{erika,murat,pixel}/. No ledger edits.
Stop rule (p2-plan 3): xp meets it. No breach, one red contract (T4:clock, a known checker-id gap, twin gate-doors clock-door; the clock was read), no new BLOCK or MAJOR candidate; xp-35/26/33/31 hold, xp-36 and xp-27 not reached (cell only), and the r3 "Run... opens no Run box" did not recur on fuzzel 1.11.1. Open ux for triage: screenshot gives no visible sign (unchanged since r3).
