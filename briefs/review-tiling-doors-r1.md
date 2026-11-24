# review tiling-doors r1 (rc 2: ti-55, ti-56 MAJOR; ti-50 MINOR): seeded review of claude/tiling-doors-r1 before its merge

Model: opus, effort high, because the diff changes a look's doors and shared chora-kit (launcher.rs order, words.rs), and a swallowed-key path; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/tiling-doors-r1 head b1ba6743, base b02dbea9 (24 files, +513 -81). One round.
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-tiling-doors-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, EXIT 0, no SKIP).
Read briefs/tiling-doors-r1.md (done criteria), reports/tiling-doors-plan.md "## r1" (decided words), reports/tiling-doors.md "## r1", findings/tiling.md ti-50, ti-55, ti-56 on claude/session-log; `git diff b02dbea9..b1ba6743`; captures media/2026-11-24/tiling-doors/.
Checks, each with its command and files:
1) Red first: ba7755f6 (tests only) before eda266a5, 2b6c775a, f69e7771 (`git log --oneline --stat b02dbea9..b1ba6743`). Do c7d88215, b447a2cf, b1ba6743 change behaviour without a red test? A fix with no red test before it is MAJOR.
2) ti-55 (plugins/chora-tiling/src/looks.rs, main.rs launch_looks, waybar config, style.css): Looks lists only running other looks (clients.list), rows by give::move_words, the one row "No other look is running"; a chosen row moves the focused window without loss (give.sh path); words en/de as the plan. `cargo test -p chora-tiling`.
3) ti-56 (act, hint_unbound, show_words): chip 3 s, once per 10 s, never while the key list shows; the key stays swallowed (no key reaches the client); help key from System::Keys; layout chip keeps 700 ms. POLS: does the hint name the real help key if the user rebinds it?
4) ti-50 (crates/chora-kit/src/launcher.rs tiling_order replacing helper_last): other looks using helper_last keep their order (xp, mc, construct launchers); `cargo test -p chora-kit`.
5) Beyond the brief: words.rs avoids_jargon exemption only NoOtherLook; the gates whose x positions moved (gate-tiling-bar-palette, -bar, -bar-spec, gate-launcher-click) still assert what they asserted (no weakened check, no new sleep); lint-ratchet 47->46 true.
Known, not findings: rth-1..6, rv9j-1..5, rpt1-3..10 (ti-54), Super+q unbound (decision).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtd1-N, severity,
file:line, text, fix in one sentence). Append to reports/tiling-doors.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
