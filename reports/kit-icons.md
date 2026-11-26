# kit-icons

## r1

F7 (rc 11; rv9ka-2 MINOR): the kit program list shows one app icon per row again.

- Base: 613844fc (session head, t151). Head: abdd0158 on claude/kit-icons-r1.
- Commits: red 18e1b9f7 (`the_rows_carry_the_icon_of_their_entry`, fails on the base: rows "Pad\nScreenshot" instead of "Pad\0icon\x1faccessories-text-editor\nScreenshot"), fix df1a2d0f, then two test-harness commits (95c1f188, abdd0158) that the fix made necessary.
- Diffstat base..head: crates/chora-kit/src/launcher.rs (+App.icon, App::row, rows built per round), tests/persona/session.sh, tests/doors/lib.sh, tests/gate-mc-sofa.sh, tests/gate-tiling-apps.sh, tests/gate-tiling-apps-click.sh, tests/gate-xp-run.sh (one line each, 6 files besides launcher.rs: 69 insertions, 10 deletions in the first two files, 5 one-line changes in the rest).

### Fuzzel icon syntax used

fuzzel 1.11.1 (this machine) takes the rofi row in `--dmenu`: `label\0icon\x1fNAME`, NAME the `Icon=` value of the desktop entry (a theme name or a path). No flag is needed: icons are on by default (`--no-icons` / `icons-enabled` switch them off, `--icon-theme` picks the theme). fuzzel prints the label only, so the stdout line still matches the row. A row of an entry without `Icon=` stays plain text. A fuzzel that does not know the syntax (before 1.7) is not supported by the gate (need_fuzzel 1.11); an icon the theme does not have shows a blank gap and the label, no stray bytes (checked with an unknown name, and for Files whose theme icon is missing).

### What the fix touched beyond launcher.rs, and why

- tests/persona/session.sh: its fuzzel stand-in read the first row with sh `read`, which drops the NUL, so row 1 reached fuzzel as `Aaa Calcicon<US>foot` and was drawn raw (seen in the first capture). It now keeps stdin in a file and logs the label of the first row.
- The stand-ins of doors (lib.sh), mc-sofa, tiling-apps, tiling-apps-click and xp-run wrote the rows and matched whole lines of labels; with icons the doors cell `tiling launcher-order` went red (doors.log: "no helper row"). They cut `\0icon\x1f...` before writing.

### Captures (1280x720, mc Programs list, light, entries with Icon= in the shared data dir and the system theme)

media/2026-11-26/kit-icons-r1/programs-before.png (129951 bytes: text rows only) and programs-after.png (134384 bytes: an icon on the Foot and LibreOffice rows; Files and "No icon" stay plain).

### Gate

`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2` (no --push, full, no step filter), run directory /tmp/merge-gate.sHhKDs, at head abdd0158:

    tiling-disc PASS    73s
    e2e        PASS   277s
    all steps passed; no --push, nothing is pushed   (68 PASS, no FAIL, EXIT 0)

Two earlier full runs were not green and are not counted: the first ran while the persona commit landed (pre-commit "files were modified by this hook"); the second went red in doors `tiling launcher-order` (the stand-in, fixed above). No foreign flake. `pgrep -fa 'chora --headless'` is empty.
