# c-cube (construct)
## Goals
- Get to the cube: no, tries ~25. Super+C (listed as "Open the cube" in F1 help) always went to a black frame for 3-10 s, then back to the same view. With one or two windows open, the black frame stayed with a floating "foot" label and the "Step back Alt+Esc" hint, and the cube never showed.
- Turn to every side and say what each side shows: no, no side was ever visible. Q/E turns and R/F look up/down work on the empty marble plane; W walking gave no visible change.
- Step into one side, use it, come back: no, never reached a side. Fly in (Return) into a terminal worked, and Alt+Esc stepped back out to the floating-window view.
- Back to the start: yes, tries 1. H ("Back to the start") returned to the empty marble view.

## Findings
| id | sev | kind | where | what I did | expected | saw | shot |
|---|---|---|---|---|---|---|---|
| c-cube-1 | major | bug | cube, empty desktop | Started with no windows. Pressed Super+C | Cube with a face per desktop | Black screen with only the "Step back Alt+Esc" hint, for 3-10 s. Then the same empty marble view, no sides. Repeated 4 times with the same result | shots/002.png, shots/003.png, shots/004.png |
| c-cube-2 | major | bug | cube, one window | Super+T (foot terminal), Alt+Esc, Super+C | Cube shows the terminal desktop | Black frame with a floating "foot" label. Stayed black about 10 s and then returned to the floating-window view. Cube never appeared (3 tries) | shots/031.png, shots/032.png, shots/033.png, shots/034.png, shots/037.png, shots/039.png |
| c-cube-3 | major | ux | cube, two windows | Second Super+T, Alt+Esc, Super+C | Cube with two faces | Black frame again, then back to the floating-window view with a second grey panel to the right. Panel never got a label or content | shots/040.png, shots/041.png, shots/042.png |
| c-cube-4 | minor | ux | Tab windows overview | Tab on the empty desktop | Window list | "0 WINDOWS" with an empty search box. Reasonable, but the cube has no matching content | shots/028.png |
| c-cube-5 | minor | ux | walking on empty desktop | W x9, Q/E turns, R/F look | Visible movement or a landmark | The horizon and marble floor barely change. Nothing to orient by, so I could not tell whether walking worked | shots/012.png, shots/019.png, shots/018.png |
| c-cube-6 | cosmetic | visual | black cube frames | Any Super+C black frame | Smooth transition | Black frame is abrupt. Floating "foot" label stays on the black frame with no pane under it | shots/033.png, shots/037.png |
| c-cube-7 | cosmetic | visual | floor reflection of the foot pane | Alt+Esc with the foot window in view | Reflection matches the window | Reflection under the window is a plain grey block, not the dark terminal content. Second pane's reflection is also grey | shots/034.png, shots/040.png |
| c-cube-8 | minor | ux | second pane | Opened a second terminal and stepped out | Pane shows its content | Second pane stays grey and blank with no label for 10+ s. Foot's pane is not affected | shots/041.png, shots/043.png |

Working as expected: F1 key list (Super+C = cube, W/A/S/D walk, Return fly in, Alt+Esc step back); Super+T opens a foot terminal; Return flies into it; H returns to start.
