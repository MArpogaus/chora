# ux r1 (rc 1): the last open MAJOR findings, claude/ux-r1 at ea2947fe (base d349a6da)

Gate: NOT passed as one run. Three whole runs of `tests/merge-gate.sh` on ea2947fe: runs 1 and 2 fail only at
tiling-audit (tile border not `line`; the bar does not show), the flaw of briefs/tiling-audit-r1.md, whose fix
b018c506 is not in this base; the desktop cell alone fails 1 of 4 here. Run 3 fails at `test` in the core test
`words::tests::a_slow_backtrace_is_not_cut_by_the_alarm` (55 ms alarm; passed in runs 1 and 2). In parts every
step passes: red-first to layout in runs 1 and 2, chat-keys to mc-paths in one `--only` run. Pushed as work in
progress. Next: merge the session branch with train 11 into claude/ux-r1 and run the gate once.

1. mc-30 fixed (05be0631 red, e1256beb, 64cff04f): over an app the mc header ends where the column begins, also
   with the column hidden under `voice_hud: host`. Red: `check mc header-1280x800 fail`, `-1024x768 fail`. Green: pass.
2. sh-10 fixed (9b27e711 red, 4a9a7fe2): the xp balloon shows the notice in two whole lines; the German sentence
   is shorter. Red: unit test, `cell xp helper-none missing`. Green: both pass.
3. sh-11 fixed (409c6fcb red, c2e80cad): a helper that is the only entry opens at once; a text typed in the list
   goes to the helper picked by a click or by its number typed after it. Red: 3 unit tests, gate-chat-keys. Green.
   With other projects to start the list still shows, else nothing could start one.
4. sh-7 fixed (b1a185d7 red, 5aaf7a13): tiling Keys and mc All keys name `Super+Alt+1-4`. Red: keys-names missing
   in tiling and mc. Green: 4 of 4. mc says `Give window to a look` (no room for four names), and its full list has
   the side to itself under the state line, else it does not fit 1024x768 or 1280x800.
5. sh-5 fixed (a51a1710 red, ea2947fe): stood back from a cube the walk capsule shows; in a face it stays hidden.
   POLS: stood back is the walk, so the same strip in the same corner; the face has the doors of xp. Red and green:
   `check construct cube-doors`. gate-construct-cube, -pull and -layout pass.
6. ti-4 verified, no change: gate-tiling-notice passes with mako.

Open: the page header of mc at 1024x768 cuts its head (`1  AGEN…`), old. gate-mc-column (no gate step) fails on
d349a6da too: `no notice that the chat does not open`. The xp balloon without a helper still shows a field.
Captures at 1x, in the worker under /tmp/claude-0/-home-user-chora/591267fb-6425-5df8-85ad-370b3163999b/scratchpad/caps/:
mc-30-1280-keys.png, mc-30-1024-keys.png, sh-10-xp-nohelper.png, sh-11-tiling-list-typed.png,
sh-11-tiling-picked-sent.png, sh-7-tiling-keys-1024x768.png, sh-7-mc-page-keys-1024x768.png,
sh-5-after-stoodback.png, ti-4-tiling-screenshot-notice.png (before: mc-before-keys.png, sh-5-before-stoodback.png).
