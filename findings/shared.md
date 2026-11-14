# shared findings (cross-look, train 4 498d9dea) | id | severity | source:line | state | text | doors proposal: findings/shared-doors.md
sh-1 | MAJOR | ux-cross-look:18; ux-deep-xp:34 | open | Lock has no door in xp, mc, construct (tiling: Super+Shift+Esc and Power menu exist, no visible result or word if the lock cannot start)
sh-2 | MAJOR | ux-cross-look:19 | open | Power (log out, restart, shut down) has no door in mc and construct; xp has Turn Off but Log Off is dead (xp-1); tiling has the Power menu
sh-3 | MAJOR | ux-cross-look:19; swarm-6:29 | open | Volume has no door in mc and construct; xp tray speaker is red/"Sound unavailable" without a card (Erika6 did not see it); tiling has a bar module
sh-4 | MAJOR | ux-cross-look:19; ux-deep-xp:129 | open | Screenshot has no door in xp, mc, construct (tiling: Super+Print and launcher rows, silent: ti-4)
sh-5 | MAJOR | ux-cross-look:19; ux-deep-tiling:25 | verify | Clock: xp tray, tiling bar and mc head show one; construct walk and stood-back views show none (only inside the xp face); check
sh-6 | MAJOR | ux-cross-look:20; ux-deep-mc:53 | open | Key list opens on four different keys: xp F1, construct F1 and Super+/, tiling Super+/ only, mc Super+F1 only; a learned look strands in the next
sh-7 | MAJOR | ux-cross-look:21 | verify | Give keys (Super+Alt+1..4) not listed in the tiling Keys sheet or the mc key block; shared.md wants every shared key in each look's help
sh-8 | MAJOR | ux-cross-look:22; ux-deep-tiling:27 | open | Launcher has no "Chat with a helper" row in tiling (applications dir) and mc (fuzzel); xp Start and construct launcher have it
sh-9 | MAJOR | ux-deep-tiling:27; swarm-6:25 | open | Tiling has no visible helper door (no bar module; waybar lists none) and Super+a says nothing when no helper runs; xp, mc, construct have a door
sh-10 | MAJOR | ux-deep-tiling:27; ux-deep-xp:38; swarm-6:13 | open | No-helper state is missing or unreadable: tiling and mc draw nothing; xp balloon clipped, title cut; a question gets no answer or waiting word
sh-11 | MAJOR | swarm-6:14; ux-deep-mc:109 | verify | Chat first use: text typed before a helper is picked is dropped, Enter does nothing until one is picked, "1 atlas" unlabelled; one helper should open direct
sh-12 | MINOR | ux-cross-look:36; ux-deep-mc:92; ux-deep-xp:46 | open | Question card: no time left (all looks) and no transcript line after an answer (xp, mc); sh-13 on spelling
sh-13 | MINOR | ux-cross-look:30,31,38; backlog:37 | verify | State word spelled three ways, helper surface counted twice (tiling pill+card, mc row+list); one surface per conversation (U3); helper list "1 atlas"
sh-14 | MINOR | ux-cross-look:35; ux-deep-tiling:136 | open | Mute word missing in tiling and xp (dot or red sign only); mc: mc-4
sh-15 | MINOR | ux-cross-look:39; ux-deep-mc:150 | open | Esc differs: xp closes the chat, tiling closes list and keys, mc first Esc closes the card and keeps the chat; keys sheet blocked while chat is open
sh-16 | MINOR | ux-cross-look:33,34 | open | Launcher words: xp "Open:", tiling "Apps", mc ">" prompt with no label; help says "Run a program" in xp and tiling, "Start a program" in mc
sh-17 | MINOR | backlog:36 | verify | HUD colours per theme (U2): chat field over the xp taskbar; assistant surfaces must follow a theme change
sh-18 | COSMETIC | ux-cross-look:32,37 | open | Door wording differs ("Chat ..." pill, "Chat with a helper ...", "Ask in your words"); notification shape differs per look (balloon, count, toast)
