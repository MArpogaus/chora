# chora-construct style guide

These rules apply to every element that `chora-construct` draws over its
space: the voice HUD, the window list, the pane labels and the
confirmation questions. The space is a calm 3D void. The light theme has
a white glossy floor, a crisp horizon and a pale sky. The dark theme has
a black void over a black glossy floor. Panes float with a fixed turn; walking past one shows it at an angle.
All elements share one visual language: floating glass, few sizes, one
accent and calm motion.

This page gives the rules. The section `style` of
`plugins/chora-construct/construct.yaml` holds the values, as `mc.yaml`
does for `chora-mc`. The tables below show the defaults of that section.
When a value changes, change it in the file and run the contrast test of
rule 5. `plans/construct-hud.md` adds the section.

## Type

1. **Two families.** Inter sets labels and text. DejaVu Sans Mono sets
   data: commands, paths, owners, sizes and the detail of a question.
   Do not add a third family.
2. **Three sizes.** Use only these sizes, in logical pixels:

    | Role | Size | Line | Family and face |
    |------|------|------|-----------------|
    | `label` | 11 | 16 | Inter 500, capitals, letter spacing 0.16 em |
    | `data` | 12 | 16 | DejaVu Sans Mono 400 |
    | `text` | 16 | 24 | Inter 400 |

    A label names a thing or a state, for example `WINDOWS` or
    `LISTENING`. Text is what people read: a spoken sentence, a prompt,
    a window name. Do not use bold, italic or other sizes.

## Colour

3. **Four roles per theme.** `glass` fills a panel. `edge` is the
   hairline of a panel. `text` is for content and the selected row.
   `muted` is for labels, notes and rows that are not selected.

    | Role | Light | Dark |
    |------|-------|------|
    | `glass` | `[1.00, 0.99, 0.975, 0.94]`, a warm white | `[0.04, 0.05, 0.07, 0.86]` |
    | `edge` | `[0.08, 0.09, 0.12, 0.24]` | `[1.00, 1.00, 1.00, 0.16]` |
    | `text` | `[0.08, 0.09, 0.12]` (#14181E) | `[0.93, 0.93, 0.95]` (#ECEEF2) |
    | `muted` | `[0.29, 0.32, 0.38]` (#4A5260) | `[0.64, 0.67, 0.71]` (#A3AAB5) |
    | glow | `accent` at 0.10 | `accent` at 0.22 |

4. **One accent for both themes.** `accent` is `[0.15, 0.50, 0.71]`
   (#2680B4). It colours the state ring, the waveform, the selection
   mark, the glow and the edge of the focused pane. It never colours
   text. Do not use other hues.
5. **Contrast.** Check each colour on the glass over the worst
   background: a black pane and a white pane. The minimum contrast
   ratios of the defaults are:

    | Role | Light | Dark | Threshold |
    |------|-------|------|-----------|
    | `text` | 15.4 | 11.6 | 7.0 |
    | `muted` | 6.8 | 5.7 | 4.5 |
    | `accent` | 3.8 | 3.1 | 3.0 (graphics only) |

    On the bare floor and the sky, `accent` reaches 3.8 or more in both
    themes. A unit test computes these ratios from `construct.yaml` and
    fails below a threshold.

## Glass

6. **The glass panel.** A panel is a rect with the `glass` fill, a
   corner radius of 12 and a hairline of 1 in `edge`. A glow in
   `accent` surrounds it and falls to zero within 16. Panes behind the
   panel do not blur. A node texture of the pane group renders the pane
   shaders without their node textures, so a blur shows empty panes.
   The opacity keeps the text readable instead: 0.94 in the light
   theme, so that the glass over a black pane stays a clean warm white
   (#F0EDEA) and not a flat grey, and 0.86 in the dark theme.
7. **The glow is the only shader.** The rect carries the fill and the
   hairline as core props. A shader node above it draws only the glow,
   the state ring and the waveform. When the watchdog removes the
   shaders, the rect and the text stay readable. A pane label has no
   glow.

## Space and place

8. **One spacing unit of 8.** Give every padding, gap, line and margin
   as a whole number of units. A panel has a padding of 16. A row of a
   list is 24 high. A screen-fixed element keeps 32 from the edge of the
   output.
9. **Screen-fixed on `screen:OUTPUT`.** The voice HUD, the window list
   and the questions are flat nodes under the group of the space on the
   screen root. They never turn or scale with the camera. The stack from
   the back is: panes, pane labels, voice HUD, window list, question.
10. **Labels sit in the space, drawn flat.** A pane label sits 8 below
    the projected bottom edge of its pane, centred. It is a flat node at
    that point, not a projected one, so it stays sharp. Only the pane
    ahead and the pane under the pointer show a label.
11. **Text only on glass.** Text never sits on the bare space or on a
    pane. Every text stays inside its panel, within the padding, as
    the core measures it. A spoken sentence wraps at 480, or narrower on
    a small output, and shows at most two lines. Longer text keeps its
    end and starts with an ellipsis. A question wraps, and its panel
    grows with it. A text that must stay on one line, such as a row of
    the window list or a pane label (at most 320 wide), ends with an
    ellipsis.

## Motion

12. **Calm and short.** An element fades in over 160 ms, with its glow
    rising over 240 ms. It fades out over 120 ms. All motion eases out.
    Use the core `animate` for fades, not frames of the plugin.

    | Motion | Timing |
    |--------|--------|
    | the arc of `thinking` | one turn in 1.2 s |
    | the arc of `working` | one turn in 2.4 s |
    | the pulse of `question` | opacity 0.6 to 1.0, a period of 2 s |
    | the bars of `speaking` | for the `duration_ms` of the sentence |
    | a spoken sentence | stays 2 s after its end, then fades out |

    Do not bounce, scale, slide or flash.

## Contents

13. **What each element shows.**

    | Element | Contents |
    |---------|----------|
    | voice HUD | the state ring; the state as a label; while speaking, the sentence; while thinking or working, the task in `muted` |
    | question | the label `QUESTION`; the prompt in `text`; the whole detail in `data`; the label `ANSWER YES OR NO` |
    | window list | the label `WINDOWS` or `GIVE TO`; per row, the name in `text` and a `muted` note in `data`; an `accent` mark left of the selected row |
    | pane label | the name of the pane as a label, in `text`, on a glass pill without glow |

14. **The state never depends on colour or motion alone.** The label
    always names the state. Then the HUD stays clear without shaders and
    for people who do not see the accent.
15. **New elements.** A new element uses the glass panel, the three
    sizes and the four roles. A new role gets a value in the section
    `style` and a rule on this page. Before a merge, capture it at
    1920x1080 and 1280x720, in both themes, over a white and a black
    pane, and compare it with these rules.

## Panes

16. **The focused pane.** The focused pane has full opacity and an accent
    hairline with a soft glow. It is the pane flown into, else the pane
    under the pointer or ahead, as the pane label picks it. The other
    panes fade into the fog with their distance.
