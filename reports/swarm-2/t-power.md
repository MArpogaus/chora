# t-power (tiling)
## Goals
- Open three terminals with usual keys: reached yes, tries 3, steps: Super+Return x3 (shots/002-004.png); tiled side by side, then right column split top/bottom.
- Move focus between them with keyboard: partly, tries 6, steps: Super+Tab moved focus to top-right terminal (shot 017.png). Super+Left/Right switch workspace, not focus (shot 005.png). Super+j, Super+k, Super+Down, Super+Alt+Right, Alt+Tab showed no focus change.
- Swap two terminals: no, tries 4, steps: Super+Shift+Left, Super+Shift+Right, Super+Shift+Down, Super+Shift+Tab. No swap; escape-like text leaked into terminal.
- Change the layout: no, tries 3, steps: Super+e, Super+w, Super+s. No visible layout change; each letter was typed into the focused terminal.
- Go to another workspace and back: yes, tries 2, steps: Super+Right to empty workspace 2 (shot 005.png), Super+Left back to workspace 1 with the three terminals intact (shot 006.png).
- Find the list of keyboard shortcuts: no, tries 4, steps: Super+slash, Super+F1, F1. No help or shortcut overlay appeared.

## Findings
| id | sev | kind | where | what I did | expected | saw | shot |
|---|---|---|---|---|---|---|---|
| t-power-1 | major | bug | keys: Super+Shift+Left/Right/Down, Super+Shift+Tab | pressed each with a terminal focused | the focused window swaps or moves | nothing moved; text leaked into the focused terminal: "dC" after Shift+Left, "[C" after Shift+Right, "[B" after Shift+Down, ";10;9" after Shift+Tab | shots/018.png, shots/019.png, shots/026.png, shots/027.png |
| t-power-2 | major | bug | keys: Super+e, Super+w, Super+s | pressed each to change layout | split, tabbed or stacked layout changes | no layout change; letter "e", "w", "s" typed into the right terminal | shots/023.png, shots/024.png, shots/025.png |
| t-power-3 | major | bug | keys: Super+slash, Super+F1 | looked for shortcut list | help or shortcut overlay | no overlay; "/" and a stray symbol typed into the terminal | shots/020.png, shots/021.png |
| t-power-4 | major | ux | key: F1 | pressed F1 to get help | some help output | nothing happened | shots/022.png |
| t-power-5 | major | bug | key: Super+Right, Super+Left | expected focus to move right | focus moves to the right-hand terminal | switched to empty workspace 2; Super+Left returned to workspace 1 | shots/005.png, shots/006.png |
| t-power-6 | major | bug | key: Super+l | expected focus to move right | focus moves | left pane widened instead; focus stayed on the left terminal | shots/009.png |
| t-power-7 | minor | ux | keys: Super+j, Super+k, Super+Down | expected focus to move | focus moves | no visible change | shots/007.png, shots/016.png |
| t-power-8 | minor | bug | key: Super+Alt+Right | pressed with left terminal focused | focus or no-op | "1C" appeared at the prompt | shots/013.png |
| t-power-9 | minor | bug | key: Super+semicolon | pressed with left terminal focused | focus move or no-op | ";" typed into the left terminal | shots/015.png |
| t-power-10 | minor | visual | top bar, left | looked at bar after Super+l | workspace indicator "1" stays visible | "1" label shown in shots 001-008, blank from shot 009 on | shots/009.png |
| t-power-11 | minor | ux | typing in terminals | typed "x" then "q" and looked | characters appear right away | characters appeared only after the next keypress | shots/009.png, shots/011.png |
| t-power-12 | minor | ux | key: ctrl+u | cleared the prompt before each test | prompt line empties | old text sometimes stayed ("qw", "1C2", "aa" remained) | shots/013.png, shots/015.png |
| t-power-13 | minor | ux | Super+Return | opened three terminals | each new terminal takes focus | focus after each open was hard to see; the cursor style changes between terminals | shots/003.png, shots/004.png |

## Notes
- Works as expected: Super+Return tiling of three terminals, Super+Tab focus cycling, workspace switch and return.
- Most Super+letter chords that are not bound leak the letter into the focused terminal, which is a surprise for an i3/sway user.
- No help or shortcut overlay could be found from the keyboard.
