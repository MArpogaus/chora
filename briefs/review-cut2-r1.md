# review cut2 r1 (F6): seeded review of claude/cut2 before its merge

Model: opus, effort high, because the diff changes the core IPC emit rule (D65 exception: any client may emit
look.theme and look.view, the core stamps data.look) and makes the cut gate blocking for every merge
Lane: cloud, read-only on code: claude/cut2 head 29b2fd25, base dc4bc46d (33 files, +734 -115). One round.
Stop at 120 k context; cost at most 0.6 %; at most 8 findings. No merge gate run (pull flake F21 is known).
Read briefs/cut2-r1.md (done criteria) and reports/cut2.md on claude/session-log; `git diff dc4bc46d..29b2fd25`.
Checks, each with its command and files:
1) Core exception is minimal and safe (crates/chora/src/ipc.rs LOOK_EVENTS, stamped_look, ipc_emit): no client
   can set data.look to another name, no core event is spoofable, rate and size limits still apply, a client
   named `look` gains nothing more; a confined client socket emitting look.theme cannot steer another look's
   costume (assistant side: plugins/chora-assistant, folders.rs, rule '@look'). `cargo test -p chora ipc::`
   and the core cell `a_look_names_itself_with_the_shared_look_events`. PM decision: the exception stands as an
   amendment of D65 if this check passes; say whether docs/api.md and docs/decisions.md D65 match the code.
2) Red first: 4643e772 fails (selftest 8 cells, 3 assistant tests, core test), later commits green
   (`tools/cut-check --selftest` or the selftest the report names; `cargo test -p chora-assistant`).
3) Cut gate blocks what the brief names (packaging, assets, links, huds entries, names over cut-known.toml,
   --base new kit-enum arms) and never blocks a valid commit: run `tools/cut-check` on the head (exit 0) and
   on dc4bc46d..29b2fd25 with --base; read tests/merge-gate.sh change.
4) Visible behaviour unchanged: hud folders (tests/hud-folders.sh), PKGBUILD and crates/chora/Cargo.toml asset
   paths still install every file (packaging-files selftest); tiling still emits tiling.theme.
Known, not findings: pull flake under --jobs 3 (F21, flakes-f lane); 5 asset includes and 16 name pairs in
cut-known.toml; old event names kept in construct, mc, xp; refactor proposals 1-7; D103 wording (PM).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rw-N, severity,
file:line, text, fix in one sentence). Append to reports/cut2.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
