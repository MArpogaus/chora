# Review xp-polish r1 (sonnet, read only, 10:42 UTC)

Base caf9c80d, head 1df6d8ad (6 commits, 9 files, +193/-14). Red-first order checked by log, not run.

- xp-24 holds: Look.pad 4 only for xp (chat.rs:515); captures xp-chat-field-1x/2x show field level with Send; gate-xp-fields asserts 28 px.
- xp-23 holds: voice.rs:611-613 adds the idle figure only for the xp form; label "no helper" when none runs (voice.rs:939 test).
- MINOR: no capture of the Run box (fuzzel) nor of the no-helper idle figure; no unit test that non-xp forms keep want = None.
- LOW: voice.rs:611-613 dense expression. COSMETIC: docs/xp-style.md:110 wrapping.

Verdict: MERGE-OK. Evidence gaps (two captures) go to xp-polish r2.
