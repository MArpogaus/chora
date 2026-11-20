## r1
red e65ea05c (unit cell fails on ce2c436b), fix 4278d4cd, capture step 8b on the branch tip. Send To in the xp window menu (title-bar right click, Alt+Space): Tiling desktop, Media center, 3D space; a look not running is gray.
Gate (merge-gate --jobs 1): pre-commit PASS, ci-check PASS, build PASS, test PASS, clippy PASS, gate-lib PASS, xp PASS, xp-fields PASS, xp-question PASS, doors PASS.
Handover cell tests/gate-handover.sh: all steps ok, incl. 8b (Send To > Media center: mc owns the window, no xp task button; Alt+Space, Send To > Tiling desktop: tiling owns it).
Capture: media/2026-11-20/xp-give/send-to-submenu.png
