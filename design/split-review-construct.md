# Split review: chora-construct

Date 2026-11-09. Subject: `plugins/chora-construct` on the session branch
`claude/construct-adopt` (81843ac), read as it is. Main has 6,494 lines; the
branch has 6,530 (adoption work: +111 -10 in `main.rs`). The rule: over
5,000 lines means a split review. Values: POLS and KISS, a few meaningful
parts or none.

## 1. What construct does, part by part

Sizes are lines on the branch. `main.rs` 3,416 (tests from 3172), `hud.rs`
1,765 (tests from 1397), `space.rs` 690 (tests from 445), `config.rs` 367
(tests from 306), `file.rs` 292 (tests from 192). About 1,000 lines are
tests. Shaders: four small `.frag` files.

| Part | Where | Lines |
|------|-------|-------|
| Types, constants, start, event list | `main.rs:25-700` | 675 |
| `Plugin` impl: events, methods, settings | `main.rs:698-875` | 175 |
| Input: press, drag, drop, list keys, pane move, resize, remove | `main.rs:931-1246` | 315 |
| Build the space and the pane nodes | `main.rs:1247-1450` | 205 |
| Adoption: place, hide, launcher start, a window found | `main.rs:1453-1700` | 250 |
| Camera: walk, look, glide, fly in, snap, fly out | `main.rs:1700-1990` | 290 |
| Frame: space and pane updates per frame, focus glow | `main.rs:1988-2190` | 200 |
| Constructs: load, leave, plugin panes, snapshot, list | `main.rs:2190-2367` | 175 |
| Window list and give menu | `main.rs:2369-2640` | 270 |
| HUD glue: list draw, styling, measure, help | `main.rs:2642-2842` | 200 |
| Voice glue: follow, answer, tick, draw | `main.rs:2842-2999` | 155 |
| Exported methods (`construct.*`) | `main.rs:3025-3118` | 95 |
| Launcher pid check from `/proc` | `main.rs:3118-3145` | 30 |
| Camera and projection math, leash, sight, place, ray hit | `space.rs:19-445` | 425 |
| HUD builders: label, list, help, glow, text probe | `hud.rs:28-932` | 905 |
| Voice state: say, status, confirm, ask, scroll | `hud.rs:933-1395` | 460 |
| Settings, keys, palette, contrast check | `config.rs:12-306` | 295 |
| Construct file format (YAML) | `file.rs:15-192` | 180 |

The plugin owns no geometry code. Panes are free: `place` and `beside`
(`space.rs:336,382`) put a pane ahead of the camera or beside another. There
is no cylinder, cube, wall or slider in the code today (grep finds none).
It uses `chora-kit` (runtime, settings, `launch`, file reads) and does not
use `chora-draw` at all.

Churn: 12 commits since 2026-10-01 touch the plugin. `main.rs` is in 9 of
them, `hud.rs` in 3, `space.rs` in 1. Nearly all are fixes in adoption and
window handling. The size sits in one file, `main.rs`, with two `impl
Construct` blocks of 1,490 and 630 lines. That is the real problem, not the
plugin boundary.

## 2. Verdicts

**Space and camera** (`space.rs`, camera and frame in `main.rs`, 1,100
lines): stay. The 3D plan (`design/3d.md`, section 5) lets the core own the
perspective, sort, ray and hit (D75). Then the construct drops its depth
sort, its ray and most per pane updates. `project`, `depth`, `hit` and the
per frame matrices (about 250 lines) will disappear. Moving them to a crate
now moves code that is due to be deleted. Do the 3D steps first, then
measure.

**HUD and its style** (`hud.rs` 905 plus `config.rs` palette 140, about
1,050 lines): stay. It builds scene nodes with glass shaders and measures
text through probe nodes in the core. `chora-draw` is an SVG and font
rasteriser (resvg) for picture themes; the HUD needs none of it, and a move
would add a dependency for no gain (POLS: a crate keeps one job). Small
action: none for `chora-draw`. The contrast check (`config.rs:154,197`) is
the rule of `docs/construct-style.md` and stays with the style values.

**Window list and keys** (about 600 lines with list input, `draw_list` and
`Action` keys): stay. It is tied to panes (fly to a window, give to a
manager). No other plugin has this list; `chora-kit` would get a user of
one. Move it to its own file in the plugin (section 4).

**Panes and adoption** (about 700 lines): stay. This is the job of a 3D
window manager (D67). One piece is generic: the check that a window belongs
to a launcher, through the parent pid in `/proc` (`main.rs:3118-3145`, two
tests at `3367,3378`). It sits next to `chora_kit::launch` (`lib.rs`), and
`chora-mc` reads `/proc/PID/stat` too (`chora-mc/src/main.rs:2580`). Move
these 30 lines with their tests into `chora-kit` when the adoption agent
has merged. Not before: that agent edits this code now.

**Constructs as YAML files** (`file.rs` 292 lines, load and save 2190-2367):
stay. It is small, has its own file and three tests, and uses `chora-kit`
for reads and writes. It will change with the geometry interface (a
construct names a geometry), so keep it close.

**Geometry code of D76**: nothing to move today, because none exists. D76
says each geometry is a small plugin or a `chora-kit` module with one
interface (slots, transforms, navigation). Advice: a `chora-kit` module,
not plugins. A geometry is pure math: a list of slots, a transform per slot,
and a step for navigation. It needs no connection, no manifest and no
process. A plugin per shape would add an IPC round trip per frame and four
manifests for 100 or 200 lines each. It stays a trait in one module; a
shape that needs a process can still be a plugin later, behind the same
trait. The cube is different: it is `chora-xp` on four virtual outputs (D76,
K11), so its geometry is four slots plus a link to D69, not a new plugin.
Write the interface at K9, together with the first user (cylinder and wall
are one ring with two radii), not before. The slider is the second user and
proves the trait.

**Voice tools** (voice state in `hud.rs:933-1395` 460 lines, glue in
`main.rs:2842-2999` 155 lines, `construct.*` methods `main.rs:3025-3118`):
stay in the plugin. The methods are the tools the voice agent calls
(`construct.load`, `construct.save` are privileged, so they ask first). The
HUD shows the agent's state and questions because the construct owns the
screen in the space. The 460 lines hold their own state machine and need no
scene knowledge: move them to `src/voice.rs` in the plugin (a file split,
not a crate).

## 3. Result

No plugin split. The plugin is one thing: a 3D window manager of panes. The
fix for its size is files and a smaller 3D core share, not new plugins.
Likely size after the steps below: `main.rs` about 900, nine files in all,
the total about 6,000 now and about 5,400 after the 3D steps drop the sort
and the ray. K9 adds the geometry module to `chora-kit` (about 300 lines
with tests), not to the plugin.

split review: chora-construct 6530 2026-11-09 (no plugin split; split main.rs into files, geometry as a chora-kit module at K9)

## 4. Steps (each small, each with tests)

1. After the adoption branch merges: move `main.rs` pieces into files with
   no change of behaviour: `voice.rs` (from `hud.rs` and the glue), `list.rs`
   (window list, give menu), `camera.rs` (walk, fly, snap), `adopt.rs`
   (adoption, launcher). Use `impl Construct` blocks in each file. Tests:
   the existing 39 unit tests pass unchanged; `cargo test -p chora-construct`
   and clippy. Check the line counts of every file stay under 1,500.
2. Move the launcher pid check to `chora_kit::launch` and use it from
   `chora-mc` too. Tests: the two moved tests, plus one in `chora-mc` that
   its stat read still works. One commit, two crates.
3. Follow the 3D plan, steps 2 to 5 (`design/3d.md`, K5): the space becomes
   a perspective group, the core hits and sorts. Delete `project`, `depth`,
   `hit` and the per frame updates in `space.rs` and `main.rs`. Tests: the
   `space.rs` tests that cover them go with the code; add the K5 check (20
   of 20 clicks on a tilted terminal pane).
4. K9: add `chora_kit::geometry` with one trait: `slots(n)`, `transform(i,
   state)`, `navigate(state, key)`. First user: the ring (cylinder and wall
   by radius). Tests: slot count and transform of a ring with 5 panes;
   removing a pane closes the gap; the wall is the ring with a flat radius;
   a scroll is one `animate` of a rotation (3D plan section 5). The
   construct file gains `geometry: ring` with a default of free panes, so
   old files load unchanged (a file test).
5. Add the slider on the same trait, the second user. Tests: scroll along
   the strip, move into a pane, leave it.
6. K11: the cube as four slots, each a face on a virtual output (D69, D76).
   Tests with the nested cells once virtual outputs exist.
7. Review again at 2026-12 or when any one file passes 2,000 lines, with the
   same rule. Do not split into plugins before step 4 shows a geometry that
   needs its own process.

Not done: `chora-draw` for the HUD, a plugin per geometry, moving the
window list to `chora-kit`. Each costs more than it saves today (KISS).
