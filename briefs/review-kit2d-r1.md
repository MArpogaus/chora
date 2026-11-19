# review kit2d r1 (F2): seeded review of claude/kit2d before its merge

Model: sonnet, effort high (a port of xp to fixed kit types; one small kit hook); the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/kit2d head 888dfb60, base 18231aa1 (wm1d, already reviewed; 16 files, +659 -234).
One round. Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green).
Read briefs/kit2d-r1.md (done criteria) and reports/kit2d.md on claude/session-log; `git diff 18231aa1..888dfb60`.
Checks, each with its command and files:
1) Red first: c316198e holds the default-bindings cell and comes before 09b1c797 (`git log --oneline 18231aa1..888dfb60`).
2) Defaults equal the old keys exactly: list every key of xp's old match at 18231aa1 (plugins/chora-xp/src/wm/keys.rs,
   wm.rs) and find each in the new defaults (config.rs, xp.yaml) with the same action; none lost, none added.
   `cargo test -p chora-xp`.
3) kit hook f7bcc021 (crates/chora-kit/src/actions/perform.rs): small, generic, default unchanged for the other looks;
   `cargo test -p chora-kit --features testing`, `cargo test -p chora-tiling -p chora-mc`.
4) Visible changes named in the report (help shows Super+/ beside Super+F1, chords alphabetical; Lock and Screenshot via
   choractl run): each is POLS-safe and matches the other looks, or a finding. Unfit bindings dropped with a warning,
   never silently; settings.set and reload regrab (read the code path).
5) No new System variant; xp-only keys stay on the look side; no keysheet work (KIT3).
6) tools/copies.toml rows true (`tools/cut-check`); docs/api.md and docs/configuration.md name `bindings` and defaults.
Known, not findings: lane merged wm1d 18231aa1 (MERGE-OK); wm1d r3 (3c84b2d1) merges later in the train.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/kit2d.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
