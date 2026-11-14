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
   commands, paths and code only. Do not add a third family. The family
   of `body` is a list, `TeX Gyre Pagella, DejaVu Serif`, so that a machine
   without Pagella shows a serif and not a second sans.
2. **Five sizes.** Use only these steps of the scale:

    | Step | Role | Styles |
    |------|------|--------|
    | -1 | margin notes, regions, HUD, code | `margin`, `hud`, `mono` |
    | 0 | body text and data | `body`, `data` |
    | 1 | panel titles | `heading` |
    | 2 | the lead of a scene, at most one | `lead` |
    | 3 | the scene title and its number | `chapter`, `chapter_number` |

    Do not set Pagella below step 0. The labels of the regions, the
    running head and the clock are the one exception: capitals of Inter
    at 0.75 em, the size of `margin`. Nothing on the page is under 14
    pixels, in the header as in the margin column; the gate
    `tests/gate-mc-column.sh` checks every text. The scale base is 19
    pixels on a baseline grid of 28.
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

6. **Three roles.** `text` is for content. `muted` is for panel titles,
   labels, notes, the scene number and its bar. `faint` is for hairlines
   and for an agent that rests. Do not set text that people must read in
   `faint`. Text colors keep their contrast on the page: `text` at
   least 7:1 and `muted`, `accent` and the four state colors at least
   4.5:1, in both themes; a test of the default checks it.
7. **One accent.** The `accent` color is on the scene title and on the
   state of the voice agent only. The
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
    rule beside an empty panel. The one other rule runs between the text
    block and the margin column, in the middle of the gap. Do not add
    boxes, fills or shadows.

## Layout

12. **Titles never clip.** A panel title has one to three words. A
    longer title wraps and moves the text of its panel down. A panel
    shows whole lines only; it never cuts through a line.
13. **Narrow outputs.** 1280 logical pixels is a main target, not an
    edge case. A page shows only as many columns as fit at `min_panel`;
    the other panels wrap into new rows in reading order. A row is never
    lower than `min_panel`, except a row of only empty panels with their
    words: it is as high as the words, so that each message shows. When the page is longer than the output, it
    scrolls with the wheel.
14. **Columns follow the content.** The agents scene has as many
    columns as agents: one agent takes the whole width, two take half
    each. A new scene with a varying number of items does the same.

## New scenes and contents

15. **Use the named styles.** A scene or a content uses `body`,
    `heading`, `lead`, `data`, `margin` or `mono`. It changes a style only with
    `base` and a color or a step of the table above. Do not set a family,
    a size in pixels or a letter spacing in a scene file. A new role gets
    a named style in `mc.yaml` and a rule on this page. Before a merge,
    capture the scene at 1920x1080 and at 1280x720, in the dark and the
    light theme, and compare it with these rules.

## The margin column

16. **The key list.** The column ends with six rows of a key and its
    words under a hairline and the head `Keys`: the key in `text` with
    weight 500, the words in `muted`, both in the style `margin`, on the
    baseline grid. It shows until the user hides the column with
    `Super+n`; `Super+F1` shows all keys in its place. The list is on
    `Super+F1`, not on `F1`, because a plain F1 belongs to a program in a
    scene and needs the grant `input.grab-any`.
    The full list goes by task, in the words of the six rows. It names
    the lock, the power menu, the screenshot and the volume keys too;
    `Super+slash` is a silent alias of `Super+F1`.
    Two lines of doors stand under the key list: `Programs`, `Lock`,
    `Power`, `Screenshot` and the volume, each in `text` and clickable.
    They stay while the chat is open.
    On a page the row of the page names the plain arrows, `← →`: there
    Right and Left turn the page without `Super`, and `Esc` goes back one
    step: it closes the full list, else it shows the page that showed
    before. In a scene of a program these keys belong to the program, and
    the row names `Super` with the arrows. With `--output`, beside
    other window managers, a page takes no keyboard: the node with the
    keyboard would draw their new windows to mc.
17. **The word Mute.** Once the voice agent told its state, the first
    line of the column holds the word `Mute` in the style `margin`: at
    the left edge beside the state word of the HUD host, flush right in
    the line of the state word where mc draws it. A click of the person
    mutes the microphone. While it is muted the badge of the host turns
    it on; without the host the word is `Turn microphone on`. When the
    agent does not switch, or the press was injected, the line says so.
18. **One focus mark.** What a click acts on shows a bar of 2 by 20
    pixels in `text`, 12 pixels left of it, while the pointer is over it:
    no underline, no fill and no change of color. A word flush right in a
    head, `Esc  Close` and `Show`, takes its own box and its own bar, not
    those of the row.
