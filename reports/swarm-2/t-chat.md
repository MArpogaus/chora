# t-chat (tiling)
## Goals
- Find how to talk to the assistant (Super+a, other keys, on-screen hints): no yes. Super+a, Super+space, Super+Shift+a, Super+c, Super+d, Alt+a, Ctrl+space, Super+i, Super+slash, Super+Shift+slash, F12, Super+F1 gave nothing usable. Empty desktop with only a top bar (workspace "1", clock, "Sound unavailable", "Connected"); clicking "Connected" and the "1" showed nothing. Only Super+Return opened a terminal.
- Ask it something and read the answer: no (never reached the assistant).
- Close it and return to terminal; typing still works: yes. `echo ana-test` ran and printed `ana-test`; no stray text after Ctrl+u and Return.
- Find a list of all keys: no. No help, keybinding list or cheat-sheet found; F1 showed a network toast instead of help.
- Keys that leaked text into the terminal: Super+slash ("/"), Super+Shift+a ("A"), Super+c ("c"), Super+d ("d"), Super+Shift+slash ("?"), Super+Shift+Return (":10:13"), Super+i (probable "i"), F12 (probable stray glyph), Super+F1 ("f1" glyph).

## Findings
| id | sev | kind | where | what I did | expected | saw | shot |
|---|---|---|---|---|---|---|---|
| t-chat-1 | major | ux | desktop, no window | Super+a on empty desktop | some assistant/launcher opens | nothing appeared, no feedback | shots/002.png |
| t-chat-2 | major | ux | desktop | Super+space, Super+Shift+a, Super+c, Super+d, Super+i, Alt+a, Ctrl+space, F12 on empty desktop | a way to reach the assistant or a launcher | nothing visible for any of them; no hint text anywhere | shots/004.png, shots/010.png, shots/011.png, shots/013.png, shots/014.png |
| t-chat-3 | major | ux | top bar | clicked "Connected" (top right) and workspace "1" (top left) | a menu or the assistant | no response | shots/007.png, shots/017.png |
| t-chat-4 | minor | ux | desktop | Super+Return | some terminal | a terminal opened (only thing that worked); no hint for this key | shots/005.png |
| t-chat-5 | major | bug | terminal, Super+slash | pressed Super+slash with terminal focused | the key is handled by the desktop, not typed | "/" typed at the shell prompt | shots/008.png |
| t-chat-6 | major | bug | terminal, Super+Shift+a | pressed Super+Shift+a | not typed into shell | capital "A" typed at prompt | shots/010.png |
| t-chat-7 | major | bug | terminal, Super+c | pressed Super+c | not typed into shell | "c" typed at prompt | shots/011.png |
| t-chat-8 | major | bug | terminal, Super+d | pressed Super+d (launcher key on tiling WMs) | launcher or nothing | "d" typed at prompt; no launcher appeared | shots/012.png |
| t-chat-9 | major | bug | terminal, Super+Shift+slash | pressed Super+Shift+slash | not typed into shell | "?" typed at prompt | shots/015.png |
| t-chat-10 | major | bug | terminal, Super+Shift+Return | pressed Super+Shift+Return | not typed into shell | ":10:13" typed at prompt, looks like a timestamp or escape fragment | shots/016.png |
| t-chat-11 | minor | bug | terminal, Super+i | pressed Super+i | not typed into shell | probably an "i" typed at prompt (glyph is hard to read) | shots/018.png |
| t-chat-12 | minor | bug | terminal, F12 | pressed F12 | not typed into shell | a stray glyph (probably "-") typed at prompt | shots/019.png |
| t-chat-13 | minor | bug | terminal, Super+F1 | pressed Super+F1 | not typed into shell or a help screen | "f1"-like glyph typed at prompt | shots/021.png |
| t-chat-14 | minor | ux | F1 on desktop | pressed F1 to look for help | help or key list | a "eth0 192.0.2.2/24" toast appeared top right, no help | shots/009.png |
| t-chat-15 | minor | ux | terminal, first open | opened terminal with Super+Return | clean prompt | top line warns "'C' is not a UTF-8 locale, using 'C.UTF-8' instead" | shots/005.png |
| t-chat-16 | cosmetic | visual | top bar | general look | clear labels | "Connected" and "Sound unavailable" are small and unlabelled; unclear what "Connected" means | shots/001.png |
| t-chat-17 | minor | ux | whole session | looked for a key list or help | a discoverable keybinding list | none found in about 30 actions; the only Super combo that did something was Super+Return | shots/021.png |

Notes: Typing and running commands in the terminal worked normally after clearing stray characters with Ctrl+u. The assistant was never reached, so the answer goal was not tested. Actions used: about 31 of 90.
