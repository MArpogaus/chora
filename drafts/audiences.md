# Audiences, competitors and launch channels

Draft of 2026-11-09. Research only. Nothing was posted.

Method. Web search and fetch, with the proxy of the session. The GitHub API
and Reddit were blocked, so star counts come from repository pages and
Reddit rules are mostly "no sources". A claim marked "single source" or
"not verified" needs a second look before any public use. The repo facts
come from README.md, docs/ and the tree at /home/user/chora.

## 1. Competitors and neighbours

Format per entry: what it is, license, activity, praise, what chora does
differently or not yet.

### niri

- What: a scrollable tiling Wayland compositor in Rust on Smithay.
- License: GPL-3.0. 29.0k stars on the repository page.
  Source: https://github.com/YaLTeR/niri
- Activity: version 26.04 in April 2026, per one blog (single source):
  https://ecosistemastartup.com/?p=77574
- Praise: LWN says it suits users who want tiling with the minimalism of a
  window manager. Source: https://lwn.net/Articles/1025866
  Other praise from users: no sources.
- chora: niri has a KDL config and `niri msg`, no plugins
  (docs/alternatives.md). chora has plugins and a scene API, but no daily
  driver status and no scrolling layout of its own in a shipped flavor.
  niri is the polish bar: chora is pre-release (README.md).

### Hyprland

- What: a dynamic tiling compositor with effects, C++.
- License: BSD-3-Clause. 40.6k stars. Source: https://github.com/hyprwm/Hyprland
- Activity: 0.55 in May 2026 with Lua config and user-defined layouts:
  https://phoronix.com/news/hyprland-0.55-wayland-comp
  0.56 in July 2026 is a single source:
  https://www.linuxcompatible.org/story/hyprland-056-ships-with-expanded-lua-scripting-motion-blur-and-zero-breaking-changes/
- Praise: looks and animations (the project describes itself as one that
  "doesn't sacrifice on its looks", same GitHub page). User quotes: no sources.
- Plugins: hyprpm installs plugins pinned to the Hyprland version, so a
  plugin is tied to one release. Source: https://codetest.hyprland.org/hyprwm/hyprland-plugins
- chora: plugins are separate processes over IPC, so a compositor update
  does not break the ABI (docs/alternatives.md, D28). Hyprland has the
  ecosystem and the ricing culture. chora has neither yet.
- Agents: community MCP servers for Hyprland exist, see section below.

### sway

- What: i3-compatible Wayland compositor on wlroots, C.
- License: MIT. Source: https://packages.fedoraproject.org/pkgs/sway/sway
  (listing) and https://alternativeto.net/news/2025/6/sway-1-11-released-with-wlroots-0-19-new-wayland-protocols-and-explicit-sync-support
- Activity: 1.11 in June 2025. Release 1.12 on 25 May 2026 is a single
  source: https://habr.com/ru/articles/1039438/
- Praise: i3 configs work as they are (same alternativeto page).
- chora: sway is the stable base people trust. chora tests with headless
  sway (docs/roadmap.md). chora offers a live scene API, sway offers
  `swaymsg` only. chora is not i3 compatible.

### river

- What: a non-monolithic compositor. The window manager is a separate
  process and speaks `river-window-management-v1`. Zig.
- License: GPL-3.0-only code, MIT protocols. Home is Codeberg, GitHub is a
  mirror. Source: https://github.com/riverwm/river
- Activity: 0.4 is out, 15 window managers exist per the news coverage.
  Source: https://linuxiac.com/river-0-4-wayland-compositor-debuts-pluggable-window-managers/
- Praise: the clean split. Closest design to chora.
- chora: same idea of an external window manager. chora adds a scene graph,
  shaders, cameras, input injection and an agent API. river has a stable
  protocol that other projects already use. chora's API is unstable
  before 1.0 (README.md).

### Wayfire

- What: a modular compositor with 3D effects, C++ on wlroots.
- License: MIT. About 3.1k stars. Source: https://github.com/WayfireWM/wayfire
- Activity: 0.10 in August 2025, then 0.11 with HDR and a
  `wayfire-plugin` command (date not found).
  Sources: https://linuxiac.com/wayfire-0-10-3d-wayland-compositor-brings-vulkan-support/
  and https://linuxiac.com/?p=216602
- Praise: the cube and effects. User quotes: no sources.
- chora: the XP flavor has a cube of four desktops, wobbly windows and a
  magic lamp (README.md). Wayfire plugins run in the compositor process.

### KDE Plasma

- What: a full desktop with KWin.
- Activity: Plasma 6.6 in February 2026, 6.7 in June 2026 with per-screen
  virtual desktops, 6.7.6 newest found.
  Sources: https://www.gamingonlinux.com/2026/04/kde-plasma-6-7-gets-per-screen-virtual-desktops-and-wayland-session-management/page=1/
  and https://linuxiac.com/?p=222370
- License: not checked here, no sources.
- Praise: configurability. No sources.
- Agents: kwin-mcp runs an isolated KWin session and reads the AT-SPI tree.
  Source: https://pypi.org/project/kwin-mcp/
- chora: no panels, no settings app, no daily use yet. chora has a
  compositor-native agent API instead of an add-on.

### GNOME

- What: a full desktop with Mutter.
- Activity: GNOME 50 removed the X11 backend in March 2026.
  Source: https://phoronix.com/news/gnome-50-alpha
- License: not checked here, no sources.
- Agents: an MCP server for GNOME uses a shell extension to get input
  injection and capture. Source: https://glama.ai/mcp/servers/ex7b1hnlel
  (listing, not read in depth).
- chora: runs nested inside GNOME for trials (README.md). GNOME is the
  default on Fedora Workstation 44 (same Phoronix page), so a nested start is
  the first contact for many Fedora users.

### COSMIC

- What: System76 desktop in Rust, cosmic-comp on Smithay.
- Activity: Epoch 1.9 on 23 September 2026 with an on-screen keyboard.
  Source: https://www.phoronix.com/news/COSMIC-Epoch-1.9-Released
  Epoch 1.8 reworked the compositor (same coverage, one aggregator:
  https://www.linuxcompatible.org/story/cosmic-epoch-180-tagged-system76-syncs-rust-desktop-stack-and-wayland-compositor-overhaul/).
- License: not checked here, no sources.
- chora: COSMIC is the complete Rust desktop. chora is a kit. They share
  Smithay (docs/alternatives.md).

### XR and spatial desktops

- Stardust XR: display server for 2D Wayland apps and 3D clients in a
  shared space, with a flatscreen mode. GPL-2.0, 188 stars on the server
  repo. The project says one developer led it for over five years.
  Sources: https://github.com/StardustXR/server and
  https://wiki.vronlinux.org/docs/fossvr/stardust/
  Release news for 2026: no sources.
- Simula: VR desktop on Godot. MIT, 3.2k stars. Hardware plan "Simula One".
  Sources: https://github.com/SimulaVR/Simula and
  https://roadtovr.com/simula-one-linux-vr-headset-kickstarter/
  Current state: no sources.
- WXRD: a Wayland compositor for XR use. Source:
  https://www.phoronix.com/news/Wayland-WXRD-Compositor (status not checked).
- chora: the construct flavor is a 3D space on a normal screen, with panes,
  walking and a camera (flavors/construct, plugins/chora-construct). It needs
  no headset. It is not an XR runtime. Say so, to avoid wrong hopes.

### Agent tools on other compositors

Several MCP servers drive existing desktops. They bolt on from outside.
- kwin-mcp (KDE): https://pypi.org/project/kwin-mcp/
- wayland-mcp (GNOME, KDE, Hyprland, sway):
  https://glama.ai/mcp/servers/@kurojs/wayland-mcp/tree/b6f20b260585cdf8e80e37395331de4bdf07447f
- computer-use-linux, a Rust MCP server, v0.4.6 in August 2026:
  https://pi.dev/packages/@agent-sh/computer-use-linux?name=mcp
- The security warning of wayland-mcp: it grants wide control of the desktop.
  chora's answer is the grant model: capabilities per client name,
  revoked by the emergency reset (docs/api.md, README.md).

## 2. Audience segments

Where they meet is partly from known community names. A channel with no
source here is marked.

### 2.1 Wayland tinkerers and ricers

- Needs: something new to configure, screenshots, a config in git.
- Ask: "Does it run my waybar? Can I get blur? Where are the dotfiles?"
- Meet: r/unixporn, Hyprland and niri chats. niri has a Matrix room
  (#niri:matrix.org, shown on the mirror
  https://upd.dev/niri-wm/niri).
- Has: tiling flavor with waybar, fuzzel, mako, kanshi (README.md);
  shaders and effects; YAML views and flavors as bundles
  (docs/vision.md principle 7); XP flavor as a showpiece.
- Missing: pictures, a config gallery, a theme how-to, a stable API.

### 2.2 Tiling window manager users

- Needs: reliable daily use, multi-monitor, IME, XWayland, no crashes.
- Ask: "Is it stable? Does it scroll like niri? Where is my layout?"
- Meet: niri, sway and river chats, r/wayland. Rules: no sources.
- Has: chora-tiling with awesome keymap, tags, layouts, XWayland through
  xwayland-satellite (README.md).
- Missing: a daily driver. README says "no daily driver yet". Only the
  author has run it as a login session, the 30 day gate is open
  (docs/roadmap.md).

### 2.3 Agent and LLM tool developers

- Needs: a safe way to let a model see and act on a desktop, a clean API,
  screenshots, a sandbox.
- Ask: "Which MCP tools? How do I limit it? Can I run it in a container?"
- Meet: MCP and agent communities, Hacker News, GitHub. Specific channels:
  no sources.
- Has: `choractl mcp`, grants such as `screen.capture`, `input.inject`,
  `activation.token`, client sockets for containers (docs/api.md,
  dist/quadlet/chora-agent@.container, marked UNTESTED in its header).
  A JSON schema of the protocol.
- Missing: a five minute agent demo, a list of MCP tools in one table,
  a tested container recipe, a comparison with kwin-mcp and similar tools.

### 2.4 Voice and accessibility users

- Needs: hands free control that works on Wayland.
- Evidence of the gap: Talon runs on X11 only and may drop public Linux
  builds, per OSNews, a news report not checked against Talon.
  Source: https://www.osnews.com/story/145162/accessibility-input-tool-removes-x11-support-doesnt-want-to-support-wayland-users-caught-in-the-middle/
  A user blog says Wayland lacks features such tools need:
  https://hn.jakechampion.name/reader/https://rykarn.se/2025/01/26/wayland
- Other Wayland tools: EasySpeak and Numen, maturity not verified:
  https://pypi.org/project/easyspeak-linux/
- Ask: "Can I control windows by voice? Offline? In my language?"
- Meet: Talon Slack and forum, accessibility lists. Sources: no sources.
- Has: chora-voice with VAD, wake words, cancel word, spoken confirmation,
  mute switches (docs/voice.md). Input injection through the API.
- Missing: it needs remote ASR, TTS and LLM servers (docs/voice.md names
  Qwen3 servers); the wake words are German ("Schlaf", "Hör zu"). No
  local, English, one command setup. No accessibility review by users who
  depend on it. Do not claim accessibility benefits until tested.

### 2.5 Teachers and education

- Needs: a controlled screen, slides, demos, easy reset.
- Ask: "Can I project slides and live demos? Can students break it?"
- Meet: no sources.
- Has: showcase apps incl. slides, pong, snake (examples/showcase); the
  emergency reset; the mc pages with a book style (docs/mc-style.md).
- Missing: classroom use case, install on school hardware, any teaching
  material. A weak segment now. Keep it a note, not a target.

### 2.6 Data visualization and exploration

- Needs: several plots and notebooks side by side, big canvas, live
  rearranging.
- Ask: "Can I put marimo or Jupyter in panes? Does it need a GPU?"
- Meet: no sources.
- Has: scenario 3 plan with Jupyter, marimo and web plots as surfaces
  (docs/scenarios.md); construct and mc pages with shaders
  (plugins/chora-mc/collectors, scenes).
- Missing: a worked data example with a picture. The scenario is a plan,
  not a demo.

### 2.7 secureblue and Fedora Atomic users

- What: secureblue is a security focused OS built on Fedora Atomic
  images with BlueBuild. About 1.1k stars, Apache-2.0, a Discord invite.
  Source: https://github.com/secureblue/secureblue
- Needs: layered packages that survive updates, small attack surface,
  signed or checkable artifacts.
- Ask: "Can I layer it? Is it in a repo? What does it need from the host?
  What does the session reach?"
- Meet: the secureblue Discord and repository. Matrix: not mentioned on
  that page.
- Has: RPM for Fedora 44, SHA256SUMS, `rpm-ostree install` steps
  (README.md); isolation by default and grants (docs/vision.md principle 8);
  the author's own target host is a derived secureblue image.
- Missing: a COPR repo (D49 says planned, README offers release assets
  only), signed packages, a note on hardened settings such as
  hardened_malloc and SELinux (the quadlet header says SELinux may need
  `SecurityLabelDisable=true`, which will worry this audience).
  uwsm is not in Fedora 44 (README.md).

### 2.8 Other segments found

- Compositor and Smithay developers. They read the scene design,
  docs/alternatives.md and the Pinnacle review. Meet: Smithay and Wayland
  developer channels, no sources. Has: the architecture docs and the
  protocol schema. Missing: a short "why not river or Pinnacle" page.
- Nostalgia and retro desktop fans, Windows XP look. Meet: r/unixporn
  style communities. Has: the XP flavor with theme assets (flavors/xp/theme).
  Missing: pictures. Caveat: the Windows XP look may touch trademark
  questions; not checked, no sources.
- Shader and creative coders. Has: GLSL nodes, examples/loader shaders.
  Missing: a shader tutorial with output images.

## 3. Channels for a launch

Facts from the pages named. Reddit rules were not readable by the tool.

| Channel | Rules and norms found | Source |
|---|---|---|
| Hacker News, Show HN | Must be something people can try. Title starts with "Show HN". No upvote asking. Minor updates do not count. "Don't post quickly-generated one-offs." Early or unpolished work is fine. Be in the thread to answer. | https://news.ycombinator.com/showhn.html |
| Lobsters | Invitation only. Self-promotion under a quarter of your stories and comments in a year. Accounts younger than 70 days cannot post self-promotion. Tags from a fixed list. A proposal for an "AI generated" flag exists (Feb 2026), outcome not verified. | https://lobste.rs/about and https://news.lavx.hu/article/why-lobsters-should-ban-llm-generated-submissions (aggregator, low trust) |
| r/unixporn | Mods changed rules in 2024. A karma minimum is mentioned by a user. Posts carry a bracket tag such as [Hyprland]. Dotfile links are common, a requirement is not confirmed. Current rule text: no sources. | https://lr.us.psf.lt/r/unixporn/comments/1fn6t2p/open_discussion_rules_and_quality_control |
| r/linux | A May 2026 meta proposal would ban personal projects under 3 months old, with a public git history as proof. Outcome unknown. Rule text: no sources. | https://reddit.sentinel-team.org/posts/1t20z8s/snapshots/2026-05-08T20%3A04%3A13.944224Z |
| r/wayland | no sources | |
| Phoronix | News tips go to news@phoronix.com. | https://Phoronix.com/contact |
| Fedora Discussion | Discourse forum. Spam is removed. The News and Announcements category is moderated. The Fedora AI policy applies. Self-promotion rule: no sources. | https://docs.fedoraproject.org/en-US/discussion-forum-guidelines-and-rules/ (summarized by search, page itself blocked) |
| Arch forums | no sources (the wiki page was blocked) | |
| Matrix and Discord | niri has Matrix. secureblue has Discord. Hyprland, sway, river rooms: no sources. Ask the moderators before posting. | see above |
| awesome-wayland | Lists about 20 compositors. The repository is archived since 17 October 2023, so new entries are not accepted. Find an active list or fork before planning on it. | https://github.com/natpen/awesome-wayland |
| FOSDEM 2027 | 30 and 31 January 2027, ULB Solbosch, Brussels. Devroom proposal deadline was 4 October, accepted rooms announced 21 October, devroom calls from 27 October, schedules by 7 December. Main track call announced 5 October, deadline not read. The accepted list and any desktop or Wayland room: no sources (page not read in full). Talks go to the call of a room, not to the organizer form. | https://fosdem.org/2027/news/call-for-devrooms/ and https://fosdem.org/2027/news/ |

Notes on timing and norms.
- LLM use is a live topic in 2026. HN rejects "quickly generated" projects.
  r/linux and Lobsters discuss limits. The README already states the use of
  LLM coding tools. Keep that statement visible and honest. Show the long
  history, tests and gates as proof of human review and of non-trivial work
  (tests/ has gate scripts, plans/ has reports).
- A Show HN needs something to try. Today that is a nested run
  (`chora -- chora-tiling`) or a package. A pre-release label is fine for HN.
- Wait for the 30 day daily use gate before r/linux or Phoronix. A crash
  in public costs more than a late launch. This is a judgment, not sourced.
- Posting dates and hours that work best: no sources. Do not invent them.
- Self-promotion is limited nearly everywhere. Plan one post per channel,
  then answer questions, then wait.

## 4. Gaps that block adoption

From the tree and docs of 2026-11-09.

1. Pictures. README.md has no image or GIF. The repo has only theme
   assets and two PNGs in tests/battle. An ricing or compositor audience
   decides in seconds from a picture. Needed: XP flavor, construct, mc page,
   tiling with waybar, a voice HUD.
2. Demo video. None. A 60 to 90 second clip of a voice turn that rearranges
   the screen would carry the main idea. It must show a real turn.
3. Packages. README offers a release RPM for Fedora 44 and an Arch package
   from the release page. D49 says COPR with Packit and no AUR
   (docs/decisions.md). The roadmap lists COPR in the 1.0 milestone
   (docs/roadmap.md). Many Arch users expect an AUR package. A decision to
   revisit, or say plainly why not.
4. Comparison page. docs/alternatives.md compares for builders and is
   written as design notes with "unverified" marks. Users need a short
   page: "chora vs niri, Hyprland, river, sway". It should state not yet.
5. First steps. README has "Try it" with a keymap. Missing: what you see,
   how to quit nested (the safety keys are listed), how to get the XP
   flavor with images, what to do when the host keeps Super.
6. Voice setup is hard. It needs servers that a new user does not have
   (docs/voice.md). No sample config for a local stack.
7. Agent quick start. One command grants and one MCP entry exist. Missing:
   a table of tools, an example prompt, a screenshot of the result.
8. Stability signals. README says the API changes without notice before
   1.0. The container recipe is untested (dist/quadlet header). No long run
   on hardware yet (docs/roadmap.md).
9. Docs site language. The docs are in plain English, but many pages are
   design records (D numbers) that outsiders cannot read.
10. No community channel is named in README (no Matrix room, no chat).
    Issues on GitHub only. Add one before launch.

## 5. Candidate USPs (at most eight)

Each ties to a file and a segment. The final five come from the simulated
user tests.

1. Agent API in the compositor with grants. `choractl mcp` and per-client
   capabilities, revoked by the emergency reset.
   Files: docs/api.md, README.md. Segments: agent developers, secureblue.
2. Policy-free core, window manager as a plugin process, so a compositor
   update does not break plugins. Files: docs/vision.md, docs/alternatives.md
   (D28). Segments: tinkerers, compositor developers.
3. One scene, several desktops: tiling, XP stacking with cube, and 3D
   construct from the same core. Files: flavors/tiling, flavors/xp,
   flavors/construct, README.md. Segments: ricers, nostalgia fans.
4. Voice agent that arranges the screen and asks for spoken confirmation.
   Files: docs/voice.md, services/voice, CHANGELOG.md (voice confirm,
   cancel word). Segments: voice users, agent developers.
5. Shaders as scene nodes, with a live API. Files: examples/loader,
   plugins/chora-construct/*.frag, dist/shaders. Segments: ricers, creative
   coders, data visualization.
6. Declarative views and flavors in git, readable by a human and an agent.
   Files: docs/configuration.md, docs/vision.md principle 7.
   Segments: tinkerers, agent developers.
7. Fedora Atomic and secureblue first install path with checksums and
   rpm-ostree steps. Files: README.md, dist/build-rpm.sh, dist/arch.
   Segment: secureblue and Atomic users.
8. Container isolated agent over a client socket, "the socket is the
   identity". Files: dist/quadlet/chora-agent@.container (untested),
   docs/api.md. Segments: agent developers, secureblue. Needs a test run
   before it is a claim.

## 6. Open questions for the owner

- Should the launch wait for the 30 day gate and COPR?
- Is a German first voice setup acceptable, or does English come first?
- Which picture set carries the first post: XP, construct or tiling?
- A chat room: Matrix, Discord, or none yet?

## Unverified or blocked items, summary

GitHub API, Reddit, Arch wiki and Fedora docs pages were blocked. Licenses of
KDE, GNOME and COSMIC were not fetched. Star counts are from the page
snapshots of 2026-11-09 and move. Several release dates rest on one
aggregator and are marked.
