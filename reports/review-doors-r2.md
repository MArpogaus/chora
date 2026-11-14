# Review doors r2 (claude/doors dd8669a4), opus, read-only, 11-14 19:5x UTC

Verdict: FIX (dr2-1 BLOCK, dr2-2 MAJOR). Checked fine: xp Start menu Lock fits, shared script for Log Off/Turn Off/Restart, F1 unbound in all looks, construct pane grabs scoped by in_pane, red-first holds, gate-construct-hud checks named rows.
| id | sev | file:line | text |
|---|---|---|---|
| dr2-1 | BLOCK | tests/doors/construct.sh cell_helper_none | doors step still red; construct says "Listening" with no helper (false state word). |
| dr2-2 | MAJOR | plugins/chora-construct/src/doors.rs:399, :247 | the capsule draws and takes clicks over a flown-into pane (over: self.flown.is_some()); a large window's title bar under the ~550 px strip loses clicks. No cell clicks that corner in a pane. |
| dr2-3 | MINOR | plugins/chora-mc/src/agent.rs:458-465 | critical notice pushed twice. |
| dr2-4 | MINOR | xp launch.rs:314, mc main.rs:3121, construct adopt.rs:296 | XDG_DATA_DIRS wrap written three ways; no quote escaping in xp and mc. Move to chora_kit::shared or set env on the Command. |
| dr2-5 | MINOR | xp help.rs:72, mc keys.rs:183, tiling keys.rs:267, construct config.rs | volume row spelled four ways; one keycap rule in chora_kit::glass. |
| dr2-6 | MINOR | plugins/chora-mc/src/main.rs:1337 | mc wakes every 500 ms and runs wpctl every 5 s for the volume word. |
| dr2-7 | MINOR | tests/doors/construct.sh cell_helper_door | checks only a non-empty word; construct doors never clicked in a pane. |
| dr2-8 | MINOR | plugins/chora-construct/src/doors.rs:247 | below 1024 px the strip may hit the voice island; audit only at 1280x800. |
| dr2-9 | COSMETIC | xp wm.rs, mc region.rs | state word case differs. |
| dr2-10 | MINOR | crates/chora-kit/src/shared.rs:50 | shared::script public with no outside caller (dr-4). |
