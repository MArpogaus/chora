MERGE-OK
Reviewed claude/media 00225164 vs 66f13d68 (3 commits, 43 files, +2101, no deletions).

| Sev | file:line | finding | evidence |
|-----|-----------|---------|----------|
| MINOR | README.md:47-48 | tiling caption "four terminals in the fair layout" is wrong: the picture shows one terminal, one editor and two scene panes. Alt text repeats it. Overstates nothing but is inaccurate. | tiling.png viewed: panes are a scene, a vim-like editor, a scene, one foot shell |
| MINOR | README.md:48 | xp caption "the editor is dragged and bends" is not visible in a still (frame looks flat) | xp.png viewed |
| MINOR | docs/img/xp.png | wallpaper is green hills and blue sky with a distant mountain: close in look to Bliss, though the repo's file is Bloff (CC-BY-SA-4.0, flavors/xp/theme/README.md). Owner should glance at it. Taskbar/start button/title bars are Luna-like (from the GPL kde-plasma-winxp pack, as documented), not Microsoft files | flavors/xp/theme/README.md lines 7-8 and the wallpaper row |
| MINOR | demo/tour/launch.py:78-79,52 | writes /home/user/notes/friday.md outside the repo (create + overwrite) for the demo; default paths /home/user/log, /tmp/claude-0/tour also outside the repo. Env-overridable only for some | grep of launch.py, ch*.py, assemble.py |
| MINOR | demo/tour/ch7b.py:6, ch8.py:7 | hard-coded OUT /home/user/log/media/2026-11-12/tour-v3 (stale v2 dates, no env override); ch8 and ch7b are left in though the report says they are out of v3 | grep |
| MINOR | status/img/ | brief step 1 images could not be inspected: neither status/img/ nor any 2026-11-16 status file exists on origin/claude/session-log (only media/2026-11-16/tour-v3 exists there) | git ls-tree of session-log. Licensing for those pictures unchecked |
| COSMETIC | docs/img/tiling.png | tooltip "Every key (Super+F1)" stuck at top right and bar text runs together ("No helperVolume 40%"); already in the worker's findings | image viewed |
| COSMETIC | README.md:40-42 | build sha 7d90f049 is not a commit of claude/media; reader cannot resolve it | git log |
| note | gate | worker reports doors FAIL (launcher-chat, mc). Diff has no compiled code, so unrelated to this change, but the PM should know before merge | reports/media-r1.md |

Checks without finding
1. Licensing: no ttf/otf/woff/wav/ogg/mp3/ico/bmp/jpg/video added anywhere in the diff; only 4 PNG in docs/img (99-137 KB). Images show our own art, no real Microsoft logo (details below).
2. 8030390b: 37 files, all under demo/tour/ (scripts, notes json, shell). Whole diff: zero files under crates/ or plugins/. Not MAJOR. Subject is docs(demo) with a "No test:" line.
3. README Looks: wording is plain, says "headless, software rendering", no superlatives; image paths docs/img/{xp,tiling,mc,construct}.png all exist. Links fine. Only the two caption issues above.
4. Tour scripts: no secrets, tokens, curl/wget, sudo, apt/pip/cargo install, no pkill/killall/kill of foreign processes (grep for these found none; launch.py starts its own session with start_new_session). Only the out-of-repo paths above.
5. Commits: all 3 conventional (docs, docs(demo), docs(demo)), author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>, no Co-Authored-By or Generated-with, longest body line is 72 or less (awk length check: none over 72).
6. Large files: nothing over 1 MB added; the only >1 MB blob in the tree is the existing services/voice/model/silero_vad.onnx (2.8 MB, in base). Videos are on session-log only.

Images (all four viewed)
- docs/img/xp.png: blue-sky green-hill wallpaper (chora Bloff art, not the Bliss photo; a small mountain at right), green "start" button, blue title bars, foot terminal, nano "Editor" window, Recycle Bin icon (generic green recycle glyph, not the Microsoft icon), tray clock. No Windows flag, no Microsoft logo, no Tahoma-labelled bitmap.
- docs/img/tiling.png: dark bar, four panes (scene text with bar chart, scene text with graph, editor with Rust code, foot shell), tooltip. No Microsoft content.
- docs/img/mc.png: cream page "2 THE MORNING", serif text, helpers column, key list. No Microsoft content.
- docs/img/construct.png: pale floor, three terminal panes in perspective, top bar, key hint bar. No Microsoft content.
