# Scenarios

Three scenarios test the concept and estimate the effort on top of the core.
The estimates are rough. They assume that the core implements the common
Wayland protocols (see [Dependencies](dependencies.md)), so existing tools
work unchanged.

## 1. Classic desktop

A Windows-like desktop with taskbar, start menu and overlapping windows,
driven by mouse and keyboard. This is not the goal of Chora, but it must be
possible.

| Part | Source | Effort |
|------|--------|--------|
| Stacking window manager: move, resize, raise, focus, minimize | plugin | about 1000 to 2000 lines |
| Window decorations: title bar, buttons | plugin with rect and shader nodes, a decoration surface of the plugin, or client-side decorations | small to medium |
| Keybindings: Alt+Tab, Super key | plugin | small |
| Taskbar, tray, clock | existing shell: Quickshell, waybar, eww | configuration |
| Start menu, launcher | fuzzel, wofi or a Quickshell widget | configuration |
| Wallpaper | swaybg or swww | configuration |
| Pixel-perfect XP look | theme assets, shaders, decorations | much detail work, little difficulty |

The taskbar needs the foreign-toplevel protocols to see the windows. The
launcher and the bar need `wlr-layer-shell`.

**Result:** medium effort, almost all of it in one window manager plugin.

## 2. Keyboard-driven tiling

A Linux-style tiling workflow with workspaces. The user configures it.

| Part | Source | Effort |
|------|--------|--------|
| Tiling layout: master and stack, columns, scrolling | plugin | a few hundred lines |
| Workspaces | one group per workspace, switched by visibility or transform | small |
| Keybindings, focus movement | plugin | small |
| Bar | waybar or Quickshell | configuration |
| Animations | the `animate` command of the core | small |

**Result:** low effort. This is a good first flavor to test the plugin API.

## 3. Agentic workday

This scenario is the core idea. A robust voice channel connects the user to an
agent, and the agent arranges content for each task.

1. **Morning view.** The agent greets the user and shows the current tasks.
   It sorts the email. User and agent plan and prioritize the day together.
2. **Task views.** The agent builds a new canvas for each task and fills it.
   A data analysis gets prepared data, plots and interactive exploration.
   Email, web research and coding get their own views.
3. **Reuse.** The inventory keeps useful elements in git for later use. The
   agent discards code for one-time use.
4. **Live change by voice.** "Make the plot bigger." "Put the terminal next to
   the notebook." "Dim everything except the editor." The agent calls the
   methods of the window manager plugin for these changes.

| Part | Source | Effort |
|------|--------|--------|
| Scene access for the agent | IPC socket and `choractl mcp` | part of the API |
| Speech in and out | services: whisper.cpp or faster-whisper, piper | integration |
| The agent | Claude Code or another MCP client, later a local LLM service | integration |
| Rich content: greeting, plots, email | the agent writes HTML, a browser in app mode shows it as a surface | generated content |
| Data exploration | Jupyter, marimo or a plot web app as surfaces | existing tools |
| One view per task | YAML views that `choractl` loads, saves and changes live | coordinator tool |
| Inventory, memory | opinionated layer above Chora | later |

**Result:** the largest scenario, but mostly integration of existing parts.
The core must supply scene queries and changes, screenshots and input
injection. Input injection and screen capture need the approval of the user.
The agent needs screenshots to see the result of its changes.

## Conclusions

- The four node kinds cover all three scenarios.
- Text, images and widgets do not need to live in the core. Existing shells,
  wallpaper tools and a browser in app mode supply them.
- Two core features decide the success. The protocol coverage lets existing
  tools work. The API lets plugins and agents arrange everything.
