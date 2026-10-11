# Theme of the XP flavor

chora-xp draws the window frames, the taskbar, the start menu and the
desktop from these files. It rasterizes the SVG files at runtime. All files
come from
[kde-plasma-winxp-launcher](https://github.com/MArpogaus/kde-plasma-winxp-launcher)
(commit `3e08801`), which bundles the Exposé theme and its own Windows XP
pack. No file here belongs to Microsoft: the fonts, Bliss, the sounds and
the original start flag stay out, as in the reference repository.

| Path | Content | Source | License |
|------|---------|--------|---------|
| `expose/decoration.svg`, `expose/close.svg`, `expose/maximize.svg`, `expose/minimize.svg`, `expose/restore.svg`, `expose/exposerc` | window frame, title bar buttons and their layout | [Exposé](https://www.opencode.net/phob1an/expose) by phob1an (Mark Whittaker), Aurorae theme `aurorae/themes/expose/` | GPL-3.0, `expose/LICENSE` |
| `expose/panel-background.svg` | the bar of the taskbar | Exposé, Plasma theme `desktoptheme/expose/widgets/` | GPL-3.0, `expose/LICENSE` |
| `winxp/winxp.svg`, `winxp/tasks.svg` | start button, start menu, notification area, task buttons | the Windows XP pack of kde-plasma-winxp-launcher, in its Plasma theme `desktoptheme/expose/widgets/` | GPL-2.0-or-later, `winxp/LICENSE` |
| `winxp/start-flag.svg`, `winxp/log-off.svg`, `winxp/turn-off.svg` | the flag of the start button, the icons of Log Off and Turn Off Computer | kde-plasma-winxp-launcher, start menu widget `contents/images/`; original artwork | GPL-2.0-or-later, `winxp/LICENSE` |
| `fonts/tahoma.ttf` | Tahoma, the font of menus, task buttons and titles | the metric-compatible Tahoma of the Wine project, from the start menu widget | LGPL-2.1-or-later, `fonts/LICENSE.Tahoma-Wine.LGPL-2.1` |
| `fonts/LibreFranklin-Bold.ttf` | the label of the start button | Libre Franklin Project, from the start menu widget | OFL-1.1, `fonts/OFL.txt` |
| `wallpaper/bloff.png` | the wallpaper Bløff, scaled from 3840x2160 to 1920x1080 | Exposé wallpapers by phob1an | CC-BY-SA-4.0, `wallpaper/LICENSE` |
| `cursors/ModernXP/` | XP cursors | [modernXP](https://github.com/na0miluv/modernXP-cursor-theme) release, unmodified | GPL-3.0, `cursors/ModernXP/LICENSE` |

The files of Exposé and modernXP carry the text of the GPL version 3 and no
"or later" notice, so their SPDX identifier is `GPL-3.0-only`. They are
artwork that chora-xp reads, not code of chora.

## XP's own files

With Trebuchet MS or Franklin Gothic Medium installed, chora-xp uses them
for the titles and the start button. Bliss works as a wallpaper with
`chora-xp --wallpaper FILE`. `get-microsoft-files.sh` of the reference
repository downloads these files.
