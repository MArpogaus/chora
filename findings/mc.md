# mc findings (train 4, mc 10eb3a7f) | id | severity | source:line | state | text | helper door, lock, power, volume, key list: see shared.md
mc-1 | MAJOR | swarm-6:21; fb-nurse6:8,52 | open | Plain Left/Right do not turn pages (only Super+Left/Right, mc.yaml); Escape does not go back; hint says arrow, key list says Super+arrow
mc-2 | MAJOR | swarm-6:23; fb-nurse6:151 | open | No on-screen way to start a program: only Super+Space (fuzzel); RC: a visible way to start a program by click
mc-3 | MAJOR | fb-swarm5-m5m:7; fb-nurse6:30 | open | No mouse way back from page 2: footer holds only "next"; footer clicks do nothing; mouse-only user cannot return
mc-4 | MAJOR | ux-deep-mc:39 | open | No visible Mute: state line is "Listening" and a dot; a click on the state word mutes with no label or hover mark
mc-5 | MAJOR | swarm-6:22; fb-nurse6:74,85 | verify | First page does not say what it is; text hard to read at 3 m (r6 added "Say Atlas or press Super+a"; min text now 14 px)
mc-6 | MAJOR | ux-deep-mc:26 | verify | Terminal scene lost the column and chat opened boxed and generic; e6cc46c7 keeps the column, check chat on scene 5 and a page turn with the chat open
mc-7 | MAJOR | ux-deep-mc:46 | verify | All-keys list overflowed the column, struck through the chat door, rows cut at the screen edge (d77f65a7 cleared the column)
mc-8 | MAJOR | ux-deep-mc:63 | verify | Critical notification: red unexplained footer line, 7 px close mark that did nothing, no urgency sign in the column row
mc-9 | MINOR | ux-deep-mc:72 | verify | Chat field is a 1.7:1 box with a generic placeholder, 36 px off the 28 grid (84f21def round field, 6902a2d9 grid)
mc-10 | MINOR | ux-deep-mc:85 | verify | Card text wraps badly (file name orphaned, "(412 / bytes)"), mono 14 px at the floor (72527394 fixed the widow)
mc-11 | MINOR | ux-deep-mc:96 | verify | Stop is a 30x14 muted word on its own row, no "Stopped." line (b50159dd gave helpers a head)
mc-12 | MINOR | ux-deep-mc:101 | verify | "NOTIFICATIONS 0" with chat open; the chat door jumps with each notice; title and body not told apart (346a6a98)
mc-13 | MINOR | ux-deep-mc:114; fb-nurse6:63 | open | Scene copy: "collector" jargon on page 2 reads as a failure, mixed fonts, red heads over nothing, divider off centre
mc-14 | MINOR | ux-deep-mc:134,138 | open | Chat body 18 px/27 vs spec 19/28; "Super+ <- ->" key text has a stray space
mc-15 | COSMETIC | ux-deep-mc:132,136,140 | open | Yes/No hover thickens the rim (other hovers use the bar); stale hover bar; footer link flush right vs mock flush left
mc-16 | COSMETIC | pixel-mc-r6:26,27,29 | open | Yes/No 32 px off grid; notification wraps with "a" at line end; duplicate head: "1 AGENDA" above the page title
mc-17 | COSMETIC | pixel-mc-r6:30-39,47 | open | Serif/sans baseline and numeral indent ±1 px, Listening dot 3 px short of the edge, caret, Stop/Esc/Close gaps, 15 vs 14 px mono, thin 7:1 margins
mc-18 | COSMETIC | ux-deep-mc:141; pixel-mc-r6:34 | open | Light terminal is a black block on cream (foot rejects colors-dark here); the question is stated 3 times in the column
mc-30 | MAJOR | doors-r3 (capture doors-r3-mc-keys.png) | open | At 1280x800 with voice and a host: helper name, the word Mute, the date and the island overlap in the mc header
