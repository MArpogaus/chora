# chora-mc style guide

These rules apply to every page of `chora-mc`: the built-in scenes, new
scenes and new kinds of content. The model is a calm book page: few
sizes, one weight, a clear hierarchy and the same space everywhere.

This page gives the rules. `plugins/chora-mc/mc.yaml` holds the values:
the styles, the colors, the scale and the page. When a value changes,
change it there, not here.

## Type

1. **Two families.** TeX Gyre Pagella sets the text that people read:
   the scene title, the panel titles, the body and the lead. Inter sets
   the frame and machine data: the header, the footer, the sides, the
   margin notes, the HUD and the style `data`. The style `mono` is for
   commands, paths and code only. Do not add a third family.
2. **Five sizes.** Use only these steps of the scale:

    | Step | Role | Styles |
    |------|------|--------|
    | -1 | margin notes, regions, HUD, code | `margin`, `hud`, `mono` |
    | 0 | body text and data | `body`, `data` |
    | 1 | panel titles | `heading` |
    | 2 | the lead of a scene, at most one | a panel `style: {step: 2}` |
    | 3 | the scene title and its number | `chapter`, `chapter_number` |

    Do not set Pagella below step 0. The labels of the regions are the
    one exception: capitals of Inter at 0.61 em, as tall as the lowercase
    of `margin`.
3. **One weight.** Pagella is always regular. Inter is regular, the
   labels of the regions are medium (500). Do not use bold or italic for
   emphasis; use place, size and space.
4. **Case and letter spacing.** The scene title is in capitals with
   0.12 em. Panel titles are in small caps (`smcp` and `c2sc`) with
   0.06 em. The labels of the regions are capitals of Inter with 0.2 em.
   Lowercase text has no letter spacing.
5. **Figures.** Body text has old-style figures (`onum`). The scene
   title and its number have lining figures (`lnum`), so that the number
   stands as tall as the capitals. Times in the margin and the clock
   have tabular figures (`tnum`). The style `data` has no `tnum`, because
   Inter then sets hyphens wide.

## Color

6. **Four roles.** `text` is for content. `muted` is for panel titles,
   labels, notes, the scene number and its bar. `faint` is for hairlines
   and for an agent that rests. Do not set text that people must read in
   `faint`.
7. **One accent.** The `accent` color is on the scene title only. The
   colors `green`, `amber`, `blue` and `red` show a state, for example of
   an agent or an error; do not use them for decoration.

## Space

8. **Baseline grid.** Each line of text is a whole number of lines of
   the grid (`scale.baseline`). Give space in lines of the grid, not in
   pixels. A panel title has one blank line under it. The scene head has
   one blank line under it.
9. **One left edge.** The scene number, the panel titles and the body
   start at the left edge of their text column. Only margin notes, such
   as times, go into the margin column, flush right.
10. **The scene head.** The number, the bar and the title stand on one
    baseline. The bar is a hairline from the baseline to the height of
    the capitals, with the same space on both sides.
11. **Hairlines, no boxes.** A rule is one pixel in `faint`, left of a
    panel. There is no rule at the left edge of the text block and no
    rule beside an empty panel. Do not add boxes, fills or shadows.

## Layout

12. **Titles never clip.** A panel title has one to three words. A
    longer title wraps and moves the text of its panel down. A panel
    shows whole lines only; it never cuts through a line.
13. **Narrow outputs.** 1280 logical pixels is a main target, not an
    edge case. A page shows only as many columns as fit at `min_panel`;
    the other panels wrap into new rows in reading order. A row is never
    lower than `min_panel`. When the page is longer than the output, it
    scrolls with the wheel.
14. **Columns follow the content.** The agents scene has as many
    columns as agents: one agent takes the whole width, two take half
    each. A new scene with a varying number of items does the same.

## New scenes and contents

15. **Use the named styles.** A scene or a content uses `body`,
    `heading`, `data`, `margin` or `mono`. It changes a style only with
    `base` and a color or a step of the table above. Do not set a family,
    a size in pixels or a letter spacing in a scene file. A new role gets
    a named style in `mc.yaml` and a rule on this page. Before a merge,
    capture the scene at 1920x1080 and at 1280x720, in the dark and the
    light theme, and compare it with these rules.
