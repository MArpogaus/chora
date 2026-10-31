# chora-construct: the voice HUD and the style (step K3, F28)

Status: built. The rules and the values come from
`docs/construct-style.md`; this plan does not repeat them. Read that page
first, then `plans/construct-requirements.md` (F26 to F31) and
`docs/voice.md` section 10.

Another agent changes the drag, the hand-over and the floor shader
(white glossy floor, crisp horizon). Rebase on `dev` before step 1 and
keep the space colours (`Palette::sky`, `horizon`, `ground`, `dust`) as
that work leaves them. This plan touches only the elements over the
space.

## 0. What is there

- `src/main.rs` builds the space on `screen:NAME` in `space_nodes`: the
  group `construct` with `construct-sky`, `construct-fog` (shader at 1/4
  size), `construct-mirrors`, `construct-panes`, `construct-windows` and
  `construct-look`. A window that the camera flew into moves to the top
  of `construct-windows`, above the panes.
- `draw_list` draws the window list as a group `construct-list` at the
  end of `construct`: a rect in `Palette::paper` at 0.96, a label of 10,
  rows of 15 and notes of 12, a hairline mark in `text`. It removes and
  creates the whole group on each change.
- `frame` sends the pose on each step of a glide (16 ms) and sorts the
  panes. `serve` waits for events, with a timeout only during a glide or
  while plugin panes wait.
- `config.rs`: `Theme::palette` holds the space colours and `paper`,
  `text`, `muted` of the list, in code. `construct.yaml` has no style.
- `EVENTS` has no `voice.*`, `client.*`, `effects.dropped` or
  `layers.dropped`. Nothing handles the watchdog.

## 1. Settings

Add to `construct.yaml`, with comments, and to `Config`:

```yaml
# The voice HUD at the bottom of the output (F28). It hides the dot of
# the voice agent while chora-construct runs.
voice_hud: true
# The elements over the space: docs/construct-style.md has the rules.
style:
  unit: 8
  radius: 12
  glow: 16
  font: Inter
  mono: DejaVu Sans Mono
  accent: [...]
  themes:
    light: {glass: [...], edge: [...], text: [...], muted: [...], glow: 0.10}
    dark:  {glass: [...], edge: [...], text: [...], muted: [...], glow: 0.22}
```

- Copy the values from the tables of `docs/construct-style.md`. Do not
  repeat them in code: `Theme::palette` loses `paper`, `text` and
  `muted`, and `draw_list` reads the style.
- The sizes (11, 12, 16), the line heights and the timings are rules of
  the guide and stay constants in a new `src/hud.rs`. A setting for them
  is not needed now.
- `check`: each colour component from 0 to 1, `unit` from 4 to 16,
  `radius` and `glow` from 0 to 32.
- `docs/configuration.md`: two rows in the settings table, `voice_hud`
  and `style`, and a short subsection "The voice HUD".

## 2. Node tree

All new nodes are children of the group `construct`, after
`construct-look`, in this order from the back. Each group has
`input: pass`, except the list, which takes clicks as now.

```text
construct (screen:NAME)
├─ construct-sky, -fog, -mirrors, -panes, -windows, -look   (as now)
├─ construct-label            group, translate to the label place
│  ├─ construct-label-glass   rect: glass, radius 12, border 1 edge
│  └─ construct-label-text    text: label style, text colour
├─ construct-hud              group, translate to the HUD place
│  ├─ construct-hud-glass     rect: glass, radius 12, border 1 edge
│  ├─ construct-hud-glow      shader glass.frag: glow, ring, bars
│  ├─ construct-hud-state     text: label style, muted
│  └─ construct-hud-say       text: text style, max_width 480
├─ construct-list             (as now, restyled, section 5)
└─ construct-ask              group, top centre
   ├─ construct-ask-glass     rect: glass, radius 12, border 1 edge
   ├─ construct-ask-glow      shader glass.frag: glow only
   ├─ construct-ask-label     text: QUESTION
   ├─ construct-ask-prompt    text: text style, max_width 640
   ├─ construct-ask-detail-N  text: data style, one node per 4 KiB
   └─ construct-ask-hint      text: ANSWER YES OR NO
```

One new shader, `plugins/chora-construct/glass.frag`, written to the
runtime directory with `shader_file` like the others. Its node is the
panel plus `glow` on each side, so the glow has room. It is transparent
inside the panel, except for the ring and the bars, and draws:

- the glow: `accent` times the glow alpha, falling from the panel edge to
  zero at `glow`, outside a rounded rect of `radius`;
- the ring, when `ring.z > 0`: a circle of diameter 40 and stroke 2,
  centred at `ring.xy`, with the mode of section 3;
- the bars, while speaking: five bars of 2 wide with gaps of 4 inside
  the ring, height from `0.3 + 0.7 * level * noise(time, bar)`.

Uniforms (own, at most 16; this shader needs 5): `accent` vec4 (rgb and
glow alpha), `muted` vec3, `box` vec4 (panel width, height, radius,
glow), `ring` vec4 (x, y, diameter, mode), `level` float. `alpha`,
`size` and `time` come from the core. Use `precision highp float` for
the noise (api.md, shader hints).

The fill and the hairline are core props of the rect, not shader work.
So after `effects.dropped`, the panel and its text stay (rule 7 of the
guide).

## 3. The voice HUD (F28)

### Events and claim

- Add to `EVENTS`: `voice.*`, `client.connected`, `client.disconnected`,
  `effects.dropped`, `layers.dropped`.
- Handle `Event::Emitted { event, client, data }` for `client ==
  "voice"`, as `Mc::follow` in `plugins/chora-mc/src/main.rs` does.
- Claim the indicator as chora-mc does (`claim_indicator`, `claim_due`):
  call `methods.call {client: voice, method: indicator.claim}` at the
  start, on `client.connected` and `client.disconnected` of `voice`, and
  on the first `voice.*` event after a failed claim. Claim only with
  `voice_hud: true`. Port the mc test
  `mc_claims_the_indicator_of_voice_once_per_connection`.
- On `client.disconnected` of `voice`, remove `construct-hud` and
  `construct-ask`. The HUD shows only after the first `voice.state`.

### States

`voice.state {state, task, said}` carries the states of mc
(`services/voice/src/core.rs`, `State::agent`): `asleep`, `listening`,
`thinking`, `working`, `question`, `speaking`. An event with a `name`
other than `voice` is a sub-agent (coder, ACP); the HUD ignores it in
v1. `voice.remove` also belongs to sub-agents.

| State | Ring (`ring.w`) | Label | Text line |
|-------|-----------------|-------|-----------|
| `asleep` | 0: thin, `muted`, still, HUD at opacity 0.6 | `ASLEEP` | none |
| `listening` | 1: full, `accent`, still | `LISTENING` | none |
| `thinking` | 2: an arc of 90°, one turn in 1.2 s | `THINKING` | `task`, muted |
| `working` | 3: an arc of 90°, one turn in 2.4 s | `WORKING` | `task`, muted |
| `question` | 4: full, pulse 0.6 to 1.0 over 2 s | `QUESTION` | none; the question panel shows |
| `speaking` | 5: full, with the bars | `SPEAKING` | the sentence of `voice.say` |
| other | as `listening` | the state in capitals | none |

Put this table in `hud.rs` as one `fn mode(state: &str) -> (f64,
&'static str)` and test each row.

### The waveform

The voice agent emits no audio levels: `voice.say {state: speaking,
text, duration_ms}` is the only signal. So the bars are a cheap
animation, not real levels:

- On `voice.say`, set `level` to 1 and a deadline `say_until = now +
  duration_ms`. `serve` folds the nearest deadline into its timeout.
- At the deadline, set `level` to 0. 2 s later, fade the sentence out
  with `animate` (120 ms, opacity 0).
- The bars move with the core `time` in the shader; the plugin sends no
  frames for them.

A real level needs an event `voice.level` of the agent, at most a few
per second under the limit of D65 (32 events per second). Not in K3.

### The spoken sentence

- `voice.say.text` replaces the text line. It wraps at 480 and keeps at
  most two lines. A longer sentence keeps its end, with a leading
  ellipsis, as `hud_tail` in `plugins/chora-mc/src/main.rs` does.
- The panel width follows the text: padding 16, ring 40, gap 16, text,
  padding 16. Without a text line the panel is 72 by 72.

### Place

- Over the space: bottom centre, 32 above the bottom edge of the output.
- Flown into a pane (`self.flown.is_some()`): only the ring and the
  label, at the bottom right, 32 from both edges; the sentence shows
  only while speaking. On fly-out it goes back to the bottom centre.
- Recompute the place in `build` (output size) and in `snap` and
  `unsnap`, not in `frame`.

### Questions

- `voice.confirm {prompt, detail, id}` opens `construct-ask`, also while
  flown in, above the window list.
- `detail` shows whole, in the data style, wrapped at 640. A text node
  holds at most 4 KiB, so split the detail into nodes of at most 4 KiB at
  line ends. When the panel is taller than the output minus 64, the
  wheel over it scrolls it; nothing is cut.
- `voice.confirm {id, done: true}` closes the question of that `id`
  only. A new question replaces an open one. Port the mc test
  `a_confirmation_shows_until_its_answer`
  (`plugins/chora-mc/src/agent.rs`).
- The answer is by voice. The HUD has no buttons in K3: the method
  `confirm` of the agent is privileged.

## 4. Pane labels

- One label at a time: the pane under the pointer, else the pane ahead
  (`pane_at`, `pane_ahead`). Hide it while flown in, while the window
  list is open, and when the pane is not `shown` (`space::sight`).
- Place: project the bottom centre of the pane with `pane_matrix`
  (`u = w / 2`, `v = h`), 8 below it, centred, rounded to whole
  pixels. Add a pure `space::project(m, u, v)`; the tests have one
  already.
- Update the transform in `frame`, and the text and the pill width only
  when the pane changes. The width comes from the text: estimate it as
  `chars * 11 * 0.72 + 16 * 2` and test it by a capture; the core gives
  no text metrics before a `scene.query`.
- `pointer.moved` over `construct-look` already arrives; store the last
  position for `pane_at`.

## 5. The window list restyled

- Replace the rect `construct-list-ground` with the glass rect and a
  `glass.frag` glow node of the same recipe.
- Sizes: label 11 (was 10), names 16 (was 15), notes 12 in DejaVu Sans
  Mono. `LIST_LINE` stays 24, the padding 16.
- The selection mark: 2 wide, `accent`, at the left padding. The selected
  name in `text`, the others in `muted`.
- Fade in with `animate` over 160 ms on open; no fade on a redraw of the
  rows. Keep the remove-and-create of `draw_list` for the rows, but keep
  the group and its glass, so that the fade runs once.

## 6. Cost limits

- **Uniforms.** At most 16 own uniforms per shader node (`scene.rs`,
  `check` of uniforms) and at most 4 node textures. `glass.frag` uses 5.
  `fog.frag` uses 15 today, with the shadows `s0` to `s7`; do not add
  uniforms to it in this step. A unit test counts the `uniform` lines of
  each `.frag`, minus `alpha`, `size`, `time` and `tex`, and fails above
  16.
- **Layer budget.** The core draws at most 64 Mi pixels of layers per
  frame (api.md). Every rect with a radius or a border and every shader
  node is a layer. At 1920x1080 and scale 1 the new layers are about:
  HUD 600x104, label 300x32, list 592x430, question 752x1016. Together
  they stay under 1.5 Mi pixels, also at scale 2 (6 Mi), beside 8 panes
  of 1920x1080 (16.6 Mi) and their mirrors. A unit test sums the node
  sizes that `hud.rs` builds at 1920x1080 and fails above 1.5 Mi.
- **Text raster.** 16 MiB per output scale and node; split the detail
  (section 3) keeps each node small.
- **Watchdog.** On `effects.dropped` with `client == "construct"`, set a
  flag `plain`: build no more `glass.frag` nodes and send no uniforms to
  them, until `settings.reload` or a restart. Log it once. The rects
  and texts stay, and the label names the state (rule 14). Do not add the
  shaders again at once: a second slow phase can disconnect the client
  at the owner budget (D57).
- **`layers.dropped`.** Log the nodes. If a HUD node is among them, hide
  the glow nodes until the list is empty again.
- **Frames.** The HUD sends no commands per frame. Only the pane label
  moves with `frame`. The fades run in the core with `animate`.
- **0.10 renderer.** Today a layer renders in every frame, so `time` in
  `glass.frag` costs nothing extra. When the damage renderer of 0.10
  lands, check that a HUD at rest (modes 0, 1, 4 without `time`) costs
  no frame; if not, switch the shader off by `ring.w` and `level` = 0
  and skip `time` there.

## 7. Code layout

- `src/hud.rs` (new): the constants of the guide, `mode`, the place of
  the HUD and the question, and pure builders that return
  `Vec<Command>` for the HUD, the question, the label and the list glass.
  No connection, so the unit tests need no fake core.
- `src/config.rs`: `Style`, `voice_hud` and the contrast check.
- `src/main.rs`: events, the claim, the deadlines in `serve`, and the
  calls of the builders.

## 8. Tests and gates

Unit tests (`cargo test -p chora-construct`):

- the default file is valid, with the new keys;
- contrast: from the default `style`, compose `glass` over black and over
  white for each theme and check `text` 7.0, `muted` 4.5 and `accent`
  3.0 (WCAG relative luminance); a broken value fails;
- `mode` for each state and an unknown one;
- a `voice.state` with another `name` changes nothing;
- the question: a `done` of another id keeps it, of its id closes it; a
  detail of 10 KiB gives three text nodes and loses no byte;
- the claim: once at the start, again after `client.connected`, and no
  more after the first event;
- the uniform count of each `.frag` and the layer sum of section 6;
- the label place of a pane straight ahead is centred under it.

Gate `tests/gate-construct-hud.sh` (headless, `gate-lib.sh`, as
`gate-construct-windows.sh`):

1. Start the construct flavor and a fake voice client: a short Python
   script `tests/fake-voice.py` that connects as `voice`, exports
   `indicator.claim`, counts the calls and emits `voice.state`,
   `voice.say` and `voice.confirm` from its stdin. Check one claim.
2. Emit each state; `scene.query` shows `construct-hud-state` with the
   label of the table.
3. Emit `voice.say` with `duration_ms: 800`; the text shows, and 2.8 s
   later its opacity is 0.
4. Emit a question with a detail of 6 KiB; the joined detail nodes equal
   it. Emit `done` of another id: it stays. Emit `done` of its id: it
   goes.
5. Restart the fake voice: a second claim arrives.
6. With `CHORA_FRAME_BUDGET_US=1`, wait for `effects.dropped`; the HUD
   rect and the label stay visible, and no `glass.frag` node comes back.
7. Fly into a pane: the HUD moves to the bottom right; fly out: back.

Checks before each commit: `cargo fmt --check`, `cargo clippy
--all-targets --locked -- -D warnings`, `cargo test --locked` (through
`micromamba run -n chora`), `shellcheck` of the gate, and
`uvx --with mkdocs-material mkdocs build --strict`.

## 9. Screenshots

The gate keeps them in `SHOTS` as `construct-hud-THEME-SIZE-CASE.png`,
for both themes and for 1920x1080 and 1280x720:

- `listening`, `thinking` and `speaking` with a two-line sentence, over
  the bare floor;
- `speaking` over a white pane and over a black pane (the worst cases of
  rule 5);
- a question with a long detail, over a pane;
- the window list open, over a pane;
- a pane label, ahead and under the pointer;
- flown into a terminal: the compact HUD at the bottom right;
- after `effects.dropped`: the plain HUD.

Compare each capture with `docs/construct-style.md` before the merge,
and send them to the XP test session as usual.

## 10. Order of commits

1. `feat(construct): read the style section` (settings, contrast test,
   list colours from the style; no visible change but the sizes).
2. `feat(construct): draw glass panels` (`glass.frag`, the list
   restyled).
3. `feat(construct): show the voice state` (events, claim, ring, label).
4. `feat(construct): show the spoken sentence and the waveform`.
5. `feat(construct): show the questions of the voice agent`.
6. `feat(construct): label the pane ahead`.
7. `fix(construct): keep the HUD plain after the watchdog`.
8. `test(construct): gate the voice HUD` and `docs(construct)` for
   configuration.md and the status line of
   `plans/construct-requirements.md` (F28 built; N6 points to the
   construct guide).
