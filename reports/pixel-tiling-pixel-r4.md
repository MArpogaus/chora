# Pixel review: chora-tiling, branch claude/tiling-pixel @ f9dabd84, round 4 (2026-11-12)

Reviewer did not build it. Captures: media/2026-11-12/tiling-pixel-r4/*.png = 46 PNGs (r3's 38 plus chatq and chattoast, Ink and Paper, 1280x800 and 2560x1600@2; desktop also at 1600x1000@1.25). All read; measurements with PIL (1x px unless stated). Every r4 PNG was diffed against its r3 twin: fullscreen identical, chat/keys/launcher/layout/toast/tooltip/desktop differ only in the bar clock and (2x, 1.25) the bar icon rows y32..62 / y20..39; question differs by the new typeface. So every r3 finding not named below is pixel-identical and keeps its r3 status. Each 2x was BOX-downsampled and diffed against its 1x: only glyph antialiasing differs, nothing moved.

## r3 findings: status

| # | r3 | Now | Evidence |
|---|---|---|---|
| R6 bar icons soft at 2x/1.25 | MINOR | FIXED | Apps grid midtone ratio (pixels between 15 % and 85 % of max contrast / all ink pixels): r3 2x 336/628 (53 %), 1.25 148/248 (60 %); r4 2x 32/468 (7 %), 1.25 42/200 (21 %); 1x unchanged 36/132 (27 %, which is the 1x floor). Nearest-neighbour zoom of Apps grid, keyboard and power at 2x and 1.25: clean rounded squares and stroke, edges as sharp as the "Apps"/"Keys" text beside them, both themes. Icon size unchanged (see #10). |
| R7 question card typeface | MINOR | FIXED | question-*: text extents x710..952 ("Atlas wants to write the file notes.txt.", 243 px), x711..965 ("Press Alt+Y to allow or Alt+N to refuse.", 255 px), x711..845 ("Question from Atlas" bold, 135 px). Inter 14 px advance widths from the installed Inter OTF: 244.4, 257.0, 136.0. Liberation/Arial would be about 5 % narrower and has a different x-height; match is within 1.5 px on all three, so Inter 14. Same in Ink and Paper, 2x identical after downsample. Residual cosmetic C1. |
| R13 chat vs question/toast | unverified | FIXED | chatq: panel x868..1268 y52..788; card moved from x696..1255 to x296..855 (shift 400 = panel width, 12 px gap to the panel), y574..744 unchanged, Yes/No (x310..398, 406..494) fully visible and clickable. "Question for you" chip + red orb at x694..854 y751..777 left of the panel. chattoast: toast x496..856 y664..738, 12 px from the panel, chip "Listening" + orb x744..852 left of the panel. Nothing hidden in either theme at 1x and 2x (2x: card 592..1710 px device / 2 = same). Consequences: the card and toast now overlap the left tile's focus border (x~634) and the right tile's text area; acceptable overlay. |
| R2 toast LCD AA | COSMETIC | OPEN | chattoast-paper "Build finished" still shows red/blue fringes. |
| R3 orb ring | COSMETIC | OPEN | unchanged. |
| R4 toast type (16 px bold/light) | COSMETIC | OPEN | unchanged. |
| R5 chat header | COSMETIC | OPEN | unchanged (chatq/chattoast show the same bold sentence + lone Close). |
| R8 launcher size/selection/shadow | COSMETIC | OPEN | launcher PNGs identical to r3. |
| R9 tooltip shadow/covers pill | COSMETIC | OPEN | identical. |
| R10 1.25 hairline neighbour, 2.5 px tile bottom | COSMETIC | OPEN | identical outside the icon rows. |
| R11 island width jumps | COSMETIC | OPEN | identical. |
| R12 bar not dimmed in keys | COSMETIC | OPEN | identical. |
| #6, #8, #10 (bar icon sizes; now crisp), #12, #16, #17, N5, N6 | COSMETIC | OPEN | identical. |
| R1, #1-#5, #7, #9, #11, #13-#15, #18-#20, N1-N4, N7 | | FIXED, held | no pixel change. |

## Known gap judged: mako started while chat is already open

The toast keeps its right-anchored slot (x896..1255 per r3), which lies entirely inside the open chat panel (x868..1268), so it is fully hidden (verified geometrically: the slot is 100 % under the panel, 0 px visible). Severity MINOR (N1 below): a notification is silently lost to the person who has chat open, but it is transient, informational and not an action prompt; the security question card is handled by the chat-aware path (R13 fixed) and is not affected. It is not MAJOR because nothing requiring a decision is hidden. It is not COSMETIC because content is lost with no cue. Fix: let the toast daemon/compositor shift its anchor on the chat open event even when mako started later (read chat state on startup, not only on a change signal), or draw the toast above the chat layer.

## New findings

| # | Rating | File / coords | What is wrong | Fix hint |
|---|---|---|---|---|
| N1 | MINOR | toast with chat already open at mako start (no capture; geometry from toast-*/chat-*): toast slot x896..1255 y677..725 under panel x868..1268 | Toast hidden under the chat panel as described above. | See above; add a capture "chat first, then toast". |
| C1 | COSMETIC | question-*: x710..965 y592..685 | Card is one size (14 px) for all five lines, title is bold (700) where the other helper titles are semibold; the file path "~/notes/notes.txt" is plain body text, not mono or muted; no 1.25 capture of card, chat, toast, launcher (only desktop at 1.25). | Title 14 semibold, path in the mono token, muted for the Alt+Y hint; add 1.25 captures. |
| C2 | COSMETIC | chatq-*: chat panel y52..788 | With a question pending the open chat shows only "A question is waiting." and Close over an empty panel; the right tile's "Chat ..." pill is under the panel. | Optional: show the question text in the chat, or dim the empty panel. |

## Verdict: MERGE-OK

No MAJOR at 1x, 2x or 1.25 in either theme. The two r3 MINORs are closed with measurements (SVG icons crisp: midtones 336 to 32 at 2x; Inter on the card: widths within 1.5 px of Inter 14), and R13 is closed: an open chat moves the card and the toast to its left with a 12 px gap, nothing covers a prompt. One new MINOR keeps "nothing above COSMETIC" unmet (N1, late-started mako toast hidden under the chat).

Counts. r3 items: FIXED 3 (R6, R7, R13), OPEN 17 (all COSMETIC: #6, #8, #10, #12, #16, #17, N5, N6, R2-R5, R8-R12), REGRESSED 0. New in r4: MAJOR 0, MINOR 1 (N1), COSMETIC 2 (C1, C2). Open by severity: MAJOR 0, MINOR 1, COSMETIC 19.
