# Theme of the XP flavor

chora-xp draws the window frames, the taskbar, the start menu and the
desktop from these files. It rasterizes the SVG files at runtime. All files
but the ones drawn for chora come from
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
| `winxp/cascade-arrow.svg`, `winxp/cascade-arrow-selected.svg` | the arrow of a program group in All Programs, black and white | drawn for chora after the arrow of XP's menus | GPL-3.0-or-later, as chora |
| `winxp/check.svg`, `winxp/check-selected.svg` | the check mark of the current choice in the Control Panel, black and white | drawn for chora after the check mark of XP's menus | GPL-3.0-or-later, as chora |
| `winxp/switch-user.svg`, `winxp/stand-by.svg`, `winxp/restart.svg` | the icons of the Log Off and Turn Off dialogs | kde-plasma-winxp-launcher (commit `868b5b9`), start menu widget `contents/images/`; original artwork | GPL-2.0-or-later, `winxp/LICENSE` |
| `winxp/dialog.svg`, `winxp/dialog-button*.svg`, `winxp/dialog-icon-*.svg` | the bands of the Log Off and Turn Off dialogs, the Cancel button normal, under the pointer and down, and the light and the pale look of the big buttons | drawn for chora after the colors of `SessionDialogView.qml` and of XP's dialogs | GPL-3.0-or-later, as chora |
| `winxp/balloon-close.svg`, `winxp/balloon-close-hover.svg`, `winxp/balloon-info.svg` | the close button of a balloon tip, normal and under the pointer, and the information icon of the notification area | drawn for chora after XP's balloon tips | GPL-3.0-or-later, as chora |
| `fonts/tahoma.ttf` | Tahoma, the font of menus, task buttons and titles | the metric-compatible Tahoma of the Wine project, from the start menu widget | LGPL-2.1-or-later, `fonts/LICENSE.Tahoma-Wine.LGPL-2.1` |
| `fonts/LibreFranklin-Bold.ttf` | the label of the start button | Libre Franklin Project, from the start menu widget | OFL-1.1, `fonts/OFL.txt` |
| `wallpaper/bloff.png` | the wallpaper Bløff, scaled from 3840x2160 to 1920x1080 | Exposé wallpapers by phob1an | CC-BY-SA-4.0, `wallpaper/LICENSE` |
| `cursors/ModernXP/` | XP cursors | [modernXP](https://github.com/na0miluv/modernXP-cursor-theme) release, unmodified | GPL-3.0, `cursors/ModernXP/LICENSE` |

The files of Exposé and modernXP carry the text of the GPL version 3 and no
"or later" notice, so their SPDX identifier is `GPL-3.0-only`. They are
artwork that chora-xp reads, not code of chora.

## XP's own files

`get-microsoft-files.sh` of the reference repository installs files of
Windows XP into the XDG data directory, `~/.local/share` by default.
chora-xp reads them at run time, when they are there, and uses the free
files above when they are not. The repository holds none of them.

| File in the XDG data directory | Use |
|--------------------------------|-----|
| `fonts/WindowsXP/*.ttf` | XP's fonts, before all other fonts: Tahoma and Tahoma Bold for menus and buttons, Trebuchet MS Bold for the titles, Franklin Gothic Medium Italic for the start button |
| `wallpapers/WindowsXP/*.jpg` | the wallpapers of Control Panel > Desktop; `bliss.jpg` is the default wallpaper |
| `winxp-kde/start_flag.png` | XP's flag on the start button, instead of `winxp/start-flag.svg` |
| `sounds/WindowsXP/stereo/*.wav` | XP's sounds for the start, Log Off, Turn Off and the balloon tips |
