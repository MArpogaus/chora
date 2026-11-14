# Backlog

One line per item: id | source date | `rc` or `later` | text. `rc` = needed for the tag (design/rc.md);
`later` = not before the tag. Done items are deleted (git has them). Fix lanes take UX findings from the
reports (reports/swarm-6.md, ux-deep-*.md, ux-cross-look.md) until the findings ledgers exist.

## Flakes and gates (rc 3, 9)
F2 | 11-10 | rc | 14 `chora` lib render tests failed once with an EGL display PoisonError; a panic in one test poisons a shared display lock: find the first panic, make the display per test or recover.
F3 | 11-10 | rc | `a_confined_client_starts_no_drag` failed once in a parallel run: two tests share a temporary directory; give each its own, grep the test helpers for shared paths.
F4 | 11-10 | rc | gate-hud waits 5 s for the HUD figure and timed out at load 8.8: serialise load tests with merge gates, make the waits name the load.
F5 | 11-10 | rc | a 2D guard run failed once with "no core" and passed on the rerun: collect the core log of such a run.
F6 | 11-10 | rc | gate-hud cell card-stays-on-focus-move flaked once in a full gate: find the race, do not raise the wait.
F7 | 11-11 | rc | hang::tests::a_main_loop_that_blocks_for_5_s_triggers_the_watchdog failed once: timing bug until shown otherwise.
F10 | 11-11 | rc | words::a_stack_overflow_leaves_its_words gets SIGALRM under load (alarm(2) ends the process before the backtrace); with F11 it fails about 1 in 3 full gates (11-12): longer alarm under test or accept SIGALRM.
F11 | 11-11 | rc | last-words gate "no last words after SIGSEGV": likely the timing of F10.
F13 | 11-12 | rc | cube navigate-click passes 2 of 5 under load (construct-v2 gate).
F14 | 11-12 | rc | cube cell "construct does not answer while the wm is deaf" failed once (construct r3 gate).
C2 | 11-11 | rc | the cube's xp child has no parent-death signal: after a construct crash the next cube.open waits on "a client xp runs already" (flaked 11-12); set PR_SET_PDEATHSIG in the cube's pre_exec as K7 did for manager panes.
A5 | 10-31 | rc | flaky `speech::tests::a_pause_stops_the_sound_and_the_resume_plays_the_cut_sentence_again`; a chora-voice test flaked once in a gate on 11-12 (which one is not recorded).
A17 | 11-09 | rc | focus under load: the matrix gate sometimes ends a window in xp without keyboard focus (cgive and voice cells); claude/focus-flake is merged: confirm closed or reproduce.
F1 | 11-10 | rc | the focus race fixed in tiling (1275e15: a stale `focus` after a hand) may exist in the hand code of xp, mc and construct: check each with the same kind of test.
A9 | 11-09 | rc | `tests/gate-construct-keys.sh` fails ("the launcher became a pane: launcher term"); opt-in gates must pass once.
T2 | 11-11 | rc | gate-construct-hud.sh pins a stale help row count ("21 rows, not 18"): fix the count or count from config.
T3 | 11-11 | rc | gate-construct-windows fails ("d1 is not shown with tiling" on 536aaee; exit 143 at the 11-09 baseline): find the cause.

## CI, packaging, dependencies (rc 4, 5, 7)
CI2 | 11-12 | rc | cargo deny fails on an advisory (xattr "unsound", via tar via tract-onnx): a fixed tar/xattr version or a reasoned ignore in deny.toml before the CI draft PR.
CI3 | 11-10 | rc | CI has never run on GitHub for this branch (the agent sandbox for CI shipped in 2f6c3e0): open the draft PR.
PK1 | 11-11 | rc | dist/arch/PKGBUILD installs only *.service, *.target and waybar/* of tiling: add foot.ini, fuzzel.ini, swaylock.conf, wallpapers, power.sh, screenshot.sh, waybar/icons and deps (inter-font, ttf-jetbrains-mono, grim, slurp, wl-clipboard, libnotify); same check for the Fedora spec.
PK2 | 11-11 | rc | packages must depend on TeX Gyre Pagella (Arch tex-gyre-fonts, Fedora texlive-tex-gyre or the otf package) next to Inter and JetBrains Mono; without it the mc chat falls back to DejaVu Serif.
PK3 | 11-11 | rc | tiling: no mako style ships (notice unthemed); libnotify and a minimum fuzzel version for mouse use as optional package deps.
A7 | 11-09 | rc | Dependabot: #105, #104, #103, #95, #88, #86, #94, #93, #92, #2, #1; majors #96 (zbus 6), #87 (serde-saphyr 2) may wait.

## UX and looks (rc 1)
S5 | 11-11 | rc | showcase findings still open after the re-record on 67f93a9: mc Agents scene empty; OPEN CUBE black for 16 frames and the SUNSET pill on the face; injected Alt+Y and "No" give no feedback; question card white in the dark chat, covers the transcript, hint cut; xp chat without a helper covers the taskbar; OPEN CUBE overlaps the chat and moves at 2x; pulled pane cut at the right edge. Recheck on train 4.
U2 | 11-11 | rc | HUD colours per theme: a folder may carry a `dark:` block, the manager emits `<wm>.theme`; partly fixed by the theme merge (S5 update): confirm in the pixel pass; xp chat field over the taskbar.
U3 | 11-11 | rc | one surface per helper conversation in every look: while the chat is open the sentence and the question live inside it, no second balloon (construct, mc); confirm after train 4.
MC1 | 11-11 | rc | open from the mc round: N4 hover marks on card buttons, N3 chat field style in services/voice/chat, C6 picker row (MINOR: only if they sit on a main path).
A13 | 11-09 | rc | after an mc drag onto the construct output, construct loses its panes (`pane: none`); the matrix gate repaired it in setup with `window.show`. Check whether it still happens (no known window loss).

## Review, docs, logging (rc 6, 8, 9a)
A2 | 11-09 | rc | security audit as part of the per-area review: the list of archive/charter.md section 15 item 2; the attacker check of the voice paths ran on 11-09 (reports/week-1.md).
G1 | 11-10 | rc | open from grants2, for the security review: a HUD name whose HUD is not running is free to any process; hello accepts the same executable (several choractl).
L1 | 11-11 | rc | logging audit per component (structured tracing, a target per area, no secrets or typed text, rotating files under $XDG_STATE_HOME/chora/logs); `chora-report` shell script in dist/ (anonymised .tar.gz the person sends by hand, scrubbing tested on a fixture); docs: how to report a problem.
R4 | 11-12 | rc | release notes name the changed mc default keys (Super+a chat, Super+Return terminal scene).
A8 | 11-09 | rc | document the watchdog node limit (about 14,600 nodes) in `docs/api.md`.

## Later (sprints chosen by an idea round after the rc, owner 11-14 05:26; not before the tag; cut list of reflections/2026-11-14-w2-1-fable.md section 6)
D4 | 11-12 | later | construct Host refactor: each cube face runs its own manager (XP, Tiling, Media center, XP 2); xp-only paths (windows.float, activate, pull) become per face.
K4 | 11-11 | later | stand back from a cube face: use and navigate modes, a key and a visible control, turn keys, a stats method for "a still cube renders 0 passes".
K12 | 11-11 | later | pull a window out of the stood-back cube by a drag on its floating plane; a window over the edge of its face is drawn in full, never clipped or wrapped; the lift grows until planes of neighbour faces clear.
V1 | 11-11 | later | a spoken "step in" should give the keyboard back: the voice service marks a call as spoken by the person (By::Voice); design with security.
V2 | 11-11 | later | a key-only question of voice's own turn gets only the earcon: speak "For this I need a key press or a click".
P1 | 11-11 | later | the construct focus glow is a timed shader and redraws every frame (about 43 fps on llvmpipe): a static glow or animate only on a focus change. High after the tag (battery).
A11 | 11-09 | later | XP split (owner 11-14 05:25: theming apart from the stacking WM; 05:26 no priority) X3-X7 (D73, roadmap "Track: XP split"; X1 4a13267 and X2 e5391bd are merged): chora-stack, chora-desk, chora-panel.
N1 | 11-11 | later | widget host: chora-panel hosts widgets (task list, start button and menu, tray, clock, dock) with a widget API in chora-kit; done when a second layout (dock plus top bar) runs from config only. Needs a design document and the owner's approval of mockups.
X8 | 11-12 | later | an XP Run dialog (icon, text, Open combo, OK/Cancel/Browse) instead of fuzzel for Start > Run.
X9 | 11-12 | later | chora-xp-import (port of kde-plasma-winxp-launcher get-microsoft-files.sh, design/xp-reference.md) and the imported-asset pixel benchmark; pending the owner's answer to state.md Needs owner (b).
PX1 | 11-14 | later | further MINOR and COSMETIC pixel rounds, full 2x reviews, more test swarms before the fix lanes land.
T4 | 11-11 | later | tiling v2 leftovers: 68 px empty after "Tile"; one keycap holds "Super+r / Super+Space"; open terminals stay dark after switching to Paper.
R3 | 11-11 | later | tiling code review LOW: layout words duplicated in waybar/layout.sh and keys.rs; locale case repeated in 5 scripts and theme.rs; runtime_shader re-implements theme::runtime_dir; LaunchError one-variant enum matching strings; tags 10-16 allowed but the bar has 9 fixed modules.
R5 | 11-12 | later | mc code review LOW: `empty` is a string on scene panels and a bool on contents; key labels English only, short and full lists disagree on Run; sent echo removed only on exact text match; wraps "run." and "Stop" alone on a line.
L1b | 11-11 | later | chora-report asks a few questions, uploads the bundle to a paste service after an explicit yes and prints a ready GitHub issue text; `--no-upload` keeps the file-only report.
M1 | 11-12 | later | status page rework, website and USPs on claude/website (one headline, two paths, every claim backed by a demo), announcement in German and English.
D1 | 11-10 | later | demo media, Reddit-ready: `demo/record.sh` (30 fps frames, GIF under 10 MB, MP4); construct walk, 3D wobble, cube turn, pull-out; captions name the machine.
D2 | 11-10 | later | demo polish: a capture clock in `choractl record`; the pane frame, glow and shadow follow the wobble mesh; a demo scene with real content.
D3 | 11-11 | later | demo glitches: slivers of floating planes during a cube turn; a pane dragged over an angled face vanishes for some frames; the pulled pane is smaller than its window; light theme only.
D0 | 11-09 | later | push-to-the-limit showcase for r/unixporn: the persona and its task, screencast and GIFs.
V3 | 11-11 | later | voice model research (design/voice-models-research.md) and voice tests with real audio.
C0 | 11-09 | later | voice: the tool `Source` registry, tool plugins as sandboxed MCP servers with one grant each, layered context (scene, task, conversation); latency to the first sound; `run_script`; settings changes confirmed by voice; barge-in; HUD in nested sessions.
A0 | 11-09 | later | the policy crate, steps 1 to 4 of design/split-review-voice.md.
A19 | 11-09 | later | H1b: `chora-kit::ask` owns the confirm wire types and the answer call, which exist three times (mc agent.rs, construct hud.rs, assistant voice.rs); also the /proc launcher check to chora-kit.
A15 | 11-09 | later | grants the user raises temporarily or permanently (design/grants.md; steps 1 and 2 merged): temporary grants with an end, voice asks for the missing grant, answer only by key or HUD.
C4 | 10-31 | later | core event `permission.requested` with answer method, timeout and default deny.
C1 | 11-09 | later | one cube: `chora-xp/cube.rs` goes when the construct cube geometry serves the xp desktop switch.
A14 | 11-09 | later | geometries (D76): cylinder, wall and slider beside the cube; several panes per source with input.
C3 | 11-09 | later | construct: K-step drift in the docs; K5 to K9; `node.requested` looks (D58); mipmaps.
A18 | 11-09 | later | Super+drag does nothing while the construct camera is inside an adopted window: decide (POLS).
R1 | 11-10 | later | shader slots: a shader node at depth 0 and inside a node texture at depth 1 in one frame overwrites slots 1 and 2; D75 "leaf planes draw their own texture" is not met.
R2 | 11-10 | later | issue #76 part 3 (3D quad edges) is not done; drop `base_2d` from the 3D plan.
E1 | 11-10 | later | the chat `epoch` restarts at 1 when chora-voice restarts: make it unique per voice start.
A3 | 11-09 | later | verify gate 2 of 0.9 (hang watchdog, D64, `hang.rs`), then #89.
A4 | 10-30 | later | battle backlog: core memory growth in W9, W11, W12; mc dropped by the frame watchdog; silent core death under load (core-death lane merged 11-10: recheck).
A6 | 11-09 | later | bugs #53, #54, #56, #71, #70, #100, #98, #46, #36, #37, #34, #35, #38, #47, #74, #75; docs #69; `settings.set` cannot set an optional setting to none. Then #102, #63, #73, #85, #81, #90, #97, #99.
A10 | 11-09 | later | repository split: the 14 preparation steps of design/split.md section 5, no split.
A12 | 11-09 | later | split reviews due: chora-mc, chora-construct, voice; each a short design note in design/.
A16 | 11-09 | later | the worker has no xwayland-satellite, so X11 programs do not run in persona sessions; office tasks only on the reference hardware.
B2 | 11-09 | later | everyday-task gaps beyond the shared RC bar (archive/baseline.md): Wi-Fi, battery, brightness, files and USB, keyboard layout, Bluetooth.
B3 | 11-09 | later | `chora-system`: one plugin over D-Bus (NetworkManager, BlueZ, UPower, logind, UDisks2; `wpctl`) for all looks, its methods as voice tools; tests with python3-dbusmock.
B4 | 11-09 | later | core protocols: wlr gamma control, output power management, wlr data control; a keyboard layout key in `input.configure`.
S2 | 11-10 | later | cut by the owner 11-10, after 1.0: windowed chora (winit) with an agent as a 2D/3D canvas, and the Nix flake.
S3 | 11-10 | later | cut by the owner 11-10, after 1.0: effects pack, off by default (burning and exploding windows on a snapshot).

## Closed
W2-1 | 11-12 | done | the Fable judge of week 1 and the notes prune: see reflections/2026-11-14-w2-1-fable.md.
