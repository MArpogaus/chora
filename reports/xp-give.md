## r1
red e65ea05c (unit cell fails on ce2c436b), fix 4278d4cd, capture step 8b on the branch tip. Send To in the xp window menu (title-bar right click, Alt+Space): Tiling desktop, Media center, 3D space; a look not running is gray.
Gate (merge-gate --jobs 1): pre-commit PASS, ci-check PASS, build PASS, test PASS, clippy PASS, gate-lib PASS, xp PASS, xp-fields PASS, xp-question PASS, doors PASS.
Handover cell tests/gate-handover.sh: all steps ok, incl. 8b (Send To > Media center: mc owns the window, no xp task button; Alt+Space, Send To > Tiling desktop: tiling owns it).
Capture: media/2026-11-20/xp-give/send-to-submenu.png

## PM review r1 (PM not the builder)
MERGE-OK 22fead94; red e65ea05c, fix 4278d4cd, cell 22fead94 (gate-handover 8b). Checks: system() keeps Close last and bold; Send To rows by plain name (POLS for Erika, Murat); gray when the look has no give method (clients.list); act_give takes sys_window and reuses chora_kit::give::Windows::hand (no copy, F2); docs/configuration.md names it. No open MAJOR.
- pr1-1 MINOR: looks_running does a blocking clients.list call on every menu open in the press path; cache it on client events if the menu ever feels slow.
- pr1-2 NIT: a failed clients.list grays all rows silently; a log line would help the honest-state bar.
- Note: xp -> tiling give is the direction the tiling r2 T7 lost a window in reverse; handover-ti covers the core path; tiling r3 checks both.
