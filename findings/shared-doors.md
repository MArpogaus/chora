# Shared doors: proposal for the PM (not a decision). Train 4, 498d9dea. Evidence: findings/shared.md sh-1..sh-11
RC bar: a visible way to start a program, the key list and the helper (mouse and keys); lock, power, volume, screenshot, clock reachable.
## Today (key / visible door)
| Door | xp | tiling | mc | construct |
|---|---|---|---|---|
| Key list | F1, Super+F1 / Help window | Super+/ / Keys button | Super+F1 only / key block | F1, Super+/ / panel |
| Lock | none | Super+Shift+Esc / Power menu | none | none |
| Power | none / Turn Off dialog (Log Off dead) | Super+Shift+e / bar icon | none | none |
| Volume | none / tray speaker | XF86Audio keys / bar module | none | none |
| Screenshot | none | Super+Print, Super+Shift+s / launcher rows | none | none |
| Clock | n/a / tray | n/a / bar | n/a / page head | none outside the xp face (verify) |
| Helper | Super+a / tray clip, Start row | Super+a (silent if no helper) / none | Super+a / margin door | Super+a / panel, launcher row |
| Launcher | Super+Space / Start, Run | Super+Space / Apps | Super+Space / none | Super+Space / launcher row |
## Gaps
- Four different key-list keys; a user of one look is lost in the next (sh-6). Give keys not in two help lists (sh-7).
- Lock, power, volume, screenshot exist only in tiling; mc and construct have none, xp has volume and a Turn Off dialog only.
- No visible helper door in tiling; no "Chat with a helper" launcher row in tiling and mc; no honest no-helper state anywhere.
- mc has no visible way to start a program; construct and mc show no clock outside their own pages.
## Options
- A. Per-look idiom, keys and doors as today (xp Start, tiling bar, mc column, construct HUD): fits each look, keeps the gaps, costs four help texts.
- B. One key set for the shared doors in every look, each look draws its own visible door for the same action: one thing to learn, small change.
- C. Visible doors only, no shared keys: fails the key-first users (tiling, mc remote) and the "keys and mouse" bar.
## Recommendation: B (KISS, POLS)
- F1 opens the key list in every look (Super+/ and Super+F1 stay as silent aliases where bound); the list names every key below.
- Super+Space launcher, Super+a helper, Super+Shift+Esc lock, Super+Shift+e power menu (Lock, Log out, Restart, Shut down), Super+Print screenshot, XF86Audio keys volume.
- Each look adds one visible door per action in its own idiom: xp Start footer (Lock, Log Off fixed) and tray; tiling bar (add Helper); mc column foot; construct HUD corner.
- Clock visible in all looks; "Chat with a helper" row in every launcher; the helper door always shows a state word, "No helper" included.
- Cost: F1 no longer reaches clients (vim help); the actions come from the tiling scripts (power.sh, screenshot.sh) moved to a shared folder.
Decision needed: key set above (yes/no) and F1 for clients. Then four fix lanes read sh-1..sh-11 once, not per look.
