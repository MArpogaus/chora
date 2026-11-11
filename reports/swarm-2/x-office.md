# x-office (xp)
## Goals
- Open a program from the Start menu: partly, tries 3 (LibreOffice Writer via Start > All Programs > Office failed twice; Start > Terminal opened on the 3rd try). Steps: Start, All Programs, Office, LibreOffice Writer; retry; Start, Terminal. shots 004-010
- Move, enlarge, minimize, get back: yes, tries 1 each. Steps: drag title bar, maximize button, minimize button, click taskbar button, restore-down button. shots 011-015
- Second program and switch (mouse and Alt+Tab): yes, tries 1 (second program = Vim from Accessories; My Computer failed, see x-office-4). Steps: Start, All Programs, Accessories, Vim; click back window; Alt+Tab twice. shots 019-022
- Close everything: yes, tries 1. Steps: X on terminal, X on Vim (Vim closed without a prompt). shots 023-024

## Findings
| id | sev | kind | where | what I did | expected | saw | shot |
|---|---|---|---|---|---|---|---|
| x-office-1 | major | bug | Start > All Programs > Office > LibreOffice Writer (and the LibreOffice entry) | Clicked Writer, waited about 13 s, then retried and waited 30 s more | Writer window opens | Balloon "Starting libreoffice / Please wait." then "Cannot open libreoffice. No window opened in 30 seconds." Same on the retry. No window ever appeared. Calc, Draw and Impress not tried | shots/005.png, shots/007.png, shots/009.png |
| x-office-2 | major | bug | Start > My Computer | Clicked My Computer in the right pane of the Start menu | A file browser window opens | Balloon "Cannot open thunar. The program is not installed." Balloon text is lowercase "thunar" and "libreoffice". Desktop My Computer icon not double-clicked | shots/018.png |
| x-office-3 | minor | ux | Start > All Programs > Accessories | Hovered Accessories | A normal set of small tools (Notepad, Calculator, Paint) | Only one item: "Vim". Nothing a typical office user would expect | shots/016.png |
| x-office-4 | minor | ux | Start > All Programs > System | Hovered System | Clearly labelled system tools | Five terminal items with no explanation: Foot, Foot Client, Foot Server, UXTerm, XTerm. Terminal items also in the left Start pane as "Terminal / foot" | shots/017.png, shots/004.png |
| x-office-5 | minor | ux | Start > Control Panel > Effects | Hovered Effects | Clear options | Shows Wobbly Windows, Magic Lamp and Desktop Cube, all ticked, with no explanation of what they do. Also "Sounds" with no arrow | shots/026.png |
| x-office-6 | minor | bug | Start > Control Panel > Effects > Sounds | Clicked Sounds | A sounds dialog, or at least a message | Menu closed and nothing else happened. No window, no message | shots/027.png |
| x-office-7 | minor | ux | Alt+Tab | Pressed Alt+Tab with two windows open | A switcher preview, as in XP | Windows swapped straight away with no preview or list. Works both ways | shots/021.png, shots/022.png |
| x-office-8 | minor | ux | Control Panel > Display | Opened Display (dialog) | A dialog with a taskbar button, as in XP | Opens fine (Themes, Desktop, Screen Saver, Appearance, Settings tabs), but has no taskbar button. Cancel closed it | shots/028.png, shots/029.png |
| x-office-9 | cosmetic | visual | Terminal window, first start | Opened Terminal from Start | Clean window | Top line is a yellow warning "'C' is not a UTF-8 locale, using 'C.UTF-8' instead" | shots/010.png |
| x-office-10 | cosmetic | visual | Start > Office > LibreOffice Writer balloon | Launched Writer | Balloon clear of icons | Balloon sits over the Recycle Bin icon and its label | shots/005.png |
| x-office-11 | cosmetic | visual | Taskbar after minimizing the terminal | Minimized the terminal, then looked at taskbar | Minimized button looks un-pressed | Terminal's taskbar button still looked highlighted/pressed. Hard to be sure at this resolution | shots/013.png |
| x-office-12 | cosmetic | visual | Desktop | Looked at desktop | Labels readable | Desktop icon labels ("My Documents", "My Computer", "Recycle Bin") are very small and hard to read at this size | shots/001.png |

Notes: Start menu, Log Off and Turn Off Computer were not tested. Window move, maximize, minimize and close all behaved as in XP.
