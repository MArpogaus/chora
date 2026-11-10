# Launch plan

Draft of 2026-11-10. Nothing posted. Source limits: Reddit rules pages could
not be read; the r/unixporn facts below come from mirrors of two mod posts of
2024 and are marked. Check the live sidebar and wiki before posting. I give no
best posting hours; no source, so none invented.

## Gates before anything goes public

1. Hero capture D2 or, if it slips, D1 with an honest caption (site-outline.md).
2. A way to try it that works for a stranger: windowed mode from a release
   package today; OCI image and flake when built (P3, P4).
3. README with at least three images, the pre-release note, the safety keys.
4. A named place for questions (issues at minimum; chat room undecided).
5. The 30-day daily-use gate (docs/roadmap.md 0.9) is open. Judgment, not
   sourced: do not go to r/linux, Phoronix or HN before the author has run it
   on real hardware for that time. r/unixporn needs a picture, not a daily driver.

## Order of channels

| # | Channel | When | What | Rules and norms (source) |
|---|---------|------|------|--------------------------|
| 1 | Close testers (Fedora/secureblue, a few ricers you know) | week -2 | try the windowed run, break it | no rules |
| 2 | r/unixporn | launch day | one image post plus dotfiles link | see below |
| 3 | GitHub release and the docs site | same day | link target of every post | none |
| 4 | Hacker News "Show HN" | after the first week of fixes | "Show HN: chora, a programmable 3D desktop (pre-release)" | must be something people can try, no vote asking, be in the thread, early work is fine (https://news.ycombinator.com/showhn.html) |
| 5 | Agent channels | with or after 4 | the "Build with it" video D3 | below |
| 6 | Lobsters, r/wayland, Fedora Discussion, Arch forum | later, one post each | only if invited / allowed | Lobsters is invite only and limits self-promotion to under a quarter of activity (https://lobste.rs/about); r/wayland, Arch rules: no sources, read first; Fedora forum guidelines: https://docs.fedoraproject.org/en-US/discussion-forum-guidelines-and-rules/ |
| 7 | Phoronix tip | after real-hardware run | news@phoronix.com (https://Phoronix.com/contact) | no rules found |
| 8 | r/linux | later, if at all | a May 2026 proposal would ban personal projects under 3 months old (https://reddit.sentinel-team.org/posts/1t20z8s/snapshots/2026-05-08T20%3A04%3A13.944224Z; outcome unknown) | check |

Agent channels (single third-party checklists, not rules): the official MCP
Registry, a pull request to punkpeye/awesome-mcp-servers (read its
CONTRIBUTING.md first), r/mcp and r/LocalLLaMA; sources
https://dev.to/thenomadevel/7-mcp-sites-every-ai-dev-should-bookmark-2gno and
https://github.blog/changelog/2025-09-16-github-mcp-registry-the-fastest-way-to-discover-ai-tools/.
chora is a compositor with an MCP entry, not an MCP server, so a registry
listing may not fit; check first. I found no official rules for r/LocalLLaMA.

## r/unixporn post plan

What is known (mirrors, may be stale):
- Mod posts of September and October 2024 on quality control and rules
  (https://lr.us.psf.lt/r/unixporn/comments/1fn6t2p/open_discussion_rules_and_quality_control,
  https://redlib.hbubli.cc/r/unixporn/comments/1g701zr/update_on_rules_and_quality_control).
  They talk about cleaning up low-effort posts, a proposal to require dotfiles
  or at least the applications used, and a warning against reposts with only
  small changes. Whether the dotfile rule applies: not confirmed.
- A user reports a karma minimum; threshold unknown.
- Posts carry a bracket tag such as [Hyprland] (audience draft).
- No self-promotion rule was found. Treat the post as a rice share, not an ad.

Plan:
1. Before: check the sidebar and wiki for the current rules, the flair list
   and the karma minimum. If the account is too new, use a long-standing
   account of a person, not a fresh one. Never ask for upvotes.
2. Title: tag in the form the rules ask, then plain words, for example
   `[chora] XP window pulled out of a 3D cube, wobbling (pre-release, runs in a window)`.
   Use the tag only if the sub's rules allow a new compositor name; else
   use the closest allowed flair and say what it is in the first line.
3. Content: the D2 capture (or D1 plus D5 stills), recorded real, no edits.
   One image or short video, then a gallery of two to four stills.
4. First comment, posted right away (the dots, as the 2024 posts ask):
   - what it is in two lines;
   - the config and HUD folder repo link;
   - the exact commands to run it in a window;
   - what is not done: pre-release, API changes, no daily driver, software
     rendering in this capture, the machine named;
   - one line that parts were written with AI coding tools, with the tests
     linked.
5. Stay in the thread for the first hours; answer every question, including
   "does it run waybar" (the tiling flavor uses waybar, README.md) and "can I
   get blur" (say what exists today and what not).
6. One post only. Show a new rice later only with a real change (the mods
   said small changes do not earn a repost).
7. If the post is removed, ask the mods why, politely, and do not repost.

## Risks and honest answers

### 1. Overselling

Risk: the headline says 3D, agents, voice; reality is pre-release and
several parts are plans.
- Rule: no claim without a demo that anyone can run or see. Each USP in
  usps.md has SHOWN and NOT SHOWN; the site tags planned items.
- Say it: "Pre-release. It runs in a window. It is not a daily driver. The
  3D wobble you see is from <date> on <machine>."
- If asked for something not built: "Not yet. It is in design/wobbly-3d.md as
  step E3; here is what works today."
- Do not publish frame rates, user counts or comparisons without a measured
  source. Do not say "first", "only", or "faster than".

### 2. Buzzwords

Risk: "agentic", "AI-native", "spatial" lose the ricers and attract hostility
(LLM use is a live topic in 2026; HN rejects "quickly-generated" projects,
see https://news.ycombinator.com/showhn.html, and the Lobsters and r/linux
discussions in the research draft).
- Lead with what you see: a window, a cube, a folder.
- Say "AI helper" or "agent" once, with a concrete act: "it moved the
  terminal next to the notebook after you asked".
- Never use: revolutionary, next-gen, seamless, AI-powered, the future of.
- Show the history and tests as proof of work, and say plainly that AI
  coding tools were used. Hiding it would cost more than saying it.

### 3. "It is just Compiz"

Risk: the cube and wobbly windows look like 2006. Compiz shipped from Novell in
January 2006, with the cube and wobbly windows as its showpieces, and the
effects were criticized for not helping the workflow
(https://en.wikipedia.org/wiki/Compiz;
https://lists.ubuntu.com/archives/ubuntu-devel/2006-October/022143.html).
KWin has both too (https://bugs.kde.org/show_bug.cgi?id=498760).
- Agree where true: "Yes, the effects come from that line. The wobble uses the
  KWin spring model (chora_kit::wobbly). We did not invent them."
- Say what differs, only what the demos show: the effects run on a scene with
  real depth and on windows from any window manager plugin; the whole look is a
  plugin you can replace; an agent can change the scene through the same API.
- Do not claim the effects are better. Show the 3D pull-out (D2), which the
  old cube did not do in the sources I read; say "I do not know of one", not
  "nothing else does it".
- Add the usefulness answer: what the 3D space is for (panes of real
  programs, a loader of views), not only decoration (lesson of Looking Glass,
  see usps.md 1).

### 4. "Another compositor, why not Hyprland or niri?"

- Answer: "If you want a stable daily driver, use them; niri and Hyprland are
  far ahead in polish. chora is for trying a different model: a scene you
  script, 3D, and agent control." Comparison page T3 with a "not yet" column.
- River already splits the window manager into a process; say chora adds a
  scene API and grants on top (usps.md 5).

### 5. "AI slop" and trust

- Show the repository history, the gate scripts and the plans. Link to the
  test list. Answer code questions with file paths, not with claims.
- Safety: say what an agent can do only with a grant, and where the claim is
  not tested (container recipe marked UNTESTED).

### 6. "It crashed" or "it does not start"

- Have the known caveats on the page (GNOME keeps Super; NVIDIA unsupported;
  software GL). Ask for the log, file an issue the same day, thank the person.
  A crash answered fast costs less than an unanswered one.

### 7. Packaging mismatch

- If someone asks for the AUR: say what exists (a package in each release).
  D49 says no AUR; see open questions. Do not promise COPR dates.

## Metrics to watch (no targets, none invented)

Issues opened by strangers, successful first runs reported, and the questions
asked, copied into docs T3 and T4 after week one.
