# Backlog

One line per item, with its source. Order within a track is the order of
work. An item without an issue gets one when it is not done on the day it
is found (charter section 8). Started from charter section 15 on
2026-11-09.

## Track A

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
