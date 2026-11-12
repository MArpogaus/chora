# Product briefs per plugin (PM, 11-11)

Asked by the user on 11-11: "Get you head around what each plugin shall
deliver. Define clear requirements USPS and target groups for each."
This file is the product contract for every design pass, build lane and
review until the RC. The look documents (`design/looks/LOOK.md` and
`LOOK-v2.md`) design against it; a conflict is raised to the PM, not
worked around. Requirements marked **RC** block the release candidate.

## chora as a whole

- **Promise:** one programmable scene that can host any desktop; the
  looks show that range, each done properly, and a window moves between
  them without loss.
- **USPs (what no other desktop shows):** (1) four complete desktops in
  one session, switched live, windows moving freely between them by key,
  drag, command or voice; (2) a 3D construct whose cube faces are live
  desktops; (3) a helper and agent woven into every look through one API,
  with confirmation for risky acts; (4) everything is plain text and a
  plugin, so a user or an agent can change any part live.
- **Shared RC requirements:** (a) a first-time user finds, in every look,
  a visible way to start a program, the key list, and the helper chat,
  with mouse and with keyboard; (b) no key the look does not use leaks
  into a client as stray text; (c) lock, power off, volume, screenshot and
  clock are reachable in every look (external tools allowed); (d) every
  failure has an honest visible state, never a black or silent screen;
  (e) no lost window across any handover (the matrix stays green).

## chora-xp: the familiar desktop

- **Goal (user, 11-12 04:20 UTC):** "pixel perfect replication and ux
  similar yo original". Reference: Windows XP SP2/SP3, Luna Blue, 96 DPI,
  1024x768 and 1280x800, Normal font size. Metrics, colours, gradients,
  spacing, positions and behaviour match the reference to the pixel; the
  look document lists each reference value.
- **Assets (user 11-12 04:25 UTC):** chora ships no Microsoft art or
  names (no flag logo, Bliss, Clippit, Windows sounds, Tahoma from a
  Windows copy). An optional, user-run import copies the icons, XP sounds
  and wallpapers from the XFCE total-conversion repository into place;
  with them in place xp is pixel perfect in look and sound. Without them
  xp falls back to our own icons on XP's grid (16, 24, 32, 48 px), Wine
  Tahoma and our own wallpaper, and stays complete. Every size, position
  and behaviour is built for the imported assets; audit cells run both
  ways.
- **Target groups:** primary, people who know XP (Erika, 74; Murat, 52;
  mouse only). Secondary, nostalgic users who notice every wrong pixel.
- **Accessibility:** XP's own way: Normal (96 DPI, 8 pt Tahoma = 11 px)
  is the replica; "Large size" (120 DPI, as XP's Display settings had)
  is the option for older eyes. This replaces the 14 px floor (D1) for xp.
- **UX like the original:** Start menu two columns with user tile, All
  Programs cascade, Run, Help and Support, Turn Off and Log Off dialogs;
  taskbar grouping, Quick Launch, tray with balloon tips, clock, volume;
  double-click to open, right-click menus, Alt+Tab switcher, Super opens
  Start, Super+D desktop, Super+E file manager, F1 help. The helper lives
  in the tray and the Start menu, as an XP-era assistant would.
- **Done means:** a side-by-side capture diff against reference
  measurements per element (table in the look document) shows no
  deviation above 1 px or one colour step; every audit cell green.
- **Not goals:** running Windows programs; themes other than Luna
  blue/olive/silver; Microsoft art.

## chora-tiling: the hacker's desktop

- **Target groups:** primary, keyboard-first power users and ricers
  (r/unixporn, Hyprland and sway users). Secondary, beginners who landed
  here must not be stranded.
- **Philosophy:** fast, precise, beautiful in motion; Hyprland-class
  visuals (gaps, rounded corners, focus glow, smooth open/move/close
  animations) on a config that stays plain text and easy to extend.
- **USPs:** stunning tiling with live config reload, plus what Hyprland
  lacks: windows that move to xp, mc or the 3D construct, and a built-in
  helper.
- **RC requirements:** visible tag/workspace indicator and focus mark;
  Super+? (or F1) key overlay; launcher reachable by key and by clicking
  the bar; lock, power and screenshot keys and bar entries; no unbound
  Super chord reaches a client; animations off with one setting; every
  visual parameter in one documented config file.
- **Not goals:** a settings GUI; floating-first layouts.

## chora-mc: voice-first computing

- **Goal (user, 11-12 04:20 UTC):** "a modern voice first computing
  experience where content shapes the view, following traditional
  typesetting rules". The user-facing name "media centre" no longer fits;
  code name stays `mc` until a name is chosen (week 2).
- **Voice first:** every task starts and completes by voice; the screen
  shows what was heard, what is being done and what needs a yes. Remote
  keys (arrows, OK, Back) and mouse stay a full second path (RC), never
  the only one.
- **Content shapes the view:** no fixed chrome or window frames; each
  content kind has its own page form: agenda as a list, a document as a
  text page, a terminal as a monospaced page, media full-bleed, chat and
  notifications in the margin. The page reflows to its content; nothing is
  boxed that is not a thing.
- **Typesetting rules (measurable):** one baseline grid (the column
  pitch) that every line, rule and bar sits on; measure 45 to 75
  characters; a modular type scale with at most four sizes; one text face
  plus one mono; rules and bars span cap top to baseline of their text;
  no widows or orphans in headings and lists; ragged right with
  hyphenation (EN, DE); hanging punctuation; margins in fixed proportion
  to the measure; small caps for labels; contrast for three metres.
- **Target groups:** hands-busy people (kitchen, sofa, the nurse),
  accessibility users, people who prefer talking to clicking.
- **RC requirements:** the first screen says what it is and how to talk
  to it and how to use the remote; every task by voice, by arrows/OK/Back
  and by click; helper, chat and notifications in the right margin (user
  11-11); a key list; programs open as pages without config errors; every
  audit cell (baseline grid, bar height, measure, overlap) green.
- **Not goals:** overlapping windows, desktop multitasking.

## chora-construct: the 3D space

- **Target groups:** primary, explorers, demo watchers and ricers (Max,
  Kai, Ben 13) who want to show something no other desktop does.
  Secondary, xp users working on a cube face at 1:1.
- **Philosophy:** spatial, glass, calm motion; delight without losing a
  window or a key; the HUD stays minimal (no buttons, F1 overlay).
- **USPs:** live desktops as cube faces, step out and in, pull a window
  out of a face, drag across faces.
- **RC requirements:** the empty floor shows how to start a window (key
  and click); keys of the construct never type into a focused client
  unless the client owns them; every pane has a label; the cube turns
  while dragging and each face is recognisable; a manager that fails
  shows an error on its face within 2 s; F1 is legible (opaque enough).
- **Not goals:** VR headsets, physics.

## chora-assistant and the chat: the helper

- **Target groups:** everyone, through each look's idiom.
- **Philosophy:** one helper, many costumes: the figure, card and chat
  follow the look they live in; the question card and its keys are the
  same everywhere (shared invariants).
- **USPs:** an agent that can act on the desktop, always asks before
  risky acts, and shows its state honestly.
- **RC requirements:** a visible entry in every look; the sent message
  shows in the transcript; Stop ends "working"; the question card works by
  click and by keys; no-helper state is shown when the agent is absent.

## chora-notify

- **Target groups:** everyone. **Philosophy:** quiet, never covers the
  work. **RC requirements:** cards placed per look (mc: right margin;
  tiling: corner; xp: tray balloon), dismiss by click and timeout.

## services/voice

- **Target groups:** hands-busy people, accessibility users.
- **Philosophy:** short spoken answers, quick first sound, confirm risky
  acts by voice. Local, interchangeable models with configurable
  endpoints (user, 11-11).
- **RC requirements:** an acknowledgement before tool rounds; every
  `run_program` confirmed; works without a GPU through a configured
  remote endpoint, and says so when no model is reachable.

## How the PM briefs workers (standing template)

Every worker brief names, in this order: (1) the goal in one sentence
and the plugin brief section it serves; (2) the design note and the exact
build list items; (3) the files and areas it may touch (one area per
worker); (4) the acceptance checks, each runnable, with a failing test or
capture first; (5) the gates it must pass (`merge-gate.sh` steps of its
area, the 2D guard if the renderer changes); (6) what it must not do (no
other areas, no history rewrite, no `git add -A`, no attribution, no
inline comments); (7) its report: at most 15 lines, commits, test
numbers, captures, open risks. A builder never reviews its own pixels; a
separate reviewer checks captures at 1x and 2x against the look document.

## Spec gaps found 11-12 04:22 UTC and the PM's defaults (user may veto)

1. xp "pixel perfect" vs the 14 px floor (D1) and the 74-year-old
   target: contradictory. Default: replica at 96 DPI, XP's "Large size"
   as the accessibility option (above).
2. xp "replication" vs trademarks: settled by the user 11-12 04:25 UTC
   (optional asset import, above). Gap: the import script is not in the
   chora repo and the source repository is not named; week-2 lane;
   the line itself goes to Needs you (licensing, rules.md 11).
3. xp reference: no reference captures exist in the repo and we may not
   ship them. Default: a measurement table (element, size, colour,
   position) in the look document, sourced from public documentation,
   checked by audit cells.
4. mc identity: "media centre" in the name and briefs vs "voice-first
   computing". Default: the brief above; rename in week 2.
5. mc "content shapes the view": which content kinds exist is unlisted.
   Default: agenda, document, terminal, media, chat; others week 2.
6. mc voice first vs the remote RC: kept both; voice is primary.
7. tiling, construct, helper: no change asked; their briefs stand.
8. "polished to perfection" has no end: defined as a full pixel review
   that finds nothing above COSMETIC (rules.md 8).
