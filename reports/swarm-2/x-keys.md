# x-keys (xp)
## Goals
- Open Start menu and start a program: reached yes, tries 4, steps: Ctrl+Escape, Down (focus shows on Terminal), Return; foot opened. Super alone did not open Start.
- Open second program and switch between both windows: reached yes, tries 9, steps: Ctrl+Escape, Right, Down x3 (My Computer, failed: thunar missing); Ctrl+Escape, Right, Down x4, Right (All Programs), Right (Accessories), Return on Vim; Alt+Tab switched both ways.
- Maximize, minimize, restore, close: reached yes, tries 12, steps: Alt+Space, Down, Down, Return (Maximize); Alt+Space, Up x3, Return (Restore); Alt+Space, Down, Return (Minimize); Alt+F4 closed the newest foot window. Super+D: no effect.
- Find assistant or help with keys: reached no, tries 9, steps: F1, Super+F1, Alt+F1 (nothing); Start menu has no Help item; checked All Programs > Accessories, Graphics, Office, System. Control Panel and Run... not explored.

## Findings
| id | sev | kind | where | what I did | expected | saw | shot |
|---|---|---|---|---|---|---|---|
| x-keys-1 | minor | ux | desktop, Windows key | key Super_L (and "Super", "Meta", "Win" names) | Start menu opens | Super_L returned ok, nothing visible; other names rejected by the key tool | shots/003.png |
| x-keys-2 | minor | ux | Start menu, focus | Ctrl+Escape opened the menu | an item visibly highlighted | no item looked focused until the first Down; Tab then moved focus away and Down went back to Terminal | shots/004.png, shots/007.png, shots/008.png |
| x-keys-3 | minor | bug | Start > My Computer | Down x3 then Return | opens My Computer | "Cannot open thunar, the program is not installed" balloon at bottom right; it was still there several actions later | shots/011.png, shots/012.png |
| x-keys-4 | minor | ux | Start > All Programs | Right, Down, Right, Right, Return | the list shows and focus moves in order | Up from the first item wrapped to Close in the window menu, and submenu focus is not visible without a Right press; the Start menu closes on Escape but leaves the focused item unclear | shots/012.png, shots/013.png, shots/014.png |
| x-keys-5 | minor | ux | Start > Accessories > Vim | Return | a Vim app window | Vim opened inside a new foot terminal window, titled foot | shots/015.png |
| x-keys-6 | cosmetic | visual | taskbar after Alt+Tab | Alt+Tab from Vim to the first terminal | taskbar shows the focused window as active | the Vim taskbar button stayed highlighted while the terminal was on top | shots/016.png |
| x-keys-7 | minor | ux | window menu (Alt+Space) | Alt+Space, then Up/Down | focus starts on a visible item | first Down landed on Minimize, Up from the start wrapped to Close; Restore needed two more Ups; disabled items (Move, Size) are skipped silently | shots/018.png, shots/019.png, shots/021.png, shots/022.png, shots/023.png |
| x-keys-8 | minor | visual | Vim after Restore | Alt+Space, Up x3, Return (Restore) | Vim redraws its content | window restored, but the Vim intro text was gone and the area was blank | shots/024.png |
| x-keys-9 | cosmetic | visual | taskbar after Minimize | Alt+Space, Down, Return (Minimize) on Vim | minimized window looks minimized | the Vim taskbar button still looked pressed | shots/026.png |
| x-keys-10 | minor | ux | Super+D | key Super+d | show desktop (not confirmed what it should do) | nothing happened, and a plain "d" was typed at the terminal prompt | shots/027.png |
| x-keys-11 | minor | ux | Super+E | key super+e | file manager | a new foot terminal opened | shots/028.png |
| x-keys-12 | ok | - | window close | Alt+F4 on the newest foot window | window closes | closed with no confirmation; the other windows stayed | shots/029.png |
| x-keys-13 | minor | ux | help keys | F1, Super+F1, Alt+F1 in terminal | some help or assistant surface | nothing visible; a stray "#" may have appeared at the prompt, unconfirmed | shots/030.png, shots/031.png |
| x-keys-14 | major | ux | help / assistant | searched Start menu and All Programs | a Help item or assistant | no Help or assistant entry; All Programs has Accessories (Vim), Graphics (ImageMagick), Office (LibreOffice Suite, Calc, Draw, Impress, Writer), System (Foot, Foot Client, Foot Server, UXTerm, XTerm) | shots/033.png, shots/034.png, shots/036.png |

## Notes
- Keyboard-only use worked for: Ctrl+Escape, Down/Up/Right/Left/Return/Escape in menus, Alt+Tab, Alt+Space, Alt+F4.
- Windows-style keys that were not reliable: Super, Super+D, Super+E, F1 for help.
- Tester key names: Super is not a valid name for the key tool, which means Tom cannot reach it by its usual name.
