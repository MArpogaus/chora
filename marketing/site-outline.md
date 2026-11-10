# Website outline and copy

Draft of 2026-11-10. English. Plain words, no hype. Every claim maps to a
demo in the last table; a block whose demo is missing carries the tag
[WAIT] and is not published until the demo exists.

## Headline (options)

The decision: "a programmable 3D desktop you shape by hand or with an agent".
Sharper versions:

1. **A programmable 3D desktop. Shape it by hand, or let an agent do it.** (recommended)
2. A desktop you can program, walk through, and hand to an agent.

Why 1: it keeps the decided words, splits them into two short sentences, and
"let an agent do it" is something the user decides. Honesty check: "3D"
is true (the construct runs, see usps.md 1), but the default flavor today is
2D tiling, so the hero must show a 3D capture [WAIT D1] or the word "3D"
comes out of the headline. Sub-line:

> chora is a Wayland compositor where everything is a plugin: the window
> manager, the look, the effects. It runs in a window on the desktop you
> have now. Pre-release: it changes without notice and is not a daily
> driver yet.

Buttons: "Try it in 5 minutes" (anchor) and "Read the docs".

## Page structure

1. Hero
2. Two paths: Rice it | Build with it
3. Try it in 5 minutes
4. Gallery
5. How it works (short, four lines, link to docs/vision)
6. Pre-release note
7. Docs and community links

### 1. Hero

Copy as above. Media: one looping capture, 10 to 15 s, no sound, of the
construct with an XP window pulled out of the cube [WAIT D2]. Until D2
exists: a capture of the construct walk-through [D1] with the caption
"Recorded on <machine>, software rendering" (the demo states the machine, as
design/wobbly-3d.md says).

### 2. Two paths

**Rice it**
> Effects, a 3D cube of desktops, HUD folders and shaders. Everything you see
> is a plain file or a small plugin, so you can push it as far as you like and
> share it as a folder.
- Wobbly windows and the magic lamp in the XP look (shown today).
- The cube of four desktops in the XP look (shown today in the README text;
  needs a capture, D5).
- HUD folders: a folder of YAML and SVG that changes the look of the voice
  agent and the chat. A folder holds data only and cannot run code (shown in
  tests/gate-hud.sh; needs a capture, D6).
- Shaders as scene nodes (examples/loader; needs a capture, D7).
- 3D wobble and pulling a window out of the cube [WAIT D2].
Link: "Make your first HUD folder" (needs a how-to, T1).

**Build with it**
> An API that scripts and AI helpers use to arrange the screen, a 2D and 3D
> canvas to play on, and a codebase to study.
- Give an agent a grant and one MCP entry; it can see and change the screen
  (shown: tests/agent.sh, gate-0.8-*; needs a video, D3).
- The 2D/3D canvas and playground: examples/showcase has six small apps (snake,
  pong, text renderers, a shader card UI, a slide deck) and examples/loader
  flies a construct of views (shown; needs captures, D8).
- Research and design study: docs/vision.md, docs/design-principles.md,
  docs/scenarios.md, the design notes in the repo. Say what the study is:
  one scene model for 2D, 3D and agent control, with the test gates open to read.
- Voice and chat with helpers: [WAIT D4]; say "needs remote servers today".
Link: "Agent quick start" (needs a tool table, T2).

### 3. Try it in 5 minutes

Lead with the way that exists, list the others as "coming" with no dates.

> Inside a window, on the desktop you have now. Nothing to log out of.
> Ctrl+Alt+Escape resets the scene, Ctrl+Alt+BackSpace quits.

| Way | Status |
|-----|--------|
| `chora -- chora-tiling` after installing the RPM or Arch package from a release | works today (README.md) |
| `chora -- chora-xp` for the XP look | works today |
| Fedora COPR: `dnf copr enable ...` | planned, not built (D49; roadmap 1.0) |
| Arch package | package in each release; an AUR package is ruled out by D49, see open questions |
| distrobox / OCI image with windowed mode | not built [WAIT P3] |
| Nix flake: `nix run github:...` | not built [WAIT P4] |

Say each row's status in the table on the live page. Show what the user will
see after the command (screenshot), the keys, and the known caveat: GNOME keeps
Super, so set `modifier: Alt` (README.md).

### 4. Gallery

Grid of captures, each with a caption that names the flavor, the machine and
whether it was recorded or is a test image. No mock-ups. List in the media
table.

### 5. How it works (four lines)

- The core draws a scene of nodes and offers one API. It has no window
  manager, no layout and no look.
- The window manager, the look and the effects are plugins in their own
  processes.
- Scripts, plugins and agents use the same API, with grants the user gives.
- Config is plain text in git.
(Sources: docs/vision.md principles 1, 5, 7, 8.)

### 6. Pre-release note (visible above the fold link, repeated here)

> chora is pre-1.0. The API changes without notice. It has no long run on real
> hardware, no NVIDIA support, and it is not a daily driver. Parts of the
> code were written with AI coding tools (the README says so); the tests and
> the gates are in the repository, so you can check them. The voice agent
> needs remote speech and language servers today and its wake words are
> German. Do not use it for work you cannot lose.

Check the exact README wording on the LLM statement before copying it.

### 7. Docs and community

Docs link (https://marpogaus.github.io/chora/), GitHub issues, license. A chat
room does not exist yet (audience draft, gap 10); decide before launch.

## Media list mapped to demos

| ID | Asset | Format | Source demo / test | Status |
|----|-------|--------|-------------------|--------|
| D1 | Walk through the construct, panes of real programs | GIF 10 s and video 60 s | construct flavor, tests/gate-construct-*.sh | to record |
| D2 | Wobbly 3D: pull an XP window out of the cube | GIF 15 s, video 45 s | design/wobbly-3d.md E1 to E5 | not built |
| D3 | Agent rearranges a view from a real prompt | video 60 to 90 s | tests/agent.sh, choractl mcp | to record |
| D4 | Voice turn that rearranges a view | video 60 s | docs/voice.md | blocked (remote servers, German) |
| D5 | XP flavor: wobble, lamp, cube of four desktops | 3 GIFs | README "Try it" | to record |
| D6 | HUD folder swap, two or three looks, same behavior | GIF | docs/hud.md, gate-hud.sh | to record |
| D7 | Shader nodes live | GIF | examples/loader | to record |
| D8 | Showcase apps in the canvas | stills | examples/showcase | to record |
| D9 | Tiling flavor with waybar, fuzzel, mako (the safe screenshot for ricers) | still | flavors/tiling | to record |
| D10 | mc media-center look | stills | plugins/chora-mc, docs/mc-style.md | to record |
| D11 | Windowed mode: chora in a window on GNOME or KDE | still + GIF | README nested run | to record |
| D12 | Chat with helpers (ACP) | still | tests/chat.sh, docs/voice.md | to record |

Capture rules: state the machine and the renderer; recorded real, never
edited; keep the raw file; headless pixel tests (tests/battle) are fine for
stills but label them. No frame rate claim from software rendering.

Docs/tasks the page needs: T1 HUD how-to, T2 agent tool table and quick
start, T3 comparison page "chora vs niri, Hyprland, Wayfire, KWin, river"
with a "not yet" column (audience draft gap 4), T4 first-steps page.
