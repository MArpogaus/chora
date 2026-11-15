# ledger r2 (rc 1): open BLOCK and MAJOR on be205f89 (train 13)

Reviewer lane: opus, cloud, read-only on code. Build: `cargo build --locked --workspace` of be205f89 in
/home/user/wt/head (3 min 11 s). Input: the 7 open lines of reports/ledger-r1.md; the 25 fixed ones stand on
the train 13 merge gate and were not rerun.

## Runs on be205f89 (all exit 0)
- `fuzzel --version`: 1.9.2 +cairo +png +svg(nanosvg).
- tests/gate-tiling-notice.sh (mako 1.x now installed by setup.sh): screenshot saved, mako notice at the bottom right.
- tests/gate-chat-keys.sh: five ok lines (one helper direct, text and Enter reach it, Esc, hand-over, two helpers).
- tests/gate-doors.sh: 80 of 80 cells pass, no failed check (5 min 31 s).
- Probes: scratch copies of tests/doors/{tiling,xp,mc,construct}.sh with `ctl capture` after the cells
  keys-names, helper-none and in check_cube_doors (ONLY set to those cells), 1280x800, 1x.

## Counts (BLOCK + MAJOR not fixed)
| Look | Before (r1) | After | Open after |
|---|---|---|---|
| xp | 0 | 0 | |
| tiling | 2 (1 BLOCK) | 1 (1 BLOCK) | ti-1 |
| mc | 1 | 0 | |
| construct | 0 | 0 | |
| shared | 4 | 1 | sh-10 |
| all | 7 | 2 | |

## Still open, with evidence
- ti-1 BLOCK: fuzzel 1.9.2 here (no mouse selection), not clicked. PK1 (5dd34fcd) declares fuzzel as a
  package dependency but sets no minimum the container meets. Open until fuzzel >= 1.11.
- sh-10 MAJOR: xp (two whole lines) and mc (wrapped, whole) are fixed. Tiling chat header reads
  `No helper is set up. Ask the person who set up this c...`; the construct chat panel clips
  `...who set up this compu` at its right edge with no ellipsis. The cells helper-none pass because they
  read only `No helper is set up`; they do not ask for the second sentence whole. The waiting word of a
  question was not rechecked.

## Fixed, with proof on be205f89
- ti-4: 6092aee3 (unit chora-tiling-mako); gate-tiling-notice ok.
- mc-30: ux-r1 merge 246e9fe8 (e1256beb, 6009cebc); gate-doors checks mc header-1280x800 and -1024x768 pass.
- sh-5: 246e9fe8 (ea2947fe); gate-doors check construct cube-doors pass; strip with clock and Lock stood back.
- sh-7: 246e9fe8 (5aaf7a13); gate-doors cell keys-names pass in all four looks, asks for Super+Alt+1-4.
- sh-11: 246e9fe8 (c2e80cad, 1a732e23, a32eb349); gate-chat-keys ok. The `1 atlas` label was not rechecked.

## New BLOCK or MAJOR
None. Seen and left as MINOR (not entered): the tiling notice cuts the screenshot path
(`.../Screenshots/2026-11-15_0...`); the tiling `Chat ...` chip and the mc CHAT line `Chat with a helper ...`
end in an ellipsis at 1280x800.

## Captures at 1x (in the worker, not committed)
/tmp/claude-0/-home-user-chora/4f796d76-e23e-58ba-8d13-8dbc938231ea/scratchpad/caps/: tiling-screenshot-notice.png,
mc-header.png, mc-keys.png, tiling-keys.png, tiling-nohelper.png, xp-nohelper.png, mc-nohelper.png,
construct-nohelper.png, construct-stoodback.png, construct-face.png
