# kit1 r1 (F2): chora_scene::paths and kit::words Language::of_session in every look

Model: sonnet, effort high
Lane: cloud, branch claude/kit1 from the session branch (5990b001, train 53: kit2, wm1, kit4 in). One round,
stop at 250 k. Cost about 1.5 %. Run `git fetch --unshallow` first.
Read reports/arch-plan-fable.md 1.4 and slice row KIT1, reports/architecture-rc.md rows 8 and 11 and
line 134 (C14) on claude/session-log.
1. `chora_scene::paths`: runtime, config, state and data dirs plus one private-dir check (owner is the
   current user, mode 0700, not a symlink). The core (ipc.rs, uploads.rs), choractl, kit icons.rs, chora-draw
   theme.rs and every look and service that reads XDG_RUNTIME_DIR, XDG_CONFIG_HOME or HOME for these dirs
   use it; their own lookups go.
2. `kit::words::Language::of_session`: one rule (LC_ALL, then LC_MESSAGES, then LANG; an empty value is
   unset). Every look, chora-assistant and voice chat take the language from it; their own env reads go.
3. Cells red first: an empty `LC_ALL` does not mask `LANG=de` (tiling theme.rs:707 today); a private dir
   owned by another user, or with group or other bits, or a symlink, is refused with a logged reason.
4. tools/copies.toml rows at 0 outside chora-scene and chora-kit (tests/probe exempt): `LC_ALL`,
   `XDG_RUNTIME_DIR`. Visible behaviour otherwise unchanged (door acts, NET2 acts; known-red only shrinks).
   No new feature.
Done when: cells red then green; `paths` and `words` coverage >= 80 % with
`cargo llvm-cov -p chora-scene -p chora-kit --features chora-kit/testing` if the tool is there (else name it a
blocker, do not install; the PM measures); merge gate once on the head (poll it in the same turn). An
independent security review follows (PM), not by you. Refactor proposals ranked in the report.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines)
and append to reports/kit1.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
