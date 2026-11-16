# secfix-a r1

Branch claude/secfix-a (bb237e8, on the session branch 6658888).

- bc5c477 test: refuse SVG and oversized PNG icon paths from a sender (red: 3 of 4 icons::tests fail, an SVG path, an SVG named .png and an oversized PNG are returned)
- d047cfb fix: raa-23. `Icons::for_sender` takes an absolute path only as a PNG of at most 512 KiB and 512x512 pixels (signature and IHDR read, no decode); anything else falls back to the theme icon by file stem; never SVG. chora-notify uses it. Green: 4 of 4.
- bb237e8 docs: raa-3, words only (PM decision: the Run box is by design). Launcher row words: xp and tiling "Run a program or a typed command" (xp help.rs, tiling keys.rs, tiling.yaml, waybar tooltip), mc and construct "Start a program or run a command". The mc gate scripts (gate-mc-column, doors/mc, mc-capture) follow the new row text. The docs of `answer` and `launcher::run` already say it and are unchanged.

Gate: tests/merge-gate.sh once on bb237e8, all steps passed (run dir /tmp/merge-gate.2z5xeJ).
Left open: the construct start-card lead ("Start a program. ...") still says only program; the German words are unchanged (the rows are English only). Pixel gates that read the mc row text ran green with the new words.
