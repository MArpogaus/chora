# c-chat (construct)
## Goals
- Find the assistant and open a chat with it: reached no, tries ~12, steps: empty desktop (no icons, dock or bar); Super+Space launcher lists only programs (Foot, LibreOffice, UXTerm, Vim, XTerm); typed "helper", "claude", "chat" in launcher: no match; Tab window switcher: "0 WINDOWS"; F1 key sheet lists "Helper and chat" keys but no way to open the helper; Super+T opened a terminal only.
- Ask it something and read the answer: reached no, tries 0, steps: could not open it.
- Answer a question or permission prompt: reached no, tries 0, steps: no helper appeared.
- Close and reopen, conversation kept: reached no, tries 0, steps: never opened a chat.
## Findings
| id | sev | kind | where | what I did | expected | saw | shot |
|---|---|---|---|---|---|---|---|
| c-chat-1 | major | ux | desktop, launcher | Looked for the assistant on the empty desktop, then Super+Space and typed helper / claude / chat | An entry point to the built-in assistant (icon, launcher entry or search hit) | No icon, no bar, no launcher entry; every search returned an empty list | shots/009.png, shots/010.png, shots/011.png, shots/012.png |
| c-chat-2 | major | ux | F1 key sheet | Pressed F1 | Key sheet tells me how to open the helper | Sheet lists "Helper and chat": Alt+Y yes, Alt+N no, Ctrl+. stop the helper, all "every look", but no key or action to open it | shots/008.png |
| c-chat-3 | minor | ux | window switcher | Pressed Tab on empty desktop | Some hint of what is available | Shows "0 WINDOWS" and an empty list; nothing to open | shots/013.png |
| c-chat-4 | minor | ux | Super+Alt+4 | Pressed "give it to the construct" shortcut with the terminal open | Visible change, or a message if no window is open | Nothing visible changed; no feedback | shots/015.png |
| c-chat-5 | minor | ux | Super+x | Pressed "close the pane ahead" with one terminal open | Terminal closes, or a message | Terminal stayed open with no feedback | shots/015.png |
| c-chat-6 | cosmetic | visual | F1 key sheet | Read the sheet | Readable text | Key labels and hints are very small and light grey on cream, hard to read at 800px width | shots/008.png |
| c-chat-7 | cosmetic | visual | terminal | Super+T | Clean start | Yellow locale warning on the first line ("'C' is not a UTF-8 locale"), unrelated to the assistant | shots/014.png |
| c-chat-8 | minor | ux | desktop | Clicked the top edge of the desktop | Something opens, or nothing (intended) | Nothing happened; no hint on the desktop | shots/005.png |
- Note: the desktop shows no focus or hint of any kind at start. A new user would not know the assistant exists or how to reach it. Keys named "Super" and "Alt" in the overlay worked with act.py only as "Super+space", "Super+t", "Super+alt+4", "Alt+Tab"; "Super", "Meta" and "super" alone were rejected by the tool (not product bugs).
- Not tested: the chat itself, replies, permission prompts, and reopen persistence, since no path to the assistant was found within budget.
