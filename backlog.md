# Backlog

One line per item, with its source. Order within a track is the order of
work. An item without an issue gets one when it is not done on the day it
is found (charter section 8). Started from charter section 15 on
2026-11-09.

## Track A

F5. A 2D guard run failed once with "no core" (the core did not come up)
   and passed on the rerun (11-10, render-fix gate). Possibly the silent
   core death under load (backlog 4, Fable lane); collect the core log of
   such a run.

F4. Gate waits under load: gate-hud waits 5 s for the HUD figure; at
   load 8.8 (battle tests in parallel) that timed out. Serialise load
   tests with merge gates (a lock file), and make the waits name the load.

F3. Flake: `a_confined_client_starts_no_drag` failed once in a parallel
   run, two tests sharing a temporary directory (11-10, grab fix). Give
   each test its own directory; grep the testing helpers for shared paths.

R2. Issue #76 part 3 (3D quad edges) is not done; step 6 covered parts 1
   and 2. `base_2d` in the 3D plan names nothing in the repo; drop it.

R1. Shader slots: a shader node rendered at depth 0 and inside a node
   texture at depth 1 in one frame overwrites slots 1 and 2; the depth-0
   element then draws the depth-1 result (pre-existing, review of 3D
   step 5). Also D75 "leaf planes draw their own texture" is not met:
   leaf planes still render one offscreen pass per commit.

F2. Flake: 14 `chora` lib render tests failed once in a full workspace
   run with an EGL display PoisonError (11-10, H6 gate); they pass alone.
   A panic in one render test poisons a shared display lock; find the
   first panic, make the display per test or recover from poison.

E1. The chat `epoch` restarts at 1 when chora-voice restarts; make it
   unique per voice start (start time plus counter), so a chat that
   lives through a voice restart never takes new answers as stale.

F1. The focus race fixed in tiling (1275e15: a relayout after a hand
   sent a stale `focus` that cleared the receiver's focus) may exist in
   the hand code of xp, mc and construct. Check each, with the same kind
   of test (`a_handed_window_leaves_the_focus_to_its_receiver`).

C1. One cube (user question 11-09): today `chora-xp` has its own desktop
   cube (`cube.rs`, 327 lines), a transform of the desktop layer only;
   the taskbar is a separate layer outside it and stays flat while the
   cube turns. D67 and D76 make the cube a construct geometry: four
   panes, each `chora-xp` on a virtual output (D69) with its own
   taskbar as part of the face, so the taskbar turns with its face;
   leaving XP flies out to that cube. Two cubes that behave differently
   break POLS and "minimal". Plan: the xp desktop switch asks the
   construct for the cube geometry; `cube.rs` goes when the geometry
   module (K9), virtual outputs (D69) and the cube (K11) work; the xp
   split X7 drops the cube from the gimmick. Until then no polish of the
   old cube beyond hiding the taskbar while it turns (small, one test).

0. DONE 2026-11-09, merge 103ce6f (claude/acp-gate): ACP permission
   requests that run or write take only a key or a HUD button; a spoken
   yes no longer answers them and "always" counts as no (was SECURITY,
   voice split review, core.rs:819 and core.rs:1047). Tests added with the
   fix; opus review: no high finding, medium and low findings fixed
   (state.md, Done). Still open from this item: the policy crate steps 1
   to 4 of design/split-review-voice.md.

1. Security check of the voice agent paths, attacker view (owner's notes
   2026-10-31; charter section 1 item 11). Running 11-09.
2. Security audit, the full list of charter section 15 item 2.
3. Verify gate 2 of 0.9 (hang watchdog, D64, `hang.rs`), then #89.
4. Battle backlog: silent core death under load; core memory growth in
   W9, W11, W12; mc dropped by the frame watchdog (owner's notes
   2026-10-30, `tests/battle/FRICTION.md`). No issue yet.
5. Flaky test `speech::tests::a_pause_stops_the_sound_and_the_resume_plays_the_cut_sentence_again`
   (owner's notes 2026-10-31). No issue yet.
6. Bugs: #53, #54, #56, #71, #70, #100, #98, #46, #36, #37, #34, #35,
   #38, #47, #74, #75; docs #69; `settings.set` cannot set an optional
   setting to none (owner's notes 2026-10-19, no issue yet).
7. Dependabot: #105, #104, #103, #95, #88, #86, #94, #93, #92, #2, #1;
   majors #96 (zbus 6), #87 (serde-saphyr 2).
8. Document the watchdog node limit (about 14,600 nodes) in
   `docs/api.md`.
9. Defects from the baseline of 11-09: `tests/gate-construct-keys.sh`
   fails ("the launcher became a pane: launcher term", twice);
   `tests/gate-construct-windows.sh` ends with exit 143 after its last
   ok line (twice). No issues yet.
10. Later: #102, #63, #73, #85, #81, #90, #97, #99.

10. Repository split: the 14 preparation steps of `design/split.md`
   section 5, in order, after the user answers questions 3 and 4; each
   step small and tested, no split.

11. XP split (D73, roadmap "Track: XP split", steps X1 to X7, plugins
    only, `tests/xp.sh` and the battle workflows green after each step).
    X1 done (4a13267). Next: X2, notifications and assistant as plugins,
    with chora-draw. X2
    (notifications as a plugin) also closes the notification gap of mc
    and construct; X4 (panel) is the shared bar of D51. Asked by the
    owner on 2026-11-09 (the "MCP plugin" in the transcript meant XP).

12. Split reviews due (`review_due.py`, 2026-11-09): chora-mc (6,695
    lines), chora-construct (6,429), voice (16,855). Each: a short
    design note in `design/`, a few meaningful parts or the reason not to
    split, recorded as "split review:" in `state.md`. After the XP split
    X1, one at a time, sonnet for the analysis, opus only for voice.

13. HARD REQUIREMENT, measured 11-09 by the matrix gate
    (`tests/gate-handover-matrix.sh`): 69 of 85 cells pass, 12 gaps, 4
    skips (46 of 84 at first; the construct adoption fix 674a7a9 and
    later fixes lifted it by 23 cells, and a 85th cell, the pane leak
    check "mc construct leak", was added). Gaps left: (b) no give key
    in tiling, xp, mc: 9 cells, and one key reaches only one target, so a
    `give: NAME` binding per target is needed; (c) mc cannot give by
    drag: 3 cells. Levers (a) construct focus on adoption and (d)
    `choractl give` without `--surface` in construct are closed. Bug: after an mc drag onto the
    construct output, construct loses its panes (`pane: none`); the gate
    repairs it in setup with `window.show`; needs its own fix and issue.
    Earlier text: window handover between all window managers by
    every input path (charter section 1 item 19). Gap matrix done
    (`design/window-handover.md`, 11-09): the 13 steps there, first the
    guard (the matrix gate with its expected table), then: one shared
    `give` key action in chora-kit; one MCP tool `give_window` and the
    voice prompt fix; mc drag and construct adoption at the window's own
    size with focus; the drop point; state across a move (one `give`
    meta key in the core).
14. Panes and geometries (D76): the geometry interface in chora-kit;
    cube, cylinder, wall and slider as geometries; the XP cube fly-out;
    several panes per source with input. After 3D steps 2 to 4.

15. Grants the user raises temporarily or permanently (question 6): a
    capability for acting on foreign windows and the camera from a
    confined connection; temporary grants with an end, permanent ones
    stored; voice asks for the missing grant and takes the answer only
    from a physical key or the HUD. Design first (opus), then the code.
    It restores the voice actions lost by fix (a).

16. The worker has no xwayland-satellite, so X11 programs (this
    LibreOffice has only its X11 plugin) do not run in persona sessions;
    office tasks of personas are only meaningful on the reference
    hardware or once xwayland is available here (owner's machine).

17. Flaky focus under load (11-09): with several parallel cargo test
    runs, the matrix gate sometimes ends a window in xp without keyboard
    focus (tiling xp cgive, tiling xp voice, xp mc cgive, xp construct
    cgive), also without construct. A possible real timing bug of focus
    on handover in xp. Reproduce under load, find the cause, fix.
18. Super+drag does nothing while the construct camera is inside an
    adopted window; the user must fly out first (Alt+Escape). Decide
    whether Super+drag should work from inside (POLS).

19. H1b (extraction of split reviews of 11-09, joined to the HUD sprint
    by the reflection of 11-09 evening, point 12): `chora-kit::ask` owns
    the confirm wire types (`voice.confirm`) and the answer call
    (`confirm.answer`), which exist three times today (mc
    agent.rs:86-165, construct hud.rs:933-975, assistant
    voice.rs:216-272). mc and construct switch to it in H9 and H10, one
    commit per plugin, also under `voice_hud: own`, so the sprint adds no
    fourth copy and one answer path is left to review. Security
    relevant: it is the answer path hardened on 11-09. Also the /proc
    launcher check (construct and mc) to chora-kit.

## Track C

S2. CUT from this session (user 11-10): Playground: windowed chora (winit) with an agent as a 2D/3D canvas;
   a distrobox/OCI image and a Nix flake to try it in minutes (user
   decision 11-10). Week 1, after the construct demo.
S3. CUT from this session (user 11-10): Effects pack, off by default, respects reduced motion: a closing
   window burns with flames and falling ash, an exploding window with a
   shock wave, more; the close never waits for the effect (it plays on a
   snapshot). After the 3D wobble demo.
S4. Website and USPs: design and marketing agents together, with a
   research agent; one headline, two paths; every claim backed by a
   demo; says pre-release. Orphan branch claude/website.

S1. Feature sprint "HUDs and ACP chat" (user request 11-09 evening,
   design/hud-sprint.md): ACP chats; HUD drafts for mc, xp and
   construct; a voice HUD that adapts to the context; HUDs that users
   ship themselves. With persona tests. Runs beside the foundation
   work; foundation merges keep priority.

0. Voice (design/split-review-voice.md): after the security steps, the
   tool `Source` registry, tool plugins as sandboxed MCP servers with one
   grant each (`tool_plugins`), and the layered context (scene, task,
   conversation) in `context.rs`.

1. 2D guard (running 11-09), then 3D per `plans/3d.md` (design running
   11-09). Issues #48, #76, #78, #83, #79 belong to the 3D steps.
2. construct: fix the K-step drift in the docs; K4; then K5 to K9 on 3D;
   `node.requested` looks (D58); mipmaps (roadmap 0.10).
3. Voice: map the context handling; layered context; tool plugin
   interface; latency to the first sound; `run_script`; settings changes
   confirmed by voice; barge-in package; HUD in nested sessions.
4. Core event `permission.requested` with answer method, timeout and
   default deny (owner's notes 2026-10-31).

## Track B

1. Driver for simulated users: done 11-09 (`tests/persona/`, 4e372b4).
2. Matrix of everyday tasks: done 11-09 (`baseline.md`). Gaps, most
   important first: Wi-Fi, volume, power and lock, battery, brightness,
   files and USB, screenshots, keyboard layout, Bluetooth.
3. Pitch candidate for 11-14: `chora-system`, one plugin over D-Bus
   (NetworkManager, BlueZ, UPower, logind, UDisks2; `wpctl` for volume;
   the notification server moved out of chora-xp) for all flavors; its
   methods become voice tools with the existing confirm rules. Tests with
   python3-dbusmock (friction item 4 first).
4. Core protocols that everyday tools need: wlr gamma control (night
   light), output power management (screen off when locked), wlr data
   control (clipboard history); a keyboard layout key in
   `input.configure`.

## Track D

0. Push-to-the-limit showcase (charter sections 8 and 9): the persona
   and its task in the panel from the first panel on; the screencast
   and GIFs once the 3D steps 3 and 4, the D76 slider and the handover
   guard work (week 2 or 3).

1. Research of audiences, competitors and channels: done 11-09
   (`drafts/audiences.md`). Notes: awesome-wayland is archived; FOSDEM
   2027 is 30 and 31 January; Show HN and r/linux are wary of quick or
   AI-made projects, so the README note on LLM tools and the test
   history must stay visible; closest design is river 0.4.
2. Five USPs, website on `claude/website`, announcement in German and
   English (charter section 9).

## CI1 (11-10): CI has no agent sandbox — DONE 2f6c3e0 (not run on GitHub yet)
grants2 makes `the_sandbox_hides_the_main_socket` fail when `CI` is set
and `agent-sandbox.sh` is missing. The script is not in the repo, so
ci.yaml on main/dev goes red. Before the first PR to main: ship a
minimal sandbox (bwrap) in dist/ and put it on PATH in CI, or install
it in the CI container. Also open from grants2: a HUD name whose HUD is
not running is free to any process; hello accepts the same executable
(several choractl), so the name guard holds against other programs
only.

## D1 (11-10): demo media, Reddit-ready
After K5 merges: a capture lane (light agent). `demo/record.sh` records a
headless session at 30 fps to frames, ffmpeg to a GIF (under 10 MB,
720p) and an MP4. Scenes as they become possible:
1. construct walk-through in 3D (camera path);
2. a construct pane dragged with 3D wobble;
3. the cube with xp faces rotating (after D69 steps 4-6 and cube
   geometry);
4. the pull-out of a window from the cube (after E3).
Media go to media/<date>/ on the log branch and the status page. Each
caption names the machine (software GL in a container).

## F6 (11-10): gate-hud cell card-stays-on-focus-move flaked
virtual-2 worker: one full gate run measured it missing; --only hud then
passed 3 of 3. Treat as a real timing bug until shown otherwise: find
the race, do not raise the wait.

## D2 (11-10): demo polish before any public GIF
The first 3D wobble capture (construct-wobble.png, wobble-frames/) is
not demo grade: the pane hardly bends; the content slides inside a
fixed frame (the glow and edge do not follow the mesh) and leaves a gap;
the side panes are flat grey; there is no scene clock, so captures run
at ~15 fps of real time. Needed before Reddit:
1. a capture clock: `choractl record` steps scene time at a fixed 1/30 s
   and writes frames (deterministic, independent of llvmpipe speed);
2. the pane frame, glow and shadow follow the wobble mesh; stronger,
   visible bending on a fast drag (tuned against the KWin feel);
3. a demo scene with real, colourful content (an image viewer, a code
   editor, a browser-like page), not empty terminals;
4. demo/record.sh + ffmpeg: GIF (<10 MB, 720p) and MP4.

## K4 (11-11): stand back from a cube face
The front face of a resting cube fills the screen at 1:1 and holds the
keyboard, so a person (and a gate) cannot step back to see the cube and
pull a window out into the space. Needs use/navigate modes (K4): a key
and a visible control that step the camera back while the face still
forwards input; the same act in every look (principle 1). Also: turn
keys for the cube, and a stats method so a gate can check that a still
cube renders 0 passes.

## V1 (11-11): a spoken "step in" should give the keyboard back
Since the K4 review, view.step_in by a method moves no keyboard (only a
person's key or chip does), so the LLM cannot take focus. But principle
11 counts a spoken command as a person's act: after "step in" by voice
the keyboard stays with the construct and a new window opens as a pane.
Needs a way for the voice service to mark a call as spoken by the
person (By::Voice), distinct from an LLM choice; design with security.

## P1 (11-11): the construct focus glow redraws every frame
Found by the K7 gate: while any pane has the focus, the construct's
focus glow is a timed shader, so the whole screen redraws every frame
(about 43 frames a second on llvmpipe). Calm and power: a still desktop
must draw nothing (render.stats 0). Fix: a static glow, or animate only
on a focus change and then stop. Priority: high (battery on laptops).

## F7 (11-11): hang watchdog test flaked once
hang::tests::a_main_loop_that_blocks_for_5_s_triggers_the_watchdog failed
once in a local run of the HUD worker (passed 3 later and in the gate).
Treat as a timing bug until shown otherwise.

## D3 (11-11): demo glitches to fix (seen in cube.gif, pull.gif)
(a) During a cube turn the floating planes of a face show for a few
frames as slivers beside the cube, before the face shows and after it
hid. (b) A pane dragged back over an angled face vanishes behind it for
some frames (one context sorts a mesh as one unit; see wobbly-3d.md
risks). (c) pull: the pane is smaller than its window and has no title
bar; the wobble in the space is subtle. (d) Light theme only; plain
white floor. Sixel pictures on faces: see the stale-frame fix.

## K12 (11-11, user asked): pull a window out of the stood-back cube
Today the pull-out works by a title drag past the face edge in use mode,
but at rest in use mode the face fills the screen (no edge to drag
past), and when stood back no face forwards. So a person cannot pull a
window out of the cube; the demo used a camera glide. Decision (PM):
in navigate mode, a modifier drag (or plain drag) on a floating window
plane pulls that window out: xp gives it with the grab to the
construct, which goes on with the drag and the wobble (E3 path). Drop
over a face gives it back. Gate: stood back, drag a floating plane off
the cube, it becomes a pane; drop on a face, it is back in xp.

Edges (user, 11-11: clipping is not least surprise; a window should
reach out into space): a window that hangs over the edge of its face is
drawn in full, the overhang continuing the plane of the face out into
the space, never clipped and never wrapped onto the next face. At rest
in use mode the face fills the screen, so the overhang lies off screen
as on a monitor; stood back and during a turn the floating plane shows
the whole window. Where windows of two faces would meet at a shared
edge (user, 11-11: adjust the distance), the floating lift resolves it:
the lift of a window grows until its plane clears every plane of the
neighbour face, within a cap; only beyond the cap do the planes cross
and the 3D base split them. Check first whether float planes already show the full window
(node texture of the group) and only the resting face clips; gate:
stood back, a window half over the right edge shows in full.

## C2 (11-11): the cube's xp child has no parent-death signal
The K7 fix set PR_SET_PDEATHSIG for manager panes only. If the construct
crashes, the cube's xp keeps running and the next cube.open waits on "a
client xp runs already". Same one-line fix in the cube's pre_exec.

## V2 (11-11): M1 for voice's own key-only questions
A spoken "Ja" now gets "For this I need a key press or a click" for a
helper's question, but a key-only question of voice's own turn still
gets only the earcon (its reply waits blocked). Speak the same phrase.

## T2 (11-11): gate-construct-hud.sh help row count stale
"the help has 21 rows, not 18": the gate pins an old count after new
bindings. Not a merge-gate step. Fix the count (or count from config).

## T3 (11-11): gate-construct-windows fails on the session branch
"d1 is not shown with tiling" on 536aaee (not a merge-gate step). For
rc criterion 3 every opt-in gate must pass once: find the cause.

## U2 (11-11): HUD colours per theme
mc in light and construct in dark show a chat in the wrong palette: a
folder carries one palette and the host cannot see a look's theme.
Decision (PM): a folder may carry a `dark:` block with the panel and
chat colours (each through the contrast check); the window manager
emits `<wm>.theme {light|dark}` and the host picks the block. Also: the
xp chat field sits over the taskbar (respect the work area).

## F8 (11-11): pull gate cell "plain drag over the front face" flaked
Failed once in the merge gate of ux-construct (2e2cd6b): "did not give
it back: s1 construct - 0 0"; passed 3 of 3 alone right after, load 1.
Find the race (likely the drop lands before the face rests again or the
pane rect is read mid-glide). Must be closed for rc criterion 3.

## L1 (11-11, user): logging and anonymised user reports (rc criterion)
"Ensure good and detailed logging in all components for debugging and
further decisions after real-world testing. We might ask users to submit
anonymised reports using a small shell script."
1. Logging audit per component (core, kit, xp, tiling, mc, construct,
   assistant, voice, chat, notify): structured tracing with a target per
   area; each act, refusal, handover, give, float, crash path and timing
   at a level a person can turn up (`RUST_LOG`-style setting in the D62
   settings); no secrets, no window contents or typed text; rotate files
   under $XDG_STATE_HOME/chora/logs. Part of the code review per area.
2. `chora-report` (small POSIX shell script, shipped in dist/): collects
   versions, the session's logs, settings (keys only where values could
   be private), GPU/driver, outputs, last words of crashes; anonymises
   (home path, user and host names, IPs, MACs, window titles and app
   ids unless the person keeps them); shows the person the whole bundle
   before anything leaves; writes a .tar.gz the person sends by hand (no
   upload, no telemetry). Its tests check the scrubbing on a fixture.
3. Docs: how to report a problem, in README and the website.
4. Later, less friction (user, 11-11): the script asks a few questions
   (what you did, what you expected, what happened, how often, which
   look), then, after an explicit yes on a screen that shows exactly
   what leaves, uploads the anonymised bundle to a paste service (one
   configurable endpoint, with an expiry; no account) and prints a ready
   GitHub issue text (title, answers, versions, the paste link) and the
   new-issue URL with the text filled in. No upload without that yes;
   a `--no-upload` path keeps the file-only report.

## U3 (11-11): one surface per helper conversation, in every look
Seen in hud-mc/-dark and hud-construct/-dark after the theme merge: the
construct shows a floating sentence balloon apart from the open chat,
and mc repeats the sentence top right while the drawer is open. Rule
(as the user set for xp): while the chat is open, the sentence and the
question live inside the chat surface; no second balloon. Also OPEN CUBE
touches the screen edge (fix pending in claude/ux-construct).

## F8 closed (11-11): the pull put-back flake was a real bug
Root cause: a pane dropped beside the cube could sit deeper than the
front face; a press there missed it under load. Fixed in cdf5985, gate
forces the case every run (d145e0e).

## F9-F11 (11-11): flakes on unchanged code, rc blockers
- F9 wm-pane "a typed key did not reach the window in the pane of xp":
  once in a full gate; 10/10 alone under load.
- F10 words::a_stack_overflow_leaves_its_words got signal 14 (SIGALRM),
  not 11: under load the backtrace after the fixed line takes longer
  than the alarm(2) of the fault handler, so the alarm ends the process
  first. Decide: longer alarm under test, or accept SIGALRM as an end of
  the same crash path in the test.
- F11 last-words gate "no last words after SIGSEGV": likely the same
  timing.

## S5 (11-11): showcase findings (media/2026-11-11/showcase/findings.md, 86)
Bugs to triage into the area lanes: mc Agents scene empty (agent.list
[] while voice names Atlas/Coder); OPEN CUBE black flash 0.5 s and a
pane pill over the face; a chat opened by voice does not take the
keyboard (by design until V1: spoken act); injected Alt+Y silent and
"No" gives no "Refused." feedback; pane.manager places panes outside
the view; window.give refuses a window with no pane; navigate-mode drag
on a floating window moves it off the face instead of turning.
Visual: the question card covers the transcript and its hint is cut;
dark construct gets white pills; light mc gets a near-black drawer;
tiling Chat pill overlaps the bar; tag "1" and OPEN CUBE at x=0; long
headers ellipsized. Built at df8dadd (before 67f93a9); re-check those
the construct merge may have fixed.
S5 update (11-11 ~21:10, showcase re-recorded on 67f93a9, de69c66):
fixed there: dark construct HUD/chat, light mc drawer, tiling pill
below the bar, xp helper chat above the taskbar. Still open: mc Agents
empty; OPEN CUBE black 16 frames and the SUNSET pill on the face; voice-
opened chat without keyboard (V1); injected Alt+Y and No give no
feedback; question card white in dark chat, covers transcript, hint cut;
xp chat without a helper covers the taskbar; OPEN CUBE overlaps the chat
and moves at 2x; pulled pane cut at the right edge.

## N1 (11-11, user idea): a modern look on xp's stacking window manager
"Now that we have blur: modern UI on top of xp's stacked WM code. Like
a separate macOS-inspired theme with dock and top bar. (Taskbar shall
be generalized to a widget?)" Plan after the rc: the stacking window
manager of chora-xp becomes the base of two looks (xp and a new modern
look); the taskbar, the tray and the Start menu become shared widgets in
chora-kit (a bar widget that renders as an XP taskbar, a top bar or a
dock), so a look is a theme plus a widget layout. Needs its own design
document (design/looks/) and the user's approval of mockups first.
