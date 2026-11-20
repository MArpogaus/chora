
## r3
Setup: branch claude/p2-r3-xp at c61ad6c8 (xp-35 027a8218 and xp-24 2d1e06c6 in); `cargo check --workspace` and `cargo build --workspace` green; round.sh snapshot once, binaries of c61ad6c8 in /tmp/persona-round/bin; sessions xp 1280x800, PERSONA_CHECK=1, 2 slots (Erika and Murat together, the pixel-watcher after); helper transcript seeded (user and agent entry) after each start; for T5 a chora-probe window by `round.sh probe` (driver slip again: it has no title bar, so X2's drag could not work; persona-probe was the window to use). Haiku personas, act.py only, caps 45 actions / 20 looks (used: Erika 30 actions, Murat 42/20 looks, pixel 29/16). Goals told in user's words, plus "close a window you only opened to look at" (the open Help window of r2 no longer turns contracts red). No code changed.
Round binaries: sha c61ad6c8 (/tmp/persona-round/bin, one snapshot).

### Erika (check.py end T1 X1 T4:clock T4:power T3)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| T1 Notepad from Start | pass | 0 | yes (typed "Dear Anna" shows) |
| X1 minimize, find again | pass | 0 | yes (taskbar button) |
| T4 clock | FAIL: no clock node (checker id, 3rd run) | 0 | yes (8:57 AM) |
| T4 power cancel | pass | 0 | yes |
| T3 helper | pass | 0 | yes ("Hello") |

### Murat (T1 T2 T4:volume T4:screenshot T6)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| T1 | FAIL: no new focused window (a later desktop click took the focus) | 0 | yes |
| T2 Help and Support | pass | 0 | yes |
| T4 volume | pass (set-volume 75% in calls.log) | 0 | yes (75% ticked on reopen) |
| T4 screenshot | pass (grim + notify-send "saved" x3) | 0 | no: no visible sign, persona unsure |
| T6 honest failure | FAIL: no notice, no Run box | 0 | no (Run... opens nothing) |

### pixel-watcher (X2 T2 T5 T4:lock; --probe)
| Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|
| X2 Start menu, drag | pass (vacuous: probe has no title bar) | 0 | menu yes, drag no (harness) |
| T2 key list | pass | 0 | yes (Super+F1 opens Help on Keyboard shortcuts) |
| T5 no stray keys | pass | 0 | yes (Super+d x2, Super+e; probe got no Super key) |
| T4 lock | pass (swaylock -f) | 0 | yes (Screen locked shows) |

Breaches: 0 lines in all three breaches.jsonl.

### Fix check (xp findings fixed since r2, plus the r2 not-reached ones)
| Finding | act or cell | Reached | Result | Capture |
|---|---|---|---|---|
| xp-35 balloon over the volume menu, balloon gone after Notepad | acts: volume popup opens while the balloon shows (I5); balloon goes when Notepad shows | Murat: Notepad then speaker menu | pass (no balloon after Notepad, menu clear, 0 I5) | murat shots 003, 009, 010 |
| xp-24 text fields (Run box row 28 px) | Run box rendered, search, dialogs | Murat clicked Run... twice | not reached: the Run box does not open here (fuzzel 1.9.2 rejects `[colors].prompt`, see new finding) | murat 019, 020; media/2026-11-20-r3/verify-run/ |
| xp-26 tick of the level | act popup ticks | Murat: 75%, reopen | pass (75% ticked; stand-in now keeps the level) | murat 010 |
| xp-33 Super+F1 key list | act keys key | pixel | pass | pixel shots after Super+F1 |
| xp-31 Super+D toggle | second Super+d | pixel | pass (hidden, then back) | pixel shots |
| xp-32 speaker click opens volume | act click on speaker | Murat | pass | murat 008-010 |
| xp-23 paperclip / helper | cell gate-xp-paperclip | Erika via Start "Chat with a helper" (not the clip) | pass for the chat and answer "Hello"; the clip click itself not tried | erika shots 011-013 |
| xp-27 question card | cell gate-xp-question | no persona met a question | not reached (cell only) | none |

### triage.py (the p2r3 lines of its output; whole run: waiting 123, overdue 27, missing ux 0, FORMAT OK)
```
== xp / contract T1 (3)
  2026-11-20-p2r3-murat.md: T1 contract red: Notepad not focused at the end (0 days)
== xp / contract T4:clock (3)
  2026-11-20-p2r3-erika.md: Clock contract red: no clock node (0 days)
== xp / contract T6 (3)
  2026-11-20-p2r3-murat.md: Run... opens no Run box (0 days)
== xp / none (24)
  2026-11-20-p2r3-erika.md: Notepad window is a dark terminal with a locale warning (0 days)
  2026-11-20-p2r3-erika.md: Small yellow square left on the desktop after clicks (0 days)
  2026-11-20-p2r3-murat.md: Screenshot gives no sign that it worked (0 days)
  2026-11-20-p2r3-murat.md: Volume menu shows a tick but no level bar (0 days)
  2026-11-20-p2r3-pixel.md: The window to drag has no title bar (0 days)
  2026-11-20-p2r3-pixel.md: Super+E names a config file in its notice (0 days)
  2026-11-20-p2r3-pixel.md: Lock screen stayed for three looks (0 days)
waiting 123, overdue 27, missing ux 0
```

Files: feedback/2026-11-20-p2r3-{erika,murat,pixel}.md; captures and logs media/2026-11-20-r3/{erika,murat,pixel}/ and verify-run/ (driver repro of the Run box). No ledger edits.
Harness candidates: round.sh probe should start persona-probe (X2 drag, 3rd run); T4:clock id; pointer marker (yellow square) in captures.
Stop rule (p2-plan 3): xp has one new check-backed finding, "Run... opens no Run box" (contract T6 red, task of the shared bar). It is probably env (fuzzel 1.9.2 rejects `[colors].prompt` at fuzzel.ini:19; this line stood in the ini at r2 when the box worked, so the fuzzel of the r2 lane differs): triage to decide `rejected: env` or a fuzzel.ini fix. No breach, and xp-35 and xp-26 hold. If it is env, xp meets the stop rule (no new BLOCK or MAJOR); if not, it is a BLOCK on T6. Round 3 of 3 is the cap, so xp does not go to round 4 either way; the Run box needs a cell (fuzzel accepts the ini) before a fix.
