# Backlog

One line per item, with its source. Order within a track is the order of
work. An item without an issue gets one when it is not done on the day it
is found (charter section 8). Started from charter section 15 on
2026-11-09.

## Track A

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

## Track C

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

1. Driver for simulated users: done 11-09 (`tests/persona/`, b28adcf).
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

1. Research of audiences, competitors and channels: done 11-09
   (`drafts/audiences.md`). Notes: awesome-wayland is archived; FOSDEM
   2027 is 30 and 31 January; Show HN and r/linux are wary of quick or
   AI-made projects, so the README note on LLM tools and the test
   history must stay visible; closest design is river 0.4.
2. Five USPs, website on `claude/website`, announcement in German and
   English (charter section 9).
