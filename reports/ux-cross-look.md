# Cross-look consistency: tiling, mc, xp (head 8f358fa)

Reviewer run, 2026-11-12, same harness as `reports/ux-deep-xp.md` (persona sessions, 1280x800, fake helper
"atlas", `--test-injected-press-is-real`). Captures `media/2026-11-12/ux-deep-xp/` named `tiling-t-*`, `mc-m-*`,
`xp1x-*` / `xp2x-*`, cited as [name]. Measured against `design/looks/shared.md` ("must stay identical") and
`product-briefs.md`. Each look may keep its own style; this page lists only where shared.md or the
RC list asks for the same thing and the looks differ, or where a look does not meet the shared RC (a)-(d).

Caveats: the persona harness starts `chora-notify` only for xp, so notification cards of tiling and mc
were not seen; the lock screen of this headless run may not be able to start (swaylock), so "no visible
result" for lock may be an environment limit, but no look showed any word either way. The mc and tiling
scenes were driven only on the first scene.

## MAJOR (ranked)

| # | Topic | Looks | What differs or fails | Evidence |
|---|---|---|---|---|
| 1 | Lock | xp: none. tiling: Super+Shift+Esc and Power > Lock give no visible result. mc: none. | Shared RC (c): lock reachable in every look. xp has no door at all; tiling and mc show nothing in this run (no lock screen, no error). | [xp1x-02, 23]; [tiling-t-14-power-lock]; [mc-m-07-lock] |
| 2 | Power, volume, screenshot doors | mc: none visible (first screen, key block). xp: no screenshot door. tiling: all three (Power menu, "No sound" text, Super+Print). | RC (c) per look. mc column has no power, volume or screenshot entry and its key block lists none. | [mc-m-01-first-screen]; [tiling-t-07-power-click]; [xp1x-17] |
| 3 | Key list: three different keys | xp F1 (and Super+F1), tiling Super+/ (F1 does nothing), mc Super+F1 (F1 does nothing). | shared.md lists `Super+a`, Escape, give keys as shared; the key list has no shared key, so a person who learned one look is lost in the next (P1, P8). One key (F1) should open the list in all three; each look may add its own. | [xp1x-17], [tiling-t-02, t-03-f1], [mc-m-04, m-05-f1] |
| 4 | Give keys (Super+Alt+1..4) missing from help | tiling: not in the Keys sheet. mc: not in the visible key block (all-keys not shown while a chat was open, see 7). xp: listed ("Super+Alt+1 ... 4"). | shared.md: "Every one of these keys is listed in each look's help". | [tiling-t-02-keys-click] vs [xp1x-17] |
| 5 | Launcher has no "Chat with a helper" | tiling Apps list and mc fuzzel list: desktop apps only. xp Start menu: yes (pinned). | shared.md "A visible way in (P3)": the launcher of each look has "Chat with a helper". | [tiling-t-05-apps-click]; [mc-m-06-launcher]; [xp1x-02] |
| 6 | Tiling: the visible chat door does nothing | tiling "Chat ..." pill (x 1194-1263, y 49-79) did not open the chat on two clicks (the question pill below it did not either); Super+a did. xp tray clip and Start row, and mc "Chat with a helper ..." row, open it. | shared.md door without a key; brief tiling RC "beginners must not be stranded". Possible harness cause is unlikely: the same flag worked for xp and mc. | [tiling-t-09-chat-working], [tiling-t-12-pill-click2] vs [tiling-t-11-superA] |
| 7 | Tiling chat surface is not tiling | Super+a opens a pale full-height box x 880-1280, y 20-800 on the dark Ink desktop: bold generic sans "Chat with a helper", a boxed "Close" button, 0 px margins, no gaps or rounded corners. mc keeps the book serif in its margin; xp keeps the balloon. | "One helper, many costumes": tiling is the only look whose chat does not follow its own style (the same boxed panel mc showed on its Terminal scene). | [tiling-t-11-superA], [mc-m-02-chat-open], [xp1x-43] |

## MINOR (same words, same behaviour where shared.md requires it)

| # | Topic | xp | tiling | mc |
|---|---|---|---|---|
| 8 | State word shown | "Atlas — Question for you" (header) | "Atlas · question for you · Write file" pill | "Atlas · question for you" and, above the column, "Atlas has a question · Write file" (two spellings, shown twice) |
| 9 | Helper surface count | one balloon | pill and card as two surfaces | row at the column head and the chat list both state it |
| 10 | Door wording | "Chat with a helper / Ask in your words" | "Chat ..." (pill), "Chat with a helper" in Keys | "Chat with a helper ..." plus "Super+a" |
| 11 | Launcher word | "Open:" (Run), Start "All Programs" | "Apps" | ">" prompt, no label; light beige panel on dark page |
| 12 | "Run" wording in help | "Run a program" | "Run a program" | "Start a program" (key block) |
| 13 | Mute | tray red no-sign is the speaker; the mic mute not seen | "Listening" pill + dot, no Mute word | "Listening" + dot, no Mute word (as ux-deep-mc M2) |
| 14 | Time left on the card | missing | missing | missing (fake sends none; verify with the real voice) |
| 15 | Notification | tray balloon, black border, 12 px, covers the clip | not seen | "NOTIFICATIONS 0" count in the margin, cards not seen |
| 16 | Helper list | "1 atlas" list even with one helper (click, then pick) | "1 Atlas ..." | "1 atlas" |
| 17 | Esc | closes the chat | closes list and keys sheet | first Esc closed the card, chat stayed open; keys sheet blocked while chat is open |

Identical and good (keep): question card order (title "Question from Atlas", prompt, detail, keys line "Press
Alt+Y to allow or Alt+N to refuse.", "A spoken yes does not count.", Yes left, No right, "Typing does not
answer.") in all three; Alt+y / Alt+n answered in xp and tiling (fake helper's answers file; mc not checked): `{"id":"1","yes":true}`, `{"id":"2","yes":false}`); Super+Space opens a launcher in all three;
Super+a opens the chat in all three; tiling and xp both say "Run a program" for Super+Space.

## Where each look keeps its own style well

- tiling: floating islands bar, "Keys" sheet with key caps, Power menu (Lock, Log out, Restart, Shut down),
  clean "Apps" launcher glass.
- mc: key block at the column foot answers "where are the keys" at once; large serif.
- xp: Start menu with pinned doors, Help window with a generated key table, Turn Off dialog, Date and Time.

## Suggested order for next week

1 F1 opens the key list in all three (and every list shows the give keys); 2 lock door in xp, a visible
word for lock/power/volume in mc, and a word when lock cannot start; 3 tiling: chat pill must open, chat
surface in the tiling style; 4 "Chat with a helper" row in the tiling and mc launchers; 5 one spelling of
the state word and one surface per helper in tiling and mc; 6 Mute word in tiling and mc; 7 time left on
the card (check with the real voice).
