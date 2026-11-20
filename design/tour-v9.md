# Tour v9: one scenario per look, every act checked, compared with tour v8

Plan of 2026-11-20 (tour-plan r1, F7 and RC 11), read on code 9dc384b3. Build next week.
Owner 11-17 14:24: "refactor, then simulated user test, then the new tour with automatic checks +
visual validation against ux ui guidelines"; 11-17 14:20: v8 was "horrible", the new one must be
smoother and more natural. F7: tour v9 at the end matches tour v8 per look; findings only for changes.
Inputs: vision-card.md, rc.md 11, tour-v4.md, review-tour-v7.md, media-r6.md and the v8 timeline
(claude/media-v8, build 733967aa), proposal-glitch-gates.md, persona reports p2-r4-xp, p2-r2-tiling,
p2-r5-mc, p2-r3-co, xp-give r1; code: demo/tour, tests/acts, tests/persona/check.py, tests/doors.

## 0. Shape and what is reused
- Five chapters as in v4 (xp, tiling, mc light and dark, construct, handover), at most 6 min with
  slides; one take per chapter, the pace of `tour8.Quick` (human.py, seeded, so a run repeats).
- An act is a row: id, door, cues, `expect`, `contract`, `cells`, `capture`. The tour reads the
  session between segments already (`Tour.flush`); there it attaches the act harness
  (`harness.Session.attach`, as persona check does), takes a snapshot, runs `Expect.check` and
  every invariant of `invariants.ALL` (I1-I14). A red act is a red row in timeline.md, not a
  reviewer's catch (review-tour-v7 proposal 5, media-r6, proposal-glitch-gates 1-2).
- `contract` names a persona contract of tests/persona/check.py (T1-T7, X1, X2, L1-L3, M1, M2,
  C1-C3); `cells` names the gate cells the act repeats on screen (tests/doors/lib.sh ids, acts of
  tests/acts/looks/*.py, gate scripts). No act brings a new cell; where an act needs more, the
  row says which existing cell it extends (marked "extends").
- The same rows run without video and pacing as an opt-in gate step (`demo/tour/run.py --check`):
  the tour cannot rot between recordings.
- Door names (vision card, shared RC bar): Launcher, Keys, Helper, Stray keys, Lock, Power,
  Volume, Screenshot, Clock, Failure, Handover (no lost window). "own" = the look's own form only.
- Capture frames: **Q** = first frame after the act is quiet (every `busy` empty) plus 2 frames;
  **Q+t** = t seconds after Q; **H50** = the held frame at half the motion (`act(held=)`, stepped
  clock frame n/2 after the cue). Stills are 1280x720 PNG without the pointer (masks below).

## 1. xp: Erika writes a letter, Murat sets the sound (mouse only; ~75 s)
Erika 74 and Murat 52 know XP; no key but typing. Fallback art (no import).

| # | Act | Door | Check (contract, cells; I1-I14 after every act) | Capture |
|---|-----|------|------|------|
| x1 | Erika sits at the desktop | Clock | doors clock-door (xp); I7 | Q |
| x2 | Start, Notepad: balloon "Starting Notepad", the window | Launcher | T1 (new window, owner xp, focused); acts "the Start button opens the Start menu", "a click on Notepad starts it", "the balloon goes when the window of Notepad shows"; I4 menu closed | Q after Start; Q after window |
| x3 | types three lines, one typo corrected, Ctrl+S | own (XP frame) | letter.txt equals the typed text (stand-in mousepad writes it; extends gate-xp-notepad-fallback) | Q |
| x4 | drags the window aside by its title: jelly wobble | own | X2; act "a drag of a window shuts the Start menu" (extends: the Notepad window) | H50, Q |
| x5 | paperclip, asks "How do I make the letters bigger?" | Helper | T3; acts "the chat of the helper opens", "the helper answers"; I8 (not Working after the answer: v8 open row) | Q+1 after answer |
| x6 | the helper wants to open Display settings: question card, Yes by mouse | Helper (asks first) | act "the helper asks"; gate-xp-question.sh; card does not move (extends I12: modal above, rect held) | Q (card), Q (after Yes) |
| x7 | Murat: speaker in the tray, 75 %, tooltip | Volume | T4:volume; acts "a click on the speaker opens the volume popup", "a click on 75 percent sets the level and shuts the popup", "the tooltip of the speaker says the level that the popup set" | Q popup; Q tooltip |
| x8 | Murat: Print Screen in the Start menu Accessories (or Super+Print) | Screenshot | doors screenshot-door/-key (xp); I9 (notice drawn, the p2-r4 "no sign" ux row) | Q+0.5 |
| x9 | Run..., types `nosuchprogram` | Failure | T6; gate-xp-run.sh; doors launcher-door; I10 | Q |
| x10 | Help and Support, Keyboard shortcuts | Keys | T2; acts "Help and Support ... opens Help on a filled pane", "the topic Keyboard shortcuts lists every key inside the window"; doors keys-door | Q |
| x11 | Turn Off Computer, Cancel | Power | T4:power; acts "Super+Shift+e opens Turn Off Computer" (extends: the Start menu click, doors power-door), "Escape shuts Turn Off Computer" (extends: Cancel) | Q |

Not in xp: the cube (WM7b: construct holds the face turn, shown in chapter 4); Control Panel
wallpaper (Desktop is grey, xp-T7); Lock (chapter 5).

## 2. tiling: Kemal builds a project, a beginner finds Apps (keyboard-first; ~65 s)
| # | Act | Door | Check | Capture |
|---|-----|------|------|------|
| t1 | Super+Return: a shell, `ls`, `nano notes.md`, two lines, save | Launcher (key) | T1; act "a window opens" | Q |
| t2 | Super+d, Super+q-less chords into the editor: nothing typed | Stray keys | T5 (probe tile: chora-probe window beside nano, its report has no key line); gate-injected-presses.sh | Q |
| t3 | three more terminals, Super+Shift+space through two layouts | own | L1; gate-tiling-gaps.sh and tiling-audit.py (gaps, borders); I2, I5 | Q per layout |
| t4 | Super+2 for the build, `cargo build` stand-in, Super+1 back | own | L3; gate-0.9-tiling-tags.sh | Q |
| t5 | Super+drag floats a window: wobble, drop | own | gate-tiling-wobbly.sh; I12 (float on top) | H50, Q |
| t6 | Super+F1 key list, Escape; Super+/ once | Keys | T2; doors keys-key, keys-alias; acts "the keys key opens the key list", "Escape shuts the key list of the keys key" (and the alias pair) | Q |
| t7 | Super+Print: the notice | Screenshot | doors screenshot-key; gate-tiling-print-notice.sh; I9 | Q+0.5 |
| t8 | the beginner clicks Apps, types `frobnicate`: Nothing found; then Foot | Launcher (door), Failure | T6; doors launcher-door, launcher-chat; gate-tiling-apps-click.sh; I10 (no raw entries) | Q list; Q failure |
| t9 | Super+a: the chat beside the tiles, Escape | Helper | T3; doors helper-key; acts "the chat of the helper opens/shuts"; I5 (ti-29 chat over tile: extends I12 with "chat covers no tiled window") | Q |
| t10 | volume pill scroll, power glyph: menu, Escape | Volume, Power | T4:volume, T4:power; doors volume-door, power-door; gate-tiling-power.sh | Q menu |

## 3. mc: the cook in a light kitchen, the sofa in the dark (voice and remote; ~75 s)
Light, the cook (hands busy): recipe pages from the scene script, fake voice agent.

| # | Act | Door | Check | Capture |
|---|-----|------|------|------|
| m1 | "next step" by voice: the page turns, the HUD line | Helper (voice) | M1 (page changed); voice events; I8 | Q |
| m2 | "how long does pasta boil?": the answer on the HUD | Helper | T3; act "the helper answers"; I5 (v8 open: Super+a row overprint, N-3 "No helper" while speaking) | Q+1 |
| m3 | the timer card, later the critical note | own (notice) | acts "notify.push of mc shows in chora-notify", "notify.clear of mc dismisses it"; I9 | Q card; Q note |
| m4 | Right, Left on the remote; Home | own | M1; acts "Right of a remote turns the page", "Left of a remote turns back", "Home of a remote goes to the first page" | Q |
| m5 | Super+n: the larger page with the clock | Clock | doors clock-door (mc, also 1024x768 header-SIZE); I5 (v8 open: clock vs pill) | Q |
| m6 | Super+F1 key list on a page, Escape | Keys | T2; doors keys-key; gate-mc-paths.sh terminal-back (mc-42, mc-43) | Q |

Dark, the sofa in the evening (remote and mouse):

| # | Act | Door | Check | Capture |
|---|-----|------|------|------|
| m7 | "mc turns dark": the plan page, Right on the remote | own | acts "mc turns dark", "Right of a remote turns the page in dark mc" | Q |
| m8 | Programs by mouse: the list, Foot: page 7 FOOT | Launcher (door) | T1; doors launcher-door; I10 (v8 open: raw entries N-2) | Q list; Q page |
| m9 | Chat by mouse, a question, the answer, Escape | Helper (door) | T3; doors helper-door; I5 (r5 ux: volume readout hidden under the chat) | Q |
| m10 | volume up on the remote, the readout | Volume | T4:volume; doors volume-key | Q+0.5 |
| m11 | Super+Space by the remote keyboard, `nosuchprogram`: "Nothing found for ..." | Failure | T6; gate-mc-sofa.sh launcher-none (mc-44); I10 | Q |

## 4. construct: a demo watcher at the cube, Max pulls a window (~70 s)
Construct with xp on the four faces (as the acts session of construct).

| # | Act | Door | Check | Capture |
|---|-----|------|------|------|
| c1 | the walk room with the HUD strip, Super+T: a terminal pane | Launcher (key) | T1; doors launcher-key (construct); I2 (32 px margin) | Q |
| c2 | Alt+Escape: the cube stands back, xp on four faces | own | C1; acts "the cube opens with one xp on four faces", "the key of the step back stands back"; I13 | Q |
| c3 | turn by Ctrl+Alt+Right, by drag, by the pager chip | own | acts "the turn key turns the cube", "a drag on a face turns the cube", "the pager turns the cube", "the cube is held halfway through a turn"; I13 held | H50, Q |
| c4 | Super+F1: sheet "Construct · Cube", Escape | Keys | T2; doors keys-key; co-26; I4 (Escape closes the sheet only: p2-r3 ux) | Q |
| c5 | Return steps into a face; Max pulls the Notepad window off it (plain drag), turns, drops it on the next face | Handover | C2; gate-construct-pull.sh (plain and Super cells); act "a pulled window goes back to the xp of its face when its pane goes" (extends: a drop on a face) | H50 drag, Q |
| c6 | stood back: Lock, Power (cancel), Screenshot, Volume, the clock in the strip | Clock, Power, Screenshot, Volume | doors clock-door, power-door, screenshot-door, volume-door (construct); I9 | Q per door |
| c7 | Helper in the strip: chat, answer, Escape | Helper | T3; doors helper-door; I8 | Q |

## 5. handover: the nurse moves one window through all four looks (~60 s)
One session, four outputs (construct, xp, tiling, mc), as tests/gate-handover-matrix.sh.
The film follows the window: the output that holds it fills the frame (1600x900), the four outputs
run as labelled thumbnails (320x180) in a column, so a lost window would show.

| # | Act | Door | Check | Capture |
|---|-----|------|------|------|
| h1 | xp: Start, Notepad, "Ward 3: check bed 4 at 14:00" | Launcher | T1 | Q |
| h2 | title-bar right click, Send To > Tiling desktop | Handover | gate-handover.sh 8b (xp-give); Expect owner tiling, focus tiling; I1, I3 | Q |
| h3 | tiling: types a line; Super+Alt+3 | Handover | gate-handover-matrix.sh cell tiling-mc, key path; Expect owner mc, focus mc | Q |
| h4 | mc: the window as a page; Super+Alt+4 | Handover | gate-handover-matrix.sh cell mc-construct, key path (gate-give-holders.sh, mc holder); Expect owner construct; C2 | Q |
| h5 | construct: drag the pane onto the xp face (Super+Alt+2 if xp-give is not merged) | Handover | gate-construct-pull.sh drop; Expect owner xp | H50, Q |
| h6 | xp: the window with all lines; the window audit | Handover | T7 against the chapter's begin (not vacuous: one window, four owners); I1; letter text equals all typed lines | Q |
| h7 | Super+Shift+Escape locks every output | Lock | doors lock-key (every look); T4:lock | Q |

Door coverage: every door is shown at least once, Launcher, Keys and Helper in every look.

## 6. The comparison with tour v8 (F7)
v8 (build 733967aa) was recorded with other scripts, pace and size (1920x1080 joined, pointer
drawn); its 95 stills cannot be diffed by pixel. So the baseline is a **replay**: the v9 rows run
on a build of 733967aa with the same seed, size and stepped clock. Acts that 733967aa cannot do
(Send To, `state.get`, `scene.boxes` are later) are marked `baseline: none` and are judged by the
checks and the review only. The v8 stills stay the eye reference for the reviewer (map: act id to
v8 still name, in the chapter file).

Per act and capture, two diffs, both by `tests/tour-diff.py` (extends tests/capture-diff.py and
png-same.py, one shared PNG reader):
- **Scene diff** (always, also H50): `scene.query` on both builds; nodes matched by id, windows by
  app_id and title (surface ids differ). Compared: visible, rect (more than 2 px is a diff),
  meta text, owner, z-order of the top layers, open menus. Text is compared here as strings.
- **Pixel diff** (Q captures): masks first: the clock nodes (xp-clock, mc-clock,
  construct-door-clock, the tiling bar clock), every text box (`scene.boxes` on the candidate,
  text-node rects of `scene.query` on 733967aa), the pointer, a typing caret. A pixel differs when
  a channel differs by more than 24 of 255. The capture is **same** when at most 0.1 % of the
  unmasked pixels differ and no 8-connected blob of differing pixels has a box over 12x12 px.

Verdict per act:
- **same**: both diffs within the thresholds.
- **change**: a diff named in `demo/tour/intended.toml` (act, kind, cause: a commit or a finding
  id, e.g. N-6 fixed, xp-give Send To row, mc-42). A v8 open row (v8 timeline) that is gone in v9
  is a change too and needs its intended row; the review confirms it is a fix, not a loss.
- **regression**: an unnamed diff; or a check (contract, cell, invariant) that is green on the
  replay and red on v9; or a door cell pass on 733967aa and fail on v9. A regression blocks F7.
- The rule "findings only for changes" (F7): the review reads the change and regression rows; a
  v8 glitch that both builds show is a known issue, not an F7 finding (it goes to backlog).

Video checker on each chapter MP4 (tests/tour-video-check.py, extends the v4 `freezedetect` use):
black stretch over 2 frames (ffmpeg blackdetect), a still over 1.5 s outside a cut, flicker (a
frame that differs from both neighbours while they are equal), an empty or wrongly themed frame at
a change. Output: timestamps; a flagged stretch is a red row.

## 7. Visual validation against design/looks
Automatic (every Q capture, every act; a finding by eye must become one of these before its fix,
proposal-glitch-gates 4):

| Rule (design/looks) | Check | Extends |
|---|---|---|
| no text over text, clock vs pill (checklist 3, 4) | I5 | - |
| text inside its panel, padding (checklist 2) | I6 | - |
| contrast 4.5:1 text, 7:1 key line and buttons (shared "Readable", checklist 6) | png-contrast.py on each text box of `scene.boxes` | I6 |
| text a person must read at least 14 px (mc: its minimum) | run height of the reported runs | I6 |
| 32 px from the output edge (construct), work area (all) | I2 | - |
| gaps in whole units (construct 8, tiling gaps) | hud-audit.py, tiling-audit.py on the boxes | I2 |
| no raw launcher entries (`.desktop`, UXTerm-like) | I10 | - |
| words from the word table, no "agent", "HUD" | the visible texts against tests/keysheet/words.table | I10 |
| one helper surface per output, not Working after the answer | I8 | - |
| no menu left open, Escape backs out one step | I4 | - |
| no bar over fullscreen, modal on top, card stays | I12 | - |
| cube faces meet (seams) | I13 | - |
| a notice is drawn and not under the chat | I9 | - |
| no black output | I7 + video checker | - |
| no frame flash, no freeze, screen-fixed nodes still in a camera move | video checker; HUD rects equal at H50 and Q | I2 held |

Fresh-eyes review (rules 7; opus, not the builder, seeded with the checker output): aspect and
whole-number scaling of pixel art; Luna fidelity on fallback art (xp.md, xp-v2.md); alignment to the
pixel and baselines; motion feel (ease out, no bounce, timings of the look pages); theme fidelity
light and dark; whether the scenario reads as a real person's task and the pace feels natural
(owner 11-17); every change row of section 6. It reads flagged timestamps, the change and
regression rows and one full watch at 1x; each finding names act, capture and rule.

## 8. Build plan (next week; % of the week)
| Lane | Model, effort | % | Does |
|---|---|---|---|
| 1 TOUR-ONE | sonnet medium | 1.5 | one set (DB11, F2): `demo/tour/{tour,human,launch,slides,assemble,run}.py` and `chapters/{xp,tiling,mc,construct,handover}.py`; folds ch1-8, ch7b, v4-v8_*, tour4-8, assemble4-8 (4,291 lines of 5,307) into rows; keeps the fixes of v8 (PATH without writers, pane-centre aim, keep-bus, probe bus); act rows with expect, contract, cells, capture; `run.py --check`; a tools/copies.toml row so no `v*_` copy returns; red test first (an act whose expect fails is a red timeline row). Diagnoses F-1 (no notice in `choractl record`) |
| 2 TOUR-DIFF | sonnet medium (+ haiku tests 0.3) | 1.3 | tests/tour-diff.py, intended.toml, tests/tour-video-check.py, planted cases (a moved node, a changed blob, a flicker frame) |
| 3 GUIDE-CHECKS | sonnet medium | 1 | the "extends" rows of section 7 in invariants.py and the audits, each with a planted breach in tests/acts/planted.py |
| 4 CHAPTERS-A | sonnet high | 1.2 | xp, tiling, handover rows, `run.py --check` green or known red |
| 5 CHAPTERS-B | sonnet high | 1 | mc, construct rows, same |
| 6 RECORD | sonnet medium | 1 | builds 733967aa and the candidate; replay and v9 take; diffs; video check; timeline, stills and verdicts committed with the media |
| 7 REVIEW | opus high | 1 | fresh eyes as in section 7 |
| reserve | - | 1.5 | one fix round and a re-take of the chapters it touches |

Order: 1 alone (the API the others use); then 2, 3, 4, 5 in parallel; then 6; then 7; then the
reserve. Total 9.5 % (8 % without the reserve). Preconditions: xp-give (22fead94) merged; the
F2-F4 slices that change the looks merged (v9 records at the end of the refactor).
