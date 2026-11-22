# review mc-focus r1 (PM review, 24 lines; builder: lane session_016iS3kNytLH7EhG5sJybjcm)

Base 6b01c976..e22f824d: crates/chora-kit/src/runtime.rs 3, plugins/chora-mc/src/main.rs 25.
- Red first: 80d7802d adds a_focus_node_of_another_client_leaves_mc_no_holder; it asserts EVENTS holds "focus.node", so it fails before e22f824d. pass
- Fix: "focus.node" in EVENTS (21), handled by focus.hear (Focus::away); refresh and Plugin::refresh docs match what the core sends. pass
- Scope: mc and one doc comment, as the brief says. pass
- Other looks: construct, mc, tiling, xp subscribe (lane report). pass
- Gate: full --jobs 2 all PASS but acts: xp I6 'Starting Notepad' notify-card-body 1.04:1, a foreign red (no xp file touched); second sighting of xp-38 (co-keys r3 first). Lane XP-38.
Findings: 0. Verdict (computed): MERGE-OK; joins the next train.
