# Unique selling points

Draft of 2026-11-10. Status: nothing here is published. Facts about chora come
from /home/user/chora (README.md, docs/, tests/) at this date. Competitor facts
come from the sources named; "audience draft" means drafts/audiences.md in this
branch, which holds the URLs it names. A competitor row says what the source
shows, not what the competitor can never do.

Rule for every USP: the site and posts may claim only the "shown" part. The
"not shown yet" part waits for its demo.

Legend: SHOWN = a test, doc or recording exists and can be run today.
NOT SHOWN = no demo yet; the page may only say "planned".

## 1. A 3D space with real windows in it (the construct)

- Sentence: You can fly and walk through a 3D scene on a normal screen, and
  your real programs run as panes in it.
- Proof, SHOWN: the construct flavor (flavors/construct, plugins/chora-construct);
  tests/gate-construct-k1.sh, gate-construct-windows.sh, gate-construct-keys.sh,
  gate-construct-nested.sh (a nested chora with tiling or xp becomes a pane and
  takes typed text); tests/gate-handover-matrix.sh (12 window-manager pairs).
  The README says the XP flavor has a cube of four desktops.
- NOT SHOWN: there is no picture or video of the construct in the repo
  (audience draft, gap 1). Needs demo D1 (walk-through capture, see
  site-outline.md). The 3D cube with XP on its faces needs the steps E4 of
  design/wobbly-3d.md (virtual outputs are "the largest unknown").
- For: ricers who want something that is not another tiling setup; people who
  study spatial interfaces.
- Who lacks it: Wayfire has a 3D cube and effects, but its plugins run in the
  compositor and the cube switches desktops (https://github.com/WayfireWM/wayfire,
  https://linuxiac.com/wayfire-0-10-3d-wayland-compositor-brings-vulkan-support/).
  KWin in Plasma 6 has a cube again, as a desktop switch
  (https://bugs.kde.org/show_bug.cgi?id=493739). Stardust XR is the closest
  idea (2D Wayland apps in a 3D space) but is aimed at XR
  (https://github.com/StardustXR/server). History: Project Looking Glass was
  an earlier 3D desktop; Sun stopped it after 1.0.1 in January 2007
  (https://www.wikipedia.org/wiki/LG3D; see also
  https://www.osnews.com/story/30091/suns-project-looking-glass-debuted-14-years-ago/).
  The lesson from that history: one commenter said 3D effects look spectacular
  but do not improve the workflow, so the demos must show something useful too.

## 2. Effects that every window manager can use, also in 3D

- Sentence: Wobbly windows, the lamp and shader effects come from one shared
  library, so tiling, XP, mc and the 3D space can all use them.
- Proof, partly SHOWN: `chora_kit::wobbly` (KWin spring model) and
  `chora_kit::lamp` exist; the XP flavor wobbles windows and minimizes through
  a magic lamp (README.md "Try it"). Shaders are scene nodes (examples/loader,
  dist/shaders).
- NOT SHOWN: the shared driver `chora_kit::effects` for the other window
  managers (steps E1, E2), the 3D wobble, and the pull of a window out of the
  cube (E3, E5) are planned in design/wobbly-3d.md. Needs demo D2: the wobbly
  3D showcase. Until it exists, say only "wobbly windows in the XP look".
- For: ricers, shader and creative coders.
- Who lacks it: KWin has wobbly windows, off by default
  (https://bugs.kde.org/show_bug.cgi?id=498760 and the KDE developer remark
  quoted in the search of 2026-11-10); Hyprland sells animations and custom
  bezier curves (https://github.com/hyprwm/Hyprland; archived site text
  https://code.hyprland.org/hyprwm/hyprland-website/raw/commit/2a39940c363cd83fb6d61c41bbf946164b0f4706/views/home.ejs).
  Both tie effects to their own compositor and window model. I found no
  source showing a compositor that offers one effects library to window
  managers written as separate plugins; absence of a source is not proof, so
  the site must not say "only chora".

## 3. An agent that changes the screen, with grants you control

- Sentence: An AI helper can see the screen and rearrange it through one API,
  but only with the rights you gave it, and one key combination takes them back.
- Proof, SHOWN: `choractl mcp`; grants `screen.capture`, `input.inject`,
  `activation.token` (README.md, docs/api.md); tests/agent.sh,
  tests/gate-0.8-*.sh (MCP focus and move through chora-tiling, clicks under a
  camera hit 20 of 20, injected text, the revoke binding, the indicator while
  input is injected). The container recipe (dist/quadlet) is marked UNTESTED
  in its header, so do not claim container isolation as proven.
- NOT SHOWN: a five-minute agent video of a real turn that rearranges a
  view (audience draft, gap 2); a table of MCP tools. Needs demo D3.
- For: agent and tool developers; people who want to try "agent as window
  manager" safely.
- Who lacks it: other compositors get agents from outside: kwin-mcp for KDE
  (https://pypi.org/project/kwin-mcp/), wayland-mcp for GNOME, KDE, Hyprland and
  sway, whose own page warns that it grants wide control of the desktop
  (https://glama.ai/mcp/servers/@kurojs/wayland-mcp/tree/b6f20b260585cdf8e80e37395331de4bdf07447f),
  computer-use-linux (https://pi.dev/packages/@agent-sh/computer-use-linux?name=mcp).
  I did not read all three in depth; say "add-on" not "unsafe".

## 4. Talk to it, or chat with helpers, and dress them in a folder

- Sentence: A voice agent and a chat field with AI helpers are part of the
  desktop, and a HUD folder of plain files changes how they look.
- Proof, partly SHOWN: tests/chat.sh (chat field with a fake ACP agent: typed,
  composed and pasted text, spoken yes cannot answer a question that waits);
  tests/gate-hud.sh and docs/hud.md (HUD folders: data only, size limits, no
  code); docs/voice.md (VAD, wake words, cancel word, spoken confirmation).
  Design principle 5 and 10: a shared folder can never answer or hide a
  question.
- NOT SHOWN: a real voice turn on video. The voice service needs remote ASR,
  TTS and LLM servers and the wake words are German (audience draft, section
  2.4). Needs demo D4 and an English local setup before "voice" goes in a
  headline. The post to ricers should lead with HUD folders, not voice.
- For: ricers (HUD folders are a shareable look), voice users (not until
  tested by them), agent developers.
- Who lacks it: no source found for a compositor that ships a voice agent and
  a user-shareable HUD format. Not verified; do not write "first".

## 5. Try it inside a window, and every look is a plugin

- Sentence: chora runs as a window on your current desktop, so you can try it
  in minutes, and each look (tiling, XP, mc, construct) is a separate plugin
  process, so one crash or one update does not break the rest.
- Proof, partly SHOWN: `chora -- chora-tiling` runs nested (README.md);
  docs/vision.md principles 1 and 8; tests/gate-handover-matrix.sh moves a
  window between window managers; RPM for Fedora 44 and an Arch package are in
  each release; the safety keys (Ctrl+Alt+Escape resets, Ctrl+Alt+BackSpace quits).
- NOT SHOWN: the packaging the user chose. COPR is planned for 1.0 (docs/roadmap.md,
  D49); no Nix flake and no distrobox/OCI image exist in the tree (checked
  2026-11-10: no flake.nix, dist/ holds arch, session, quadlet, plugins,
  portal, shaders, fonts). Needs demos/tasks P1 (COPR), P2 (Arch), P3 (OCI
  image with windowed mode), P4 (flake). No long run on real hardware yet.
- For: everyone, first of all anyone who will not reinstall a desktop to look.
- Who lacks it: Hyprland plugins are installed with hyprpm and tied to one
  Hyprland version (https://codetest.hyprland.org/hyprwm/hyprland-plugins);
  Wayfire plugins run in the compositor process (audience draft). river also
  splits the window manager into a process
  (https://github.com/riverwm/river), so "plugin as a process" is not unique;
  what chora adds is the scene API, shaders and agent grants on top. Say so.

## Summary table

| # | USP | Shown today | Blocked by |
|---|-----|-------------|-----------|
| 1 | 3D space with real windows | tests only, no picture | D1 capture; cube needs E4 |
| 2 | One effects library for all window managers | XP wobble and lamp | E1, E2, E3, E5 |
| 3 | Agent with grants | tests and one MCP entry | D3 video, tool table |
| 4 | Voice, chat, HUD folders | chat and HUD tests | D4, English local voice setup |
| 5 | Windowed try-out, plugin looks | nested run, RPM, Arch pkg | OCI image, flake, COPR |

Ranking for r/unixporn: 1 and 2 lead, 5 gives the way to try, 4 (HUD folders)
is the ricing hook, 3 is the "Build with it" path.
