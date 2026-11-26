# review kit-swallow r1 (rc 1, rc 2 tiling; rpt2-1 MAJOR): seeded review of claude/kit-swallow-r1 before its merge

Model: opus, effort high, because it changes the core key-grab limit and the swallow of every look (core and look review; procedures: opus high)
Red on head: none (a review; the lane's reds b279bd09, 752af69f precede their fixes)
Lane: cloud, read-only on code: claude/kit-swallow-r1 at 02e745c335c7333fb1d06ac586840d40dc167257, review 4d8bbe36..02e745c3 (9 files, +252 -28: chora-kit swallow.rs, runtime.rs; chora ipc.rs; chora-xp wm/startup.rs; tests/gate-swallow-chords.sh, gate-construct-cube.sh, merge-gate.sh; docs). One round. Stop at 120 k context; cost at most 0.5 %; at most 8 findings.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-kit-swallow-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/kit-swallow-r1.md (done criteria), reports/kit-swallow.md "## r1" and reports/rc1-pass.md rpt2-1 (line 736, 771) on claude/session-log.
Checks, each with its command and files (no merge gate; `cargo test -p chora-kit -p chora -p chora-xp` and `cargo clippy` on those only):
1) Red first: b279bd09 fails on its parent and 37af9880 makes it pass; 752af69f before 1c57c3d5 likewise. 02e745c3 (xp sets its give keys before its first swallow) has no own red commit: is the matrix gate cell the lane cites a real red on 1c57c3d5 (name it), else MAJOR.
2) rpt2-1: every Super+Alt chord and Super+Ctrl+Shift+key are swallowed in tiling, xp, mc and construct; the give keys (Super+Alt+1..4 handover) still act; no chord the look binds is swallowed before its handler (POLS). Read gate-swallow-chords.sh: does the cell type into a real client and assert no character arrives, per look?
3) Core: MAX_GRABS 256 -> 1024 in crates/chora/src/ipc.rs: bounded, no per-key allocation in the hot path, the error at the limit is honest; 586 chords is not a sign the swallow should be a pattern (one grab per modifier set) instead of a list: judge KISS, a finding only if the list form is wrong, not merely larger.
4) The escape hatch (core keys no plugin overrides) still wins over every swallow.
5) Docs api.md, architecture.md say the same as the code.
Known, not findings: rpt2-2, sh-37..sh-45, TF-1, tiling-audit paper-2560x1600 bar flake, launcher-mc mc dark flake.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rksr1-N, severity, file:line, text, fix in one sentence). Append to reports/kit-swallow.md under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
