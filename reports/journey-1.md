# Journey 1: tour v4 review (rc 11, round 1)

Material: media/2026-11-16/tour-v4 (media r2, claude/media 2ef891d), 3:18, six chapters.
Builds: the session branch before train 24's lanes land, so xp-23, xp-24, ti-21, sh-23, co-16 are expected open.
Reviewers: PM (contact sheets every 2.5 s, scratchpad pm-tour/) and one independent opus reviewer (below, when back).

## PM findings
| id | sev | chapter | what | owner |
|---|---|---|---|---|
| j1-1 | MAJOR | xp 0:20-0:55 | helper is a bare yellow panel, no paperclip (= xp-23) | xp-polish r1 |
| j1-2 | MAJOR | xp 0:05-0:50 | "Editor" for Erika (74, mouse only) is nano in a dark terminal with ^X keys; not XP, breaks POLS for the persona; LibreOffice Writer is installed (seen in the mc launcher) | xp-polish r2 / media |
| j1-3 | MAJOR | handover 0:00-0:38 | two screens scaled into one 1920x1080 frame with black margins: windows unreadable, the move between looks cannot be followed | media r3 |
| j1-4 | MAJOR | tiling bar, all frames | plain dark bar, "No helperVolume 40%" collides (= ti-21) | tiling-look r1 |
| j1-5 | MAJOR | tiling, screenshot act | Print / Super+Print gives no visible notice (media report): shared bar "honest states" | tiling-look r1 check, else new |
| j1-6 | MINOR | mc dark 0:08 | program launcher is a cream light box on the dark theme; raw entries (UXTerm, Foot Server, ImageMagick) | known issues |
| j1-7 | MINOR | mc light 0:22 | urgent-note ticker overlaps the page footer line | known issues |
| j1-8 | MINOR | construct | 0:19 only; window pull not shown (scripted drag fails) | media r3 |
| j1-9 | COSMETIC | all title cards | dated 2026-11-12 | media r3 |

Pace: 3:18 total (owner 09:01 "too slow"): met. freezedetect 19 spans 1.5-2.4 s: media r3.

## Independent reviewer
(pending)

## Verdict
FIX: five MAJOR (j1-1..j1-5); j1-1 and j1-4 are already in running lanes, j1-2, j1-3, j1-5 new.
