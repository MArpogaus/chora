# Split review: chora-mc

Date 2026-11-09. Rule: over 5,000 lines gets a split review. Values: POLS, KISS.
Read: `plugins/chora-mc/src/*.rs`, mc-style, configuration (mc), D51 D65 D66 D73 D76 D77,
`docs/xp-split.md`, `git log --since=2026-10-01`.

## Size

6,704 lines in 7 files. About 877 are tests, 145 are `testing.rs` (a fake core),
so about 5,680 are product code. Outside the binary: `mc.yaml` 287, 4 collector
scripts 639, 6 scenes 70, 2 shaders 88.

## What mc does, part by part

One client `mc`, one `struct Mc` (`main.rs:244`, about 45 fields), four `impl Mc`
blocks that all use these fields.

| Part | Where | Lines | Job |
|------|-------|-------|-----|
| Shell | `main.rs:1-486, 686-975` | ~620 | args, params, `Mc::new`, events, dispatch |
| Method table | `main.rs:508-685` | 178 | one entry per method, privileged marked (D62) |
| Build, recolor | `main.rs:1161-1487` | ~330 | root group, theme fade, work area, `--manage-windows` |
| Pages | `main.rs:1488-1978` | ~490 | panels, chapter head, grid, whole lines, scroll, shaders |
| Show, windows | `main.rs:1979-2342` | ~365 | show a scene, launch, `adopt` by token, dock, fit |
| Text contents | `main.rs:2343-2566` | ~225 | `content.put`, `hud.say`, typing, key actions |
| Regions | `region.rs:1-1264` | 1,264 | rules, slots, boxes, cross-fade render, toggle, solo, clocks |
| Agents, notify | `agent.rs:1-712` | 712 | `agent.set`, agents scene, `notify.push` (:444), `voice.confirm` (:476-678) |
| Scene order | `manage.rs:1-314` | 314 | numbers, overview, rename, move, `window.give` |
| Settings, style | `config.rs:1-483` | 483 | state signs, themes, `Style` and `Type` (:123-232), scale |
| Scene files | `scene.rs:1-324` | 324 | YAML scene, panel, grid, built-ins |

The voice agent (`voice.*`, D65) and the collectors fill it; mc reads no sources.

## What changes together

Since 2026-10-01 there are 12 commits without merges. 11 touch only `main.rs` (core
API changes: injected keys, work area, pane focus, privileged methods). One
(4e011f5) touches `agent.rs`, `main.rs`, `manage.rs`. 173581b, the first in the
window, touches all files. No commit changes `region.rs`, `agent.rs` or `config.rs`
alone. mc changes as one unit, driven by the core. A split would turn each such
commit into changes in two or three plugins.

## Verdicts

The XP split worked because `wm.rs` mixed jobs that flavors want apart (taskbar,
notifications, desktop), and each block got its client, settings and assets and
calls no other block (`xp-split.md`, Rules). Nothing in mc is wanted without the rest.

**Regions** (`region.rs`, 1,264): stay.
- Coupling: `region.rs:362-1197` is `impl Mc` over `scene`, `current`, `docks`,
  `toggled`, `agents`, `solo`, `confirming`. The page asks `side_widths`,
  `drawer_box`, `window_box` on every build, so the page never moves when a region
  shows. Across a process that is a call per build and a race on resize.
- Reuse: none. `chora-draw` has no region idea; `chora-construct/src/hud.rs` is a 3D
  HUD of another design. The kit part (runtime, settings, give) is used already.
- A second client would need the docked windows too (`adopt`, `main.rs:2148`), which
  breaks "blocks meet only through the core". D77 gives no release gain.

**Agents view** (`agent.rs:198-440`): stay.
- `chora-assistant` draws a figure and balloon from an asset folder (D73); mc draws
  text on the calm page. Both subscribe to `voice.*` on purpose: each is a view of D65.
- The agents scene is the normal panel grid (`fill_agents`, `agent.rs:329`) and feeds
  the header state. Cut out, two files must agree on one `Agent` type.

**notify.push and margin notes** (`agent.rs:72-86, 183-195, 444-474`, ~55): stay.
- `chora-notify` is the freedesktop server with cards; `chora_kit::notify` is its
  client (new today, used by `chora-xp`). `notify.push` is a text line in the right
  column, newest on top, 12 lines (`MAX_LINES`), critical in red. A card on that page
  breaks mc-style rule 11 (no boxes) and needs the session bus. POLS: the margin note
  is the notification of mc. If an agent wants a card, it calls `chora_kit::notify`.
- Too small to be a part. Do not grow it.

**voice.confirm wire types** (`agent.rs:86-165`, ~80): move to `chora-kit`.
- `Confirm`, `Keys`, `Say` and the `voice.confirm.answer` call exist three times:
  `chora-mc/src/agent.rs:86-165`, `chora-construct/src/hud.rs:933-975`,
  `chora-assistant/src/voice.rs:216-272` (+ `assistant.rs:502`), with the same
  `serde(default)` fields and the id as text or number.
- Three users now, so a module `chora_kit::voice` meets D73 (code moves with a real
  user). Drawing stays: drawer buttons, 3D HUD and balloon differ.

**Docked terminal and solo app** (`region.rs:406-440, 1106-1133`, `main.rs:2148-2342`,
~330): stay, watch `chora-xp`.
- It is the app-in-a-region feature: start by token (`chora_kit::launch`, `main.rs:2069`),
  adopt at `surface.created`, park until the size fits, hand the keyboard. It needs
  regions and scenes. `window.give` is shared already (`chora_kit::give`, `manage.rs:130`).
- Overlap: `chora-xp/src/launch.rs` (465) also waits for a window by token and uses
  `chora_kit::alive`; mc does it in `launch_scene` and `look_at_launched`
  (`main.rs:2069-2118`, ~50). The XP code goes to `chora-panel` in X4. Judge a shared
  helper in `chora-kit` then, with two real users. Not now.

**Typography and style** (`config.rs:123-232`, `mc.yaml`, mc-style, ~110): stay.
- `Style`, `Type`, `Scale` build `PropsUpdate` for core text nodes (family, OpenType
  features, letter spacing). `chora-draw` rasterizes SVG and text into image nodes
  (`text.rs`, `theme.rs`). Different mechanism; mc does not use `chora-draw`.
- No second user: `chora-construct` has its own `Style` (`config.rs:126`: unit, radius,
  glow, two families) with no baseline grid or features.
- Only `Theme`, `Themes`, `Colors` (`config.rs:63-111`, ~50) look like
  `chora-construct/src/config.rs:62-117`. The shapes differ (RGBA with fallback to the
  built-in file, RGB). Note it, wait for a third user.

**Others**: scene files and order (`scene.rs`, `manage.rs`, 638) are the data model of
mc: stay. Collectors and shaders are outside the Rust code already; the collectors
ship apart (`crates/chora/Cargo.toml:50`).

## Result

One coherent job (the voice-first page with regions), one state, one client, one
commit stream. The only candidate with a second user is the voice wire types. Plugins
for regions, agents or style would add IPC for geometry and state without reuse
(D73, D77, KISS).

split review: chora-mc 6704 2026-11-09 (no split; share the voice wire types in chora-kit)

## Steps (no split advised; small follow-ups)

1. Add `crates/chora-kit/src/voice.rs`: `Say`, `Confirm`, `Keys`, answer params.
   Tests: parse `voice.confirm` with a numeric id, with `physical` and keys, with
   `done`; build the answer for yes and no. No behavior change.
2. Switch `agent.rs` of mc (delete :86-165). Tests: `cargo test -p chora-mc`,
   `tests/gate-0.9-daily.sh` unchanged.
3. Switch `chora-assistant` and `chora-construct`, one commit each. Tests: their unit
   tests; the balloon and HUD captures stay equal.
4. After X4: compare `launch_scene` with the panel code; extract into `chora-kit` only
   if they agree. Tests: a started app, an app that exits without a window, a refused start.
5. Optional, inside the crate: cut `main.rs` (2,584 product lines) into `build.rs`
   (:1161-1978) and `window.rs` (:1979-2342). Code moves only; `cargo test -p chora-mc`
   and `tests/battle/w11.sh` stay green.

Review again if a second plugin needs the `region.rs` rules: then they move to
`chora-kit` with that plugin as first user.

## Summary

1. chora-mc has 6,704 lines (about 5,680 product), one process, one state.
2. 11 of 12 recent commits touch only `main.rs`: the parts change together.
3. Regions, agents, notify.push, docked app and style stay: coupled through `impl Mc`,
   no second user, no overlap with `chora-draw` or `chora-notify`.
4. One real duplication: voice wire types in mc, construct, assistant go to `chora-kit`.
5. Launch watch is judged after X4; an optional `main.rs` cut stays inside the crate.
