
## r1

Branch claude/mc-focus, head e22f824d (pushed; origin/claude/autonomous-dev-session-3w-i33pg7 already merged, nothing to merge).

- Red 80d7802d: `a_focus_node_of_another_client_leaves_mc_no_holder` fails on 6b01c976: `assertion failed: super::EVENTS.contains(&"focus.node")` (main.rs:1965).
- Green e22f824d: "focus.node" in EVENTS (21), `Plugin::refresh` docs (runtime.rs, main.rs) say the core sends `focus.node` to looks that listen. `cargo test -p chora-mc`: 190 passed. clippy --workspace --all-targets -D warnings clean.
- Looks that subscribe to "focus.node": chora-construct (main.rs:761), chora-mc (now), chora-tiling (main.rs:469), chora-xp (wm.rs:139). chora-assistant and chora-notify do not, and hold no keyboard node: no finding.
- Gate (`tests/merge-gate.sh --jobs 2`): all cells PASS except `acts` FAIL (1604s), a foreign red, not mc: `FAIL: xp I6: in act 'a click on Notepad starts it to see it go': on HEADLESS-1 'Starting Notepad' (notify-card-body) has the contrast 1.04:1 ((119, 179, 75) on (115, 176, 72)), the guide wants 4.5:1`. Not rerun.
