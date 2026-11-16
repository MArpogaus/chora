# Review of claude/secfix-a r1 (bb237e8), sonnet, static read, 10:0x UTC

Verdict: FIX (two MEDIUM).
- MEDIUM icons.rs:92: is_small_png opens before any type check; a FIFO path blocks open() and hangs the notify thread. Fix: O_NONBLOCK open, fstat, is_file().
- MEDIUM icons.rs:44-55, chora-notify main.rs:250, card.rs:249: TOCTOU, the path is validated then re-opened by picture.foreign; the sender can swap in an SVG or huge PNG. Fix: validate bytes once, render those bytes or a private copy.
- LOW: check order otherwise right (fstat size, 24 header bytes, magic and IHDR 1..=512, is_file on handle).
- LOW: missing tests: exact boundaries, empty file, FIFO, symlink, file:// prefix, theme fallback by stem.
- LOW: docs/known-issues.md:92 sh-16 still describes the old wording.
Words (raa-3): consistent across looks and gate scripts.
