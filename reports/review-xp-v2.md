# Review: xp v2 pixels (independent, round 1)

Reviewer: opus UI review, did not build it. Branch `claude/xp-v2` at 52e7044 (binaries 02:16:38 UTC are newer than the
head commit 02:16:04, no rebuild). Session: `tests/persona/session.sh xp 1280x800`, flavor theme from the checkout,
`--test-injected-press-is-real`, a fake voice agent as `tests/gate-hud.sh` starts it (with a transcript and a question).
34 looks at 1280x800 plus 3 at 2560x1600. Captures: `media/2026-11-12/review-xp-v2/` (named below).
Judged against xp-v2.md, shared.md, mocks 1 to 3, decisions-taste (xp), product-briefs (chora-xp), rules.md 8.

## Verdict: FIX

Four MAJOR items, three of them visible on the main path (Start, Help, Run). The look is otherwise close to the mock: taskbar,
caption bar, pager, balloon, question card and the paperclip are clean. The builder's s2 stills must not ship (see M5).

## First look at xp-v2-r2 (before notes)

- `desktop-s1`: wallpaper, cloud and icons good. Icon labels are about 12 px with 12 px line spacing ("My / Documents" lines touch);
  the My Documents folder is a pale translucent blue on a blue sky (low contrast), My Computer is saturated: two icon styles.
  Recycle Bin label ends 13 px from the edge. Tray: clock ends 19 px from the screen edge, first tray icon 7 px from the tray edge
  (uneven). The mic-off glyph is a tiny white block under a red sign, unclear.
- `start-s1`: a four-colour waving flag on the Start button and in the user tile (the Windows flag). "Chat with a helper" has a
  plain green dot, not the paperclip-in-balloon of the mock. Run... icon is a faded grey gear pair; ImageMagick row ends "(color depth..." ;
  a 3 px green line between the white body and the blue footer. User name "User".
- `help-s1`: looks like a plain foot window pair, no help window at all (the file shows the wrong state).
- `chat-question-s1`: balloon fine; placeholder wraps in two lines whose left edges differ by 5 px (959 vs 954) and the text box
  is 38 px high for a two-line placeholder; "Typing does not answer." sits outside the card, as the doc asked not to.
  Paperclip crisp at 80x116, tail tip about 16 px above the head, 6 px left of its centre.
- `*-s2`: 2560x1600 PNGs with the shell drawn only in the top-left 1280x800 quarter and black elsewhere, balloon at the far corner.

## MAJOR

| # | What | Measured | Fix |
|---|---|---|---|
| M1 | Windows flag lookalike (red, green, blue, yellow waves) on the Start button (15-30 x 773-790), the Start-menu user tile (`15-start-user-tile-4x`) and the Turn Off dialog title (757-785 x 310-337). xp-v2.md line 12 and product-briefs say no Microsoft flag. | 3 places | Replace by an own mark (a four-leaf or abstract wave in one palette, or the "start" word alone); same in the user tile (use a user glyph or the account photo as the mock). |
| M2 | Help and Support "Keyboard shortcuts" table is cut at the window bottom and cannot be scrolled (no scroll code in `help.rs`, wheel does nothing, no max button): the row at y 690 is cut in half, every row after "Ctrl+Alt+Shift+Right" is unreachable (`06-help-keys-clipped`). The doc promises every bound key is in the table. | about 17 rows fit at 1280x800, later rows are lost | Scroll (wheel, a 16 px scroll bar) or put the table in two columns / smaller groups; check with the gate that the last row is on screen. |
| M3 | Run (Start > Run..., Super+Space) opens the shared launcher in monospace, translucent cream (text box 244,242,229 over the wallpaper, wallpaper shows through), "Foot" instead of "Terminal", truncated "ImageMagick (color depth=…". It reads as a different product (`03-run-launcher`). | 350x320 box, mono font, not Tahoma, not black-edged | Theme fuzzel from the xp palette: Tahoma 14, opaque #FFFFE1 or white, 1 px #000 or #7F9DB9 edge, #316AC5 selection. |
| M4 | False launch balloons: launching with Super+Space showed "Starting fuzzel / Please wait." that stayed 30 s, then "Cannot open fuzzel / No window opened in 30 seconds" although the launcher was on screen and worked (`04-cannot-open-fuzzel`). The chat open path showed "Starting chora-chat / Please wait." for about 40 s over the working chat (`07-chat-open-starting-balloon`). A layer-shell launcher is not a toplevel, so the tracker never sees it. | 30 s wait, then an error on a success | Do not track launcher/layer surfaces; clear "Starting" when any new surface of the pid maps; no error balloon when the program answered. |

M5 (capture, not product) the `*-s2` stills are invalid evidence: the 2x frames show the shell drawn at 1x in a corner. I
reproduced: `configure-output --scale 2` on a live session shows a zoomed wallpaper without taskbar for about 5 s, then
a correct 1280x800 logical desktop (`16-scale2-settled`). Re-take s2 after 10 s; the 5 s transition is a MINOR.

## MINOR

1. F1 alone does nothing: `chora-xp: F1: F1 needs Ctrl, Alt or Super or the capability input.grab-any (code 4)` in chora.log. D3 says F1 is global; only Super+F1 and Start > Help and Support work. The table honestly lists Super+F1 only. Fix: grant `input.grab-any` to chora-xp in the flavor, or say "Super+F1" in the brief.
2. Caption buttons are 22 x 22 with 4 px gaps (min 653-674, max 679-700, close 705-727 at y 28-50), spec 24 x 24, 2 px (D2; the 24 px target rule P7). `14-caption-buttons-6x`.
3. Help window has only a close button (no minimize, no maximize); title 14 px bold is fine.
4. Notify balloons use 12 px type (title bold 12, body 11-12), below the 14 px floor (D1); long technical text "name it in the setting file_manager of chora-xp.yaml" is for a developer, not Erika. Balloon at the tray overlaps the paperclip: the paperclip is drawn over the balloon's right 25 px (`10-no-file-manager`, `04`).
5. Turn Off: Cancel is 56 x 17 px (target under 24 px), labels 12 px; the paperclip stays in colour over the greyed desktop (`11-turn-off`).
6. Chat: placeholder in the field wraps in two lines with different left edges (5 px) while a question waits (`08-question-card`); the mock has one line "A question is waiting above." and a 28 px field. Send is enabled with an empty field (`07`). Agent list row is "1  atlas" with an index and lowercase name (polish #19 asked for none). 54 px blank between transcript and card, 38 px between card and field.
7. Question card lacks the mock's key caps for Alt+Y/Alt+N and the "time left", the title is "Question for you" (mock: "Question from Atlas"); the answer leaves no line in the transcript (cannot judge with a fake agent, needs the real one; `09-after-yes`).
8. Balloon tail tip is about 16 px above the paperclip's head and 6 px left of its centre (xp-v2 #14 asked tip on the head). hud-audit overlap kind reports `hud-chat 932,170 340x470 and hud-figure 1192,638 80x116` (2 px, the tail; xp marks no surfaces).
9. Folders in the right Start column and on the desktop are still pale line-art blue next to a saturated My Computer (polish #5); Run and Control Panel icons are desaturated; the pinned chat icon is a green dot, not the paperclip balloon.
10. Tray padding: first icon 7 px from the tray edge, clock 19 px from the screen edge (spec 12/12).
11. Desktop labels about 12 px, line spacing 12 px (the two lines touch).
12. A maximize click right after a taskbar restore deactivated the window and did nothing; the second click maximized (`12-maximized`).
13. Wallpaper strip: 3 px green line between the Start menu body and its blue footer.
14. Tail of a live scale change: the shell shows a zoomed wallpaper without taskbar for about 5 s.

## COSMETIC

- Start menu user name "User" (GECOS not set here, fine). Menu text right edge of the pinned subtitle "Type commands" grey #8A8A8A on white (about 3.4:1; the ink-2 token #4D4D4D asks 8:1).
- Help page: "Show the desktop; press again for the windows" ends 1px from the rule end (1003 vs 1004), no right padding.
- The paperclip's face (eyes, brows, wire clip) is close to a well known assistant; decision-wise accepted, flag for the user once.

## Checked and good

Taskbar 36 px (y 764-800), frame 4 px, title text 14 px bold white, pager buttons 24 x 24, task button 180 x 28 (126-306 x 768-796),
Super alone toggles Start, Super+D hides the windows and a second press brings them back, Super+Space opens the
launcher, Esc closes Help, the Help window has a taskbar button, the Help topic page and key caps look like XP, Stop is greyed
when no prompt runs and Send is greyed while a question waits, Yes writes `{"id":"51","yes":true}`, minimise and restore through the
taskbar button work, balloon is #FFFFE1 with a black 1 px border and a 340 px column, paperclip hops without blur.
hud-audit (`origin/claude/tooling:tests/hud-audit.py`, live session, owners assistant xp notify, kinds image,outside): `no finding`,
so the paperclip at 80x116 is no longer resampled and starts on a whole pixel; the full run prints only the 2 px tail overlap above.

## Main-path flaws, any lane

| Path | Flaw | Lane | Sev | Blocks |
|---|---|---|---|---|
| Start button, menu, Turn Off | Windows flag lookalike x3 | xp assets | MAJOR | yes |
| Help > Keyboard shortcuts | table cut, no scroll, rows lost | xp help | MAJOR | yes |
| Start > Run, Super+Space | launcher is mono, translucent, off-look | launcher / shared | MAJOR | yes |
| Run, chat open | "Starting..." 30 s, then false "Cannot open" | xp launch tracker | MAJOR | yes |
| F1 | key refused (grab-any) | xp / core grants | MINOR | no |
| Every window | caption buttons 22 px, gap 4 | xp wm | MINOR | no |
| Balloons | 12 px type, overlap with the paperclip | notify | MINOR | no |
| Turn Off | Cancel 56x17 | xp dialog | MINOR | no |
| Chat | placeholder 2 lines misaligned, "1 atlas" row, Send enabled when empty | assistant hud | MINOR | no |
| Question card | no key caps, no time left | assistant hud | MINOR | no |
| Desktop, menu | pale folder icons, 12 px labels, green dot chat icon | xp assets | MINOR | no |
| Tray | uneven padding 7 / 19 | xp tray | MINOR | no |

Round 2 should confirm M1 to M4 only.
