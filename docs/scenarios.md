# Scenarios

Three scenarios to check the concept against reality and estimate the effort
each one costs on top of the core. The estimates are rough and assume the core
implements the common Wayland protocols (see [Dependencies](dependencies.md)),
so existing tools work unchanged.

## 1. Classic desktop (Windows-like)

Taskbar, start menu, overlapping windows, mouse- and keyboard-driven. Not the
goal of Void, but it must be possible.

| Piece | Source | Effort |
|-------|--------|--------|
| Stacking window manager: move, resize, raise, focus, minimize | plugin | ~1–2k lines |
| Window decorations (title bar, buttons) | plugin drawing Rect/Image nodes, or client-side decorations | small–medium |
| Keybindings (Alt+Tab, Win key) | plugin | small |
| Taskbar, tray, clock | existing shell: Quickshell, waybar, eww | configuration |
| Start menu / launcher | fuzzel, wofi, or a Quickshell widget | configuration |
| Pixel-perfect XP look | theme assets + shader/decoration work | diligence, not difficulty |

Core requirements: `xdg-shell`, `xdg-decoration`, `layer-shell`,
`foreign-toplevel` (so the taskbar sees windows), `xdg-activation`.

**Verdict:** medium effort, almost entirely in one WM plugin.

## 2. Keyboard-driven tiling with workspaces

A Linux-style tiling workflow, configured by the user.

| Piece | Source | Effort |
|-------|--------|--------|
| Tiling layout (master/stack, columns, scrolling) | plugin | a few hundred lines |
| Workspaces | one group per workspace; switching = visibility/transform | small |
| Keybindings, focus movement | plugin | small |
| Bar | waybar / Quickshell | configuration |
| Animations | plugin animating group transforms per frame | small |

**Verdict:** low effort. A good first flavor to prove the plugin API.

## 3. Agentic workday (the core idea)

A robust voice channel to an agent that arranges content for each task.

1. **Morning canvas.** The user is greeted; current tasks are shown; email is
   pre-sorted; a plan for the day is drafted and prioritized together.
2. **Task workspaces.** For each task the agent designs a new canvas and fills
   it: for a data analysis, an area with prepared data, plots and interactive
   exploration; for writing email, research or coding, other specific views.
3. **Reuse.** Elements that prove useful are stored in the inventory
   (git-versioned) and reused; one-off code is discarded.
4. **Live change by voice.** "Make the plot bigger", "put the terminal next to
   the notebook", "dim everything except the editor".

| Piece | Source | Effort |
|-------|--------|--------|
| Scene access for the agent | core IPC + MCP bridge | core work (part of the API anyway) |
| Speech in / out | services: whisper.cpp / faster-whisper, piper | integration |
| The agent itself | Claude Code or any MCP client; later a local LLM service | integration |
| Rich content (briefing, plots, mail) | the agent generates HTML; a **webview service** shows it as a surface | small service + generated content |
| Data exploration | Jupyter / marimo / a plotting web app, shown as surfaces | existing tools |
| Views per task | YAML views, loaded/saved/modified live | core feature |
| Inventory, memory | opinionated layer above Void | later |

**Verdict:** the largest scenario, but mostly integration of existing pieces.
What the core must provide: scene query and modification, view load/save,
screenshots (so the agent can see the result), input injection, and
capabilities to keep this safe.

## What the scenarios tell us

- The five node kinds plus Wayland surfaces cover all three scenarios.
- Text and widgets do **not** need to live in the core; existing shells and a
  webview service cover them.
- The decisive core features are the **protocol coverage** (so existing tools
  just work) and the **API** (so plugins and agents can arrange everything).
