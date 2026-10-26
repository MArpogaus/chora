# Roadmap

The path from the current state to chora 1.0. Version 1.0 is a daily driver
with a stable, versioned API (D3).

## Rules

- Each milestone is a minor release: 0.6, 0.7 and so on.
- Each milestone has a goal, a scope, an estimate and a **gate**.
- The gate is a list of checks. A check is a CI job or a step on the
  reference hardware with a log as evidence (D46).
- A milestone is done when every check of its gate holds. The next
  milestone of the core starts only then.
- Each milestone has a gate issue on GitHub with the checks as a task
  list. Each ticked check links to its CI run or log. The release of the
  milestone closes the issue.
- Two tracks run in parallel to the core: the XP showroom and the voice
  pipeline.
- The showroom track has no milestones of its own. Its CI must be green
  against each release candidate of the core, so every core gate has that
  check.
- The voice track has milestones of its own. It never blocks the core.

## Current state

- The core runs nested and as a login session. It has no long run on real
  hardware.
- The scene has five node kinds, 4x4 transforms, cameras, overlay roots,
  meshes, effects and animations.
- The API covers scene changes, events, grabs, plugin methods, captures,
  input injection, grants, programs, assets and output configuration.
- `choractl` loads views and serves MCP.
- `chora-tiling` and the XP showroom are plugins in this repository.
- The core is a library and a binary. Unit tests run IPC dispatch,
  transactions, focus and grants in process, without a backend. The tty
  backend has no tests. One pixel test renders in process on the headless
  backend. Pixel tests in a headless sway check the behavior.

## Estimates

The estimates count the lines of Rust that a milestone adds or changes.
They also count the hours of agent work and the hours of hands-on work of
the user. The calendar time includes the gate. Agent hours include the
tests and the review rounds.

The basis is this repository: about 26,000 lines of Rust in 54 hours of
calendar time. A rework of existing code costs more than new code. The
table uses about 140 to 200 lines per agent hour for 0.6 to 0.9. The
milestones 0.10 and 1.0 have hardware work and use about 80 to 125.

The user uses chora as the daily session from milestone 0.10. The
gates of 0.10 and 1.0 measure that use, so they set the calendar time.

| Milestone | Rust lines | Agent hours | User hours | Calendar |
|-----------|-----------:|------------:|-----------:|---------:|
| 0.6 Foundation | 2,500 | 14 to 18 | 3 | 1 week |
| 0.7 Identity, processes and read model basics | 3,000 | 16 to 20 | 3 | 1 to 2 weeks |
| 0.8 Agent read model | 4,600 | 23 to 30 | 3 | 2 weeks |
| 0.9 Renderer | 7,000 | 40 to 50 | 6 | 3 weeks |
| 0.10 Daily driver | 2,500 | 20 to 30 | 8 plus daily use | 3 weeks |
| 1.0 Stable API | 1,500 | 12 to 16 | 4 plus daily use | 5 to 6 weeks |
| **Total** | **21,100** | **125 to 164** | **27** | **15 to 17 weeks** |

The renderer has the largest risk. A fault on real hardware in 0.10 can
add one to two weeks.

## 0.6: Foundation

Released as `v0.6.0` on 2026-10-19 without the gate checks 1, 2 and 5.
These checks moved to the gate of 0.7. Check 1 then moved to the gate of
1.0: the repository stays one monorepo until then.

**Goal:** the project works with gates in the GitHub organization
`chora-desktop`. Unit tests check the core in process.

Scope:

- The organization `chora-desktop` with the core repository (D36). The
  split of the plugins into repositories of their own is postponed past
  0.6; the plugins build with the core.
- `camera.release` on the admin socket: the administrator takes the
  camera of an output from its owner, live or gone.
- One socket directory per client: `$XDG_RUNTIME_DIR/chora/clients/NAME/`
  with `ipc.sock` and a Wayland socket `wayland.sock` (D33, D40). The
  socket fixes the name of its clients. `choractl clients up` applies a
  `clients.yaml`. The emergency reset removes the client sockets. An
  example Quadlet unit in `dist/` shows an agent container that mounts
  its client directory.
- The `ipc.sock` of a client socket narrows the scene: its connections see
  and change only their own nodes and windows. They set no camera, use
  no `screen:*` parent, and use `focus_node` only after a press.
- The capability `input.grab-any` allows grabs without Ctrl, Alt or Super
  (D45).
- `surface.created` carries the pid and the activation token.
- An unknown event name in `events.subscribe` is an error.
- The git flow of D47 in every repository: a protected `main`, releases
  from a merge commit and an annotated tag.
- The release workflow starts from a tag and needs a green CI run of the
  same commit.
- A pinned Rust toolchain in `rust-toolchain.toml`, a job for the minimum
  Rust version, and container images pinned by digest.
- The core as a library and a binary (D48). IPC dispatch, the transaction
  queue, focus and grants run in unit tests with a fake clock.
- A headless test backend that renders with GLES on the software driver
  of Mesa in process, for pixel tests in Rust. GLES checks the same render
  path as the screen.
- A smoke test in a VM with `vkms` in CI (D11). chora starts on the tty
  backend and shows a foot window. A capture shows its pixels.
- The reference documentation stays with the code of the core; the
  website repository follows with the split (D36).

Gate:

1. The organization `chora-desktop` and the core repository exist, and
   `main` accepts pull requests only.
2. A test release `v0.6.0-rc.1` comes from a tag on a merge commit. A
   tag on a commit with red CI starts no release.
3. `cargo test` runs at least 40 unit tests of the core crate for IPC,
   transactions, focus and grants, without a compositor.
4. One pixel test runs in process on the headless backend.
5. The `vkms` job is green in 10 runs in a row.
6. `tests/nested.sh`, `tests/agent.sh` and `tests/xp.sh` are green in CI.
7. `rust-toolchain.toml` pins the toolchain, and the job for the minimum
   Rust version is green.
8. Every container image in the workflows has a digest.

## 0.7: Identity, processes and the basics of the read model

**Goal:** a client cannot take another name or give itself capabilities,
the core starts no programs, and an agent can see focus and geometry
without guessing.

Scope:

- The client socket also fixes the capabilities of its clients: the
  socket decides which methods a connection may use (D33).
- A versioned `hello` with the protocol version and the features of both
  sides (D29). Until then `hello` answers with the version of the build,
  and the client library warns on a mismatch.
- Stable error codes: one code for each kind of failure. The error data
  already names the failed command.
- The core starts no program except its startup command (D42).
- Focus in queries: the surface and the node that have the keyboard.
- Queries return the screen rectangle of a node on each output.
- `choractl wait SELECTOR` returns when a node matches, or fails after a
  timeout.
- `input.inject` keeps a button down across requests with `hold`, so that
  a drag can span several requests. A dry run checks a batch without
  applying it.
- `chora-tiling` exports methods for tags, focus and moves. `chora-xp`
  exports methods for the start menu, minimize, restore and Restart.
- Small fixes from the battle test: compact JSON from `choractl run`,
  a clear error from `clients create` for a taken name,
  `chora-probe --exit-on-close`, and `tests/ipc-bench.sh` on the headless
  session.

Gate:

1. Moved to the gate of 1.0: the organization and the protected `main`.
2. A test release `v0.7.0-rc.1` comes from a tag on a merge commit. A
   tag on a commit with red CI starts no release (from 0.6).
3. The `vkms` job is green in 10 runs in a row (from 0.6).
4. A Podman container that mounts only `clients/agent/` queries the scene
   and draws a node. Its `grants.set` and its `hello` with another name
   fail. A CI job checks this.
5. A Quadlet unit from `dist/` connects through its client directory
   after a restart of chora, without a change to the unit.
6. A client with `input.grab-any` grabs F13 and a letter key. A client
   without it gets the error code for a missing capability.
7. The core has no code that forks a process except the startup command.
8. `choractl run foot` starts foot in a transient systemd unit of its
   own, and the `surface.created` event of its window carries the
   activation token. Without a user systemd, the same command starts foot
   directly.
9. Each error code of the API has a test that provokes it.
10. The battle workflows in `tests/battle/` pass with `choractl wait`, the
    focus and the rectangles of queries and the plugin methods, without
    pixel clicks.
11. The gate checks of the earlier milestones still hold, and the showroom
    CI is green against the release candidate.

## 0.8: Agent read model

**Goal:** an agent sees what the user sees, it can ask the window manager
for changes, and it can label what it shows.

Scope:

- JSON filters replace the selector strings (D32). They can match focus
  and visibility.
- `events.subscribe` starts with a snapshot of the state, and every event
  has a sequence number.
- `input.inject` takes a node and a point inside it.
- A client that does not own a node can request a change: focus, move,
  resize or close. The core sends the request to the owner as an event,
  and the owner decides (D31). `chora-tiling` handles the requests.
- Plugin methods carry a description and a JSON schema. `choractl mcp`
  makes one tool from each method (D34).
- `choractl mcp` uses rmcp, answers requests in parallel and supports
  cancellation.
- Text injection types every character of the text, also characters that
  the current keymap does not have, for example German umlauts on a US
  layout.
- A visible indicator while a client injects input, and a binding of the
  safety layer that revokes the capabilities of all agents (D23).
- A text node (D18): text with a font family, a size, a color and a
  maximum width. cosmic-text shapes it and finds fallback fonts. The
  indicator, plugins and agents use it for labels without a client.

Gate:

1. An MCP client in CI calls the tools to focus the second terminal and to
   request a move to the right. `chora-tiling` performs both, and a capture
   shows them.
2. Under a camera with pan and zoom, a click on a node through the MCP
   tools hits that node in 20 of 20 tries.
3. `wait_events` returns while another tool call runs, and a cancelled
   call ends within 100 ms.
4. Injected text `Grüße äöüß` appears unchanged in foot on a US layout.
5. A subscription that starts during a transaction misses no event and
   sees no event twice: a test with 1,000 transactions checks the
   sequence numbers.
6. The tool list of `choractl mcp` matches the exported methods of the
   plugins after each `methods.export`.
7. A capture shows the indicator while a client injects input.
8. A test shows that the revoke binding removes the capabilities of all
   agents.
9. A text node shows `Grüße 你好 🙂` with fallback fonts, and a pixel test
   against a reference capture passes.
10. An MCP client puts a text node under `screen:NAME`, and a capture shows
    it.
11. The gate checks of the earlier milestones still hold, and the showroom
    CI is green against the release candidate.

## 0.9: Renderer

**Goal:** the core renders only what changed, draws the full 4x4 model,
and drives plugins with the frame clock (D17, D24).

Scope:

- A retained scene: each node keeps its world transform, its bounds and
  its render resources. Integer handles replace string keys inside the
  renderer.
- A scene renderer of chora on GLES 3. It draws each node as a quad with
  its full matrix. Rounded corners, borders and clips come from signed
  distance fields in the same pass.
- A layer renders offscreen only for group opacity, effects or an explicit
  `cache`. A layer keeps its texture until its subtree changes. Textures
  have mipmaps, and their resolution follows the size on screen.
- A redraw state machine per output. Without damage, the output renders
  nothing and the loop sleeps. A fullscreen surface without effects goes
  to direct scanout.
- Frame events with the predicted presentation time for subscribed
  plugins (D28). Animations run on the same clock. They interpolate
  translation, rotation and scale apart, run per property, and support
  springs.
- Transactions on disjoint surfaces do not wait for each other (D22).
- Shader compile and capture encode run on worker threads, as the image
  decode does.
- The watchdog runs in its own thread with a heartbeat of the main loop
  (D23).
- Debug counters for rendered frames and texture allocations, readable
  through the API.
- fd passing for images and captures over the IPC socket (D54).
- A measurement of the plugin round trip at 120 Hz decides on a private
  Wayland protocol for plugins (D54).

Gate:

1. A static scene with rounded windows, a clipped group and a blurred
   panel renders 0 frames in 10 s of idle time. A counter of the core
   shows it, in CI and on the reference hardware.
2. On the reference hardware at the refresh rate of the internal panel:
   four windows with blur and rounded corners and a 300 ms transition.
   `presentation-time` shows 99 percent of the frames on time over 60 s.
3. A node with rotation and no size renders. A camera with zoom 2 keeps
   text sharp: a pixel test against a reference capture passes.
4. A size animation of a layer over 2 s allocates no new texture after the
   first frame. A counter of the texture pool shows it.
5. A client that delays its commit for 150 ms delays no transaction of
   another client. A unit test checks it.
6. A main loop that blocks for 5 s triggers the watchdog. A test with a
   debug hook checks it.
7. The wobbly windows, the magic lamp and the cube of the showroom run on
   frame events. At 60 Hz and at 120 Hz in CI, 99 percent of their frames
   are on time over 30 s.
8. The decision on the plugin protocol is in the decision log, with the
   measurement.
9. The gate checks of the earlier milestones still hold, and the showroom CI
   is green against the release candidate.

## 0.10: Daily driver

**Goal:** the user works in chora every day on the reference hardware,
and installs it from a package repository.

Scope:

- Packages through COPR with Packit and subpackages per plugin, and
  PKGBUILDs in the AUR (D49). The release keeps its assets.
- The image of `secureblue-images` installs chora from COPR.
- The session starts through uwsm (D50). A flavor is a systemd target.
- The tiling flavor: `chora-tiling` with waybar, fuzzel, mako, swaylock,
  swayidle and kanshi (D51).
- Input device settings through the API: tap, acceleration, natural
  scroll, key repeat. A view sets them, so the core reads no file (D35).
- Output hotplug, suspend and resume, lid switch, and the VT switch on the
  reference hardware.
- Screen sharing through `xdg-desktop-portal-wlr`, and X11 apps through
  `xwayland-satellite`.

Gate:

1. The secureblue image builds with chora from COPR. `rpm-ostree upgrade`
   on the reference hardware updates chora, and the login screen offers the
   session.
2. The user works 10 working days in a row only in chora on the reference
   hardware. `coredumpctl list chora` shows no entry, and the log shows no
   emergency reset that the user did not start.
3. A checklist on the reference hardware passes, with the log of each
   step. An external monitor goes in and out 10 times, and the laptop
   suspends and resumes 10 times. The user closes and opens the lid,
   switches the VT, and locks and unlocks the session. A screen share in a
   browser and an X11 app work.
4. The `vkms` job covers hotplug of a second output.
5. The AUR packages build in CI.
6. A view sets tap and key repeat, and a test shows that the core applies
   them.
7. The gate checks of the earlier milestones still hold, and the showroom CI
   is green against the release candidate.

## 1.0: Stable API

**Goal:** the API is version 1 and stays compatible until 2.0.

Scope:

- The protocol version 1: a JSON schema of all methods, events and errors
  in the repository, generated from `chora-scene`. A CI job fails on a
  change of the schema without a change of the protocol version.
- The reference documentation comes from the schema. A test checks every
  example of the documentation against it.
- A final review loop over the whole code base until a round finds
  nothing that matters (D47).
- The organization `chora-desktop` (D36). The core repository moves
  there, and `main` accepts pull requests only. The plugins, the XP
  showroom and the voice pipeline move into repositories of their own.
- All plugins of the organization use the protocol version 1.

Gate:

1. The user works 30 days with the release candidate as the daily
   session. `coredumpctl list chora` shows no entry.
2. No open issue has the label `severity:high`.
3. The last review round leaves no open finding of high or medium
   severity.
4. The schema job and the documentation test are green.
5. Every plugin repository of the organization pins protocol version 1,
   and a CI job checks it.
6. The organization `chora-desktop` and its repositories exist, and
   `main` of each accepts pull requests only (from 0.6).
7. The gate checks of the earlier milestones still hold, and the showroom CI
   is green against the release candidate.

## Track: XP showroom

The showroom grows in the core repository and moves to
`chora-desktop/chora-xp` with the split at 1.0 (D36, D39). It shows
the effects of the scene and serves as their test.

- Each core milestone ends with a showroom release that uses its API. The
  core gate checks that the showroom CI is green against the release
  candidate.
- Scope and order of new effects are the decision of the user. Each new
  effect gets a pixel test in `tests/xp.sh`.

## Track: voice pipeline

The voice pipeline lives in `services/voice/` of the core repository and
moves to `chora-desktop/chora-voice` with the split at 1.0. It started with
milestone 0.6 (D53). It runs as Quadlet units on the reference hardware:
speech to text, a router model, a main model and text to speech, served by
[Gufo](https://github.com/gufo-org/gufo).

| Milestone | Goal | Gate | Needs from the core |
|-----------|------|------|---------------------|
| V1 Pipeline | speech in, an answer as speech out, with barge-in | median 1.0 s or less from the end of speech to the first audio, over 20 German utterances | nothing |
| V2 Control | the voice agent drives chora through MCP | "move the terminal to the right" works in 18 of 20 tries | 0.7 (push-to-talk with `input.grab-any`), 0.8 (read model, text injection) |
| V3 Views | a morning view and task views | "show my tasks for today" opens a view with the tasks of an MCP source; "open a view for the first task" opens a view with two surfaces | 0.8 |

## After 1.0

- Small plugins that window managers share: a panel with a task list and
  the notifications. Until then, shared code lives in `chora-kit`.
- Conformance tests with wlcs.
- Items from the [Idea backlog](ideas.md).
