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

- **Target groups:** primary, beginners and older users who know the
  classic Windows look (Erika, 74; Murat, 52; mouse only, read slowly,
  click twice). Secondary, nostalgic users who notice every wrong pixel.
- **Philosophy:** familiar, friendly, mouse first; a faithful homage to
  the XP Luna feel without trademarked assets. Everything is visible and
  clickable; nothing needs a shortcut.
- **USPs:** an XP desktop that is pixel-true and also has a helper that
  can act for you; it runs as a face of the 3D cube.
- **RC requirements:** Start menu, taskbar, tray and window buttons work
  and match the style guide to the pixel; F1 opens a key and help sheet in
  XP style; the helper is reachable from the tray and the Start menu;
  Super opens Start, Super+D shows the desktop, Alt+Tab shows a switcher;
  the taskbar highlight follows focus and minimize; Turn Off shows its
  dialog; menu labels open their submenus.
- **Not goals:** Windows compatibility, themes other than Luna blue/olive
  /silver.

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

## chora-mc: the media centre

- **Target groups:** primary, living-room and hands-busy people (sofa,
  kitchen, the nurse) on voice and a remote (arrows, OK, Back).
  Secondary, beginners with no keyboard.
- **Philosophy:** calm, large, readable from three metres; voice first,
  no window management; one scene fills the screen, context lives in
  quiet margins.
- **USPs:** a voice-first scene desktop where the helper shows its work
  in a margin, not over your content; same session as the other looks.
- **RC requirements:** the first screen says what it is and shows how to
  act (remote keys and voice); every task reachable with arrows, OK and
  Back, and by mouse click; the helper and chat live in the right margin
  with the notifications (user, 11-11), never half the screen; a key list
  exists; terminals and apps open in a scene without config errors.
- **Not goals:** desktop multitasking, overlapping windows.

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
