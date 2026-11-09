# Design principles

This page holds the philosophy, the core values and the principles that
every user-facing part of chora follows: the window managers, the HUDs,
the bars and launchers, the voice agent, the chat, the prompts, the
error messages and the docs a user reads. [Vision and
principles](vision.md) holds the technical principles of the core. The
style guides of [chora-mc](mc-style.md) and
[chora-construct](construct-style.md) hold the look of one window manager;
they apply these principles and never contradict them.

## Philosophy

chora gives forms a place and stays out of the way. The interface
serves the task of the person in front of it and recedes when it is not
needed. Content comes first; the frame around it is calm and quiet.

A look can be anything: a classic desktop, a tiling setup, a media
center, a 3D space. The behavior under the look stays the same, so a
person who learned one look is not lost in another. People of every
skill use chora: a person who has never used a computer and a person who
rebuilds the whole desktop for fun. The simple path comes first, and
power sits on top of it, never in its way.

The person decides. Agents, plugins and scripts act for the person, show
what they do, and ask before they do something that matters.

## Core values

1. **Least surprise (POLS).** Things behave as a person expects from
   what they see and from the rest of chora.
2. **Simplicity (KISS).** The simplest thing that solves the task. No
   feature without a person and a task it serves.
3. **Quality and stability before features.** A flaw in a flow that
   exists ranks above a new flow.
4. **Everyone.** Low-tech people, people who use only a keyboard or only
   their voice, and power users all reach their goal.
5. **The person in control.** Nothing acts in a person's name without
   their knowledge; every step can be undone or left.
6. **Honesty.** The interface shows the true state, names failures, and
   never pretends.
7. **Open and ownable.** Everything a person sees can be changed and
   shared as plain files.

## Principles

Each principle has a check. A review round with the UX/UI view, the
persona tests and the tests named below apply them.

### 1. The same act does the same thing everywhere

Moving, closing, focusing, backing out and confirming work the same way
in every window manager and every HUD. Escape backs out of the current
step. A Super drag moves a window. A window moves between all window
managers by every input path.

*Check:* `tests/gate-handover-matrix.sh`; the UX/UI view compares new
flows with the existing ones.

### 2. Every task by every path

Each task works with the mouse only, with the keyboard only and with the
voice only, and through the API for scripts and agents. No path ends in
a dead end.

*Check:* each UX design note lists the flow for each path; persona tests
cover mouse-only, keyboard-only and voice-only personas.

### 3. A visible way in

A first-time person finds every basic task on screen without knowing a
key or a command: starting a program, finding a window, opening the
chat, changing the volume, connecting to a network. Keys and voice
commands are shortcuts, never the only door.

*Check:* the low-tech persona tasks start from a fresh session with no
instructions.

### 4. Say what happens, in the person's words

Every element shows its state: idle, busy, done, failed, refused,
offline, waiting for an answer. A failure names the thing that failed
and the next step a person can take. Nothing fails in silence. The words
are plain, with no internal terms: "helper", not "agent"; "Connected",
not an address.

*Check:* each UX design note has a table of states; persona findings
of the kind `ux`.

### 5. The person decides

Anything that acts in a person's name asks before it acts on something
that matters, and the question shows the whole of what will happen. A
question that guards a risky act takes a physical answer: a key or a
click. Text that a third party supplies, such as a HUD folder or an
agent's message, can never draw or fake a question. A person can stop an
agent and undo or leave every step.

*Check:* the confirmation tests of the voice service; the security view
of review rounds.

### 6. Calm

Few text sizes, one accent color per look, one family of shapes. Motion
shows a change and is short. Nothing asks for attention, with a sound,
a popup or a blink, unless the person asked for it or safety needs it.
A question stays in place while it waits for an answer.

*Check:* the style guide of each look; the UX/UI view.

### 7. Readable for everyone

Text a person must read is at least 14 px. Text and its background have
a contrast of at least 4.5:1, and large text and controls at least 3:1.
Click targets are at least 24 by 24 px. Words come from one table per
language; English and German exist, and German uses the formal
address.

*Check:* the contrast tests of the style guides; persona captures for
reading tasks at a size where the text is legible.

### 8. Looks differ, behavior does not

Each look has its own style guide for type, color, space and motion.
Behavior, words, keys and questions come from shared parts in
`chora-kit` and the HUD host, so a look changes how things appear, not
what they do.

*Check:* the UX/UI view; a new look gets a style guide before its first
merge.

### 9. Simple first, power on top

The defaults serve a person with no experience. Power users get keys,
settings, plain files and HUD folders they can ship. A power feature
never makes the simple path harder.

*Check:* each persona round has low-tech personas and the
push-to-the-limit persona; a finding of either blocks a merge in the
same way.

### 10. Open and ownable, never unsafe

Themes, HUDs and settings are plain files a person can read, change and
share. A file that someone shares is untrusted: it can change how things
look and what they show, never what runs. Code comes only as a plugin
with its own grant.

*Check:* the HUD folder limits of `chora-kit`; the security view.

## How a change follows these principles

A change that a person sees, hears or types gets a UX design note before
programming starts. The note gives the persons and their goal, the flow
for each path, the states, the layout and the words, and checks with
expected answers. The review round of the change adds the UX/UI view,
which compares the result with the note and with this page.
