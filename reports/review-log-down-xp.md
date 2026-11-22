# review log-down-xp

## r1 (PM review, 2026-11-22 13:45 UTC; builder: lane session_016iS3kPYbLrqZVkcAvctZ1S, PM not the builder)

Branch claude/log-down-xp, base 70cbf9bd, head ce11e056 (a54d5bd3 levels, 924f9ec3 merge of the session branch, ce11e056 helpers). Area diff plugins/chora-xp: 13 files +59 -19 (under 200 lines: PM review allowed, procedures).

Checks against briefs/log-down-xp-r1.md:
1) eprintln 18 -> 1: `git diff 70cbf9bd ce11e056 -- plugins/chora-xp`; the one left is launch.rs:39, the --run-box usage line before log init (allowed by the brief). Levels per message: failures warn or error (spawn error, unsaved change, adopt failure: error; reserve, shader, subscribe, turn, clients.list, sound: warn), state notes info ("stays"), per-event debug (notice id, "stays on its face"). Prefix "chora-xp: " dropped; target names the crate. Pass.
2) No new crate-level allow; allow 360 = list. Pass.
3) Test wm::tests_moved::a_failed_clients_list_warns_once asserts " WARN chora_xp::wm" and ": clients.list". Pass.
4) Gate: lane report, merge-gate --jobs 1 EXIT 0, 49 PASS, on the branch merged with the session branch. Pass.
5) User data at info or above (rc 9a): warn lines carry a command string from the config and a theme sound path; no typed text. Pass.

Findings:
- ldx-1, MINOR, plugins/chora-xp/src (ce11e056): eight one-line helper fns (warn_status, error_start, warn_reserve, ...) exist only to keep six fns under the complexity lint that the tracing macro expansion raised; indirection without a second caller. Fix: none for the RC (cheaper than six local allows); revisit if the lint learns tracing macros.

Verdict (computed): 0 BLOCK, 0 MAJOR, 0 MEDIUM, 1 MINOR -> MERGE-OK. Joins train 110. After both log-down merges the PM lowers the eprintln ratchet (tree 74, list 91).
