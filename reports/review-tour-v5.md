# Review of tour v5 (independent, sonnet, read only), 2026-11-17 01:1x UTC

Build 4105f8d1; tour media/2026-11-17/tour-v5/tour-all.mp4 (3:48). Frames every 2 s plus dense crops;
media-r3 read only after the findings. PM saw j1-4 and N2 on the stills (02-01, 04-12).

| time | look | sev | finding |
|---|---|---|---|
| 1:07-1:53 | tiling | MAJOR | rv5-1 no mouse path to lock or power: the bar has no power icon though the key list names one |
| 1:53 | tiling | MAJOR | j1-5 Super+Print shows no notice (stills/02-25) |
| 1:07-1:55 | tiling | MAJOR | j1-4 "No helperVolume 40%" runs together (stills/02-01) |
| 0:03-0:19 | xp | MAJOR | j1-2/N1 Notepad is nano in a terminal; lowriter fails with only a notify error, no fallback |
| 3:30-3:40 | handover | MAJOR | rv5-2 mc leg: the window is never drawn in mc for about 6 s; audit says owner xp, none lost |
| 2:37-3:03 | construct | MAJOR (lean) | rv5-3 HUD has no helper button; helper only by Super+A |
| 2:46 | construct | MAJOR | N2 (media-r3): stepped back after the pull shows neither pane nor window |
| 0:24-0:58 | xp | MINOR | chat stays "Working, step 1 of 1" with Stop for 30 s after the answer |
| 2:32 | mc dark | MINOR | "No helper" while a "Helpers: Atlas working" row shows |
| 2:04 | mc light | MINOR | Super+a row overprints in the key column with a note |
| 2:28 | mc dark | MINOR | launcher cream box, raw entries (j1-6) |
| 3:05 | handover | MINOR | dark bands; tiling screen black without bar early; Start menu stays open |

Journey-1: co-18 BLOCK fixed (audit card), mc-32 BLOCK fixed, j1-1 fixed, j1-3 partly (mc leg),
j1-2, j1-4, j1-5 open. Disagreement with media-r3: the mc leg of the handover is not followable.
Verdict: FIX. Fix lanes: tiling-look r3, xp-polish r5, construct r2, mc r6.
