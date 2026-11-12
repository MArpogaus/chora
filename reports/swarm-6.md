# Swarm 6 triage (2026-11-12, build 8ca9802 = 8f358fa + persona harness)

Three haiku personas, 30 steps each, 1280x800, one round (budget past the
85 % swarm limit; user 02:17 UTC asked for testing to the week's end).
Reports: feedback/2026-11-12-swarm6-{erika6,kai6,nurse6}.md; shots under
media/2026-11-12/swarm-6/. Haiku reports are leads, not proof: each MAJOR
below needs a capture check by the fix lane before work starts.

## Cross-look (helper, every look)

| # | Finding | Looks | Rating | Note |
|---|---|---|---|---|
| X1 | the helper shows the question but never answers, no waiting or no-helper state | xp, mc, tiling | MAJOR | harness has no agent backend; the honest "no helper" state (RC) is missing either way |
| X2 | text typed before picking a helper is dropped; Enter does nothing until a helper is picked; "1 atlas" unlabelled | xp, tiling | MAJOR | first-use path of the chat |
| X3 | stray yellow square in the top-left corner | mc, tiling | MINOR | likely a helper surface at 0,0 |

## Per look

| # | Look | Finding | Rating | Note |
|---|---|---|---|---|
| M1 | mc | Left/Right do not turn pages; Escape does not go back | MAJOR | RC: every task by arrows, OK, Back |
| M2 | mc | first page does not say what it is; key text too small at 3 m | MAJOR | RC (first screen); known (ux-deep-mc) |
| M3 | mc | no on-screen launcher; only Super+Space | MAJOR | RC (start a program by click) |
| T1 | tiling | chat panel white, default style | MAJOR | known (ux-deep-tiling) |
| T2 | tiling | bar Keys and Chat buttons do nothing on click | MAJOR | RC (mouse path) |
| T3 | tiling | no focus-left/right key; Super+Right switches workspace | MINOR | key design question |
| T4 | tiling | theme switch, lock not found | expected | look entries are on swarm5-fixes (not merged) |
| P1 | xp | Start menu offers LibreOffice, which is not installed: "Cannot open" after 30 s | MINOR | container lacks it; the menu should list installed programs only |
| P2 | xp | no volume icon in the tray | MAJOR | RC (c) volume in every look |
| P3 | xp | paperclip tiny and unlabelled | MINOR | taste |

## Next week
X1, X2, M1, M3, T2, P2 are new main-path MAJORs; they go into the week-2
fix lanes by area (helper, mc, tiling, xp). No repeat round this week.
