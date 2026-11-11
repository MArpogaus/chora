# m-tv (mc)
## Goals
- Find out what you can do on the first screen: partly. Screen shows only a header ("1 AGENDA", "WEDNESDAY 11 NOVEMBER 16:49", big "1 AGENDA" title). No list, menu, or instruction visible. Clock ticks, so the app is live, but nothing changed in any screenshot.
- Move around with arrow keys and open something: no. Down, Up, Left, Right, Return, Tab, space, Page Down, Escape all produced no visible change.
- Go back to where you started: not applicable (never left the first screen).
- Find help or a list of keys: no. F1 did nothing. No hint on screen about keys.
Tries: about 23 actions of the 90 budget. Stopped early because the screen never responded.

## Findings
| id | sev | kind | where | what I did | expected | saw | shot |
|---|---|---|---|---|---|---|---|
| m-tv-1 | major | ux | first screen, whole app | Looked at first screen, no input yet | Some visible list, menu or content to choose from | Only header and title "1 AGENDA" on a blank dark page. Nothing else. | shot 001 |
| m-tv-2 | major | bug | first screen, arrow keys | Pressed Down, Right, Left, Up (each with a 1 s wait and screenshot) | Selection or focus moves, or content scrolls | No visible change after any arrow key | shots 002, 003, 007, 008 |
| m-tv-3 | major | bug | first screen, Return | Pressed Return | Opens the highlighted item or something happens | No change | shot 004 |
| m-tv-4 | major | ux | first screen, help | Pressed F1 | Help or key list appears | No change, no help shown anywhere | shot 005 |
| m-tv-5 | minor | bug | first screen, click | Clicked screen centre (400,250), then Down | Click focuses something; Down then moves focus | No change | shot 006 |
| m-tv-6 | minor | bug | first screen, Escape | Pressed Escape then Up | Escape closes or backs out; Up moves | No change | shot 007 |
| m-tv-7 | minor | bug | first screen, Space, Tab, Page Down | Pressed space, Tab, Next (Page Down) | Advances, moves focus or pages | No change | shot 009 |
| m-tv-8 | minor | bug | window switching | Waited 3 s, pressed Alt+Tab | Switches to another window if one exists | No change, no other window seen | shot 010 |
| m-tv-9 | minor | bug | first screen, right-click | Right-clicked (400,250) | Context menu or nothing, but some reaction | No change, no menu | shot 011 |
| m-tv-10 | minor | ux | key name | Sent key "Super" | Accepted as a key name | Tool error: unknown key "Super". Gives no clue which names are valid. | (text only) |
| m-tv-11 | cosmetic | visual | first screen, layout | Looked at the whole screen | Some sign of what the page is for | Large empty dark area, tiny grey text in the corners (date at top right is hard to read from a sofa). No button or hint. | shot 001 |

## Notes
- Screen is live (clock went 16:49 to 16:50) but the content never changed. The page may be waiting for data, or input may go to a different element than the keyboard focus.
- Screenshots were removed when the slot stopped, so shot numbers refer to the persona capture folder of this run (001 to 012).
