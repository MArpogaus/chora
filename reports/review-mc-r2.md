# Review mc r2 (sonnet, read only)

Base c2c9c353 (r1 head), head 66ca64a9 (reviewed sha; 7 own commits plus the merged session branch 655429b6, voice/secfix files not in scope). Verdict: MERGE-OK (no open BLOCK or MAJOR).

| # | sev | where | finding |
|---|---|---|---|
| 1 | holds | hud.rs Panel.size (+check 14..48), config.rs Line.size, assistant.rs plain(), hud.yaml `size: 19` | mc-34 MAJOR: size is optional, default None keeps Font::Ui at 11 px and MAX_PLAIN 420, so xp/tiling/construct draw as before (only mc's hud.yaml sets it). Sized sentence uses the chat Face at 19 px, wrapped at 640. Gate cell sofa `answer` passes (node height >= 30). Unit tests pass (hud size range, sentence larger). |
| 2 | holds | c884ed79, e90abaaf before 0fbb751e | mc-34 red first: the unit tests c884ed79 and cell `answer` e90abaaf precede the fix. |
| 3 | holds | mc.yaml, region.rs 514-527, 1289, 1352-1380 | mc-35 card: urgent is a right-column card (slot -3, body size, red, wrapped, drawn close mark, lead room in max_width and x); where the right side does not show it falls back to the footer. Paths `urgent` passes (card in column, whole text twice, >= 14 px, press takes it away, arrows work). |
| 4 | holds | region.rs fit/lines_of/shed_updates | mc-35 overflow: notify items shed from the last, title goes with no entry left, the rest never sheds. Sofa `notices` passes (Chat door above KEYS rule, card whole, nothing in footer). Unit tests pass. |
| 5 | holds | 3337875f before 3c19f217 | Red first for mc-35: paths `urgent` and sofa `notices` changed in 3337875f and FAILed there (per builder; not rerun). The new unit tests arrive with the fix commit, which the brief allows (gate cell first). |
| 6 | holds | region.rs 2029-2043, mic.rs 61-66 | LOW 6 (doc comment of `lead` back above `lead`) and LOW 7 (mic.rs comment wrapped) done. |
| 7 | holds | tests/gate-injected-presses.sh | The hit id moved to mc-r1-right-urgent-hit; matrix gate follows the card. Only the id changed; the click and clear checks are intact. |
| 8 | holds | region.rs, hud.rs | Shared code: `urgent` loses the `is_line` condition, `shed` and `lead` default off/0 for items without a mark, so other items and looks are unmoved. No key handling touched; keyboard focus code (take_free_keyboard) is unchanged. No new keys reach clients. mc `cargo test` 125 passed. |
| 9 | MINOR | region.rs fit | No cap on a notice's length: only `notify` items shed. A long card plus many helper rows can still push the door onto the rule (builder-declared open). Not a fault of the brief's case (helper row plus two long notices). |
| 10 | LOW | mc-paths `urgent` | Dark only (gate has no THEME); the light card is in sofa `notices`. Builder-declared. |
| 11 | LOW | assistant.rs | `said` of voice.state is not drawn (hud.yaml names no such panel); allowed by the brief. |
| 12 | LOW | rebuild_pages (r1 LOW 8) | Left as it is; accepted. |
| 13 | COSMETIC | audit.rs comments | The doc comment of the close-mark test wraps short (3 lines of 66 chars); fine. |

## Rerun (CARGO_TARGET_DIR=/tmp/rev-target, 20G free at start, removed at the end)
- `cargo build --workspace` ok.
- `cargo test -p chora-mc`: 125 passed. Test hud a_panel_names...: pass. Test chora-assistant a_sentence_with...: pass.
- tests/gate-mc-paths.sh (all cells incl. urgent): pass once.
- tests/gate-mc-sofa.sh (dark, all cells incl. answer, notices): pass once. Note: it first failed with "no HUD folder named neutral" because the build dir was not at <repo>/target (folders.rs finds shipped huds relative to a `target` dir); fixed in the rerun with XDG_DATA_DIRS pointing at plugins/chora-assistant/huds. An environment fact, not a product fault.
- Not run: tests/doors/mc.sh (not changed by r2; not listed in mc-r2.md), light theme, merge-gate.sh (builder's pass accepted).
- r1 still holds: its three reds and cells (remote, wide, door, status, empty-head) pass in the sofa run; mc unit tests green.
