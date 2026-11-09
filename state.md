# State

last_reviewed: 4513ba2 (each branch since 8435faa was reviewed before its merge; the round of 8435faa..8b567b5 is clean)
base_2d: the commit before 5c447ce (the 2D guard); no scene or renderer change since
block: three work branches in parallel, started 11:50

## Plan of the day

2026-11-09, run 1 (started by hand at 10:27 Bangkok):

1. Tools (installing in the background), baseline.md.
2. backlog.md from charter section 15.
3. Security check, attacker view (subagent, opus high).
4. 2D guard (subagent).
5. 3D design (subagent, opus high).

## Next step

Order until the foundation holds (charter section 1 item 20):
stability and bug fixes, 3D, plugin separation; no feature sprints.

0. Disk (user request 11-09): after the security agent ends, `cargo
   clean`, add `[profile.dev] debug = "line-tables-only"` to
   `~/.cargo/config.toml` (incremental is already off there), rebuild;
   make `tests/2d-guard.sh` build the base into the shared target
   instead of `/home/user/target-base`, so the dependencies are shared;
   update charter section 3 "Disk" with the measured sizes.
1. Review round on 02cdeed..HEAD (running), answer its findings.
2. The two failing construct gates (backlog track A item 9).
3. Probe: a child session per run (reflection point 1).
4. Then 3D step 0 (plans/3d.md).
5. Remove inline comments that today's commits added (2d-guard.sh,
   tests/persona/, the security commits), per the user rule of 11-09.

## Done

- 11-09 15:00: XP start feedback merged (7dccda3): a missing program
  says so at once; a slow start shows "Starting NAME" after 2 s and
  "Cannot open" after 30 s without a window; launchers that exit 0 are
  followed like any program. Two review regressions fixed before the
  merge. Gate: 909 tests, 2D guard, xp.sh, matrix gate green.

- 11-09 14:30: grants step 1 merged (1ce735c): confined callers cannot
  claim the indicator, answer, confirm, talk or mute; injected pointer
  presses are marked and ignored by the HUD answer buttons; docs name
  the one-hop routes of methods.privileged. Opus security review done.
  Gate: 895 tests, 2D guard, xp.sh, matrix gate green.

- 11-09 afternoon: merged persona driver (b7897ba), 3D step 1 (7e3bb63),
  review fixes with guard hardening (9e48a34), XP split X2 (a83933f),
  xp.sh cleanup and the handover matrix gate (4513ba2: 46 of 84 cells
  pass, every cell as expected). Gate on 4513ba2: 891 tests, 2D guard,
  xp.sh, matrix gate green.

- 11-09: construct gates fixed and merged (89158c7): launcher window
  under a forking shell, two test script fixes; 3D step 0 merged
  (8aa0529): RenderStats and 2D path tests, review clean; 859 tests.

- 11-09: XP split X1 merged (97afe81): docs/xp-split.md, skeletons of
  chora-notify and chora-assistant, chora-draw as a shared library, D77
  (four repositories, amends D36); review: no high, two medium fixed;
  853 tests, 2D guard PASS. Panes as views of sources, D76 (8b567b5).

- 11-09: security fixes, all seven findings (0e3afa2 to 37052ae after
  the rewrite), 851 tests; history of the session branch cleaned of
  session files (37052ae); roadmap without plan links (4295193); guards
  `no-inline-comments`, `no-session-files`, `commit-message-length`
  (64c5999, b796e75, dbaf94b); target shrunk from 20.9 to 3.4 GB; 2D
  guard PASS against 8742dee after all of today's code.

- 11-09: security check, attacker view (opus). Items (a), (b), (c) of
  charter section 1 item 11 HOLD: (a) `confined: true` in
  `crates/choractl/src/mcp.rs:108-111,352-355`, test
  `ipc::tests::a_confined_main_socket_connection_needs_the_grants_of_a_client_socket`,
  live probe refused `run_program`, `tiling.settings.set`, `focus_next`
  and `load_view` without grants; (b) every tool call passes
  `confirmed()` (`services/voice/src/chat.rs:990-995`), default rule for
  `run_program` (`config.rs:362-375`); (c) `Config::spawn` puts
  `sandbox_cmd` first (`config.rs:421-426`), `check_harnesses`
  (`config.rs:1056-1061`), tests
  `harnesses_run_in_the_sandbox_unless_unsandboxed`, `check_rejects_mistakes`.
- 11-09: 2D guard `tests/2d-guard.sh` (5c447ce), red test against a
  changed premultiply in `render.rs`.
- 11-09: 3D strategy `plans/3d.md`, revised after an opus review
  (9cde738); decision D75 and roadmap (f5f346f).
- 11-09: baseline (07c6c09): 846 tests, most gates pass; `cargo deny`
  fixed (8589485, yanked async-executor).
- 11-09: driver for simulated users (b28adcf), audience research and
  everyday task matrix (b4b7520).

## Running

(none)

## Blocked

## Split reviews

split review: chora-xp 18258 2026-11-09 (planned: D73, roadmap track X1 to X7)
split review: chora-mc 6704 2026-11-09 (no split; share the voice wire types in chora-kit; design/split-review-mc.md)
split review: chora-construct 6530 2026-11-09 (no plugin split; split main.rs into files after the adoption merge; D76 geometry as a chora-kit module at K9; the /proc launcher check to chora-kit; design/split-review-construct.md)

## Review findings

Security check of 11-09 (probes in the scratchpad, `sec/`):

1. medium: spoken questions hide what runs: `run_program` speaks only
   the file name (`config.rs:1283-1286`), `inject_input` asks "Shall I
   type or click?", privileged calls do not name the command. Fix: speak
   the whole argv when it has more than one word or a `/`, the typed
   text, and the keys and values of `settings.set`.
2. medium (if voice-mcp has `methods.privileged`): the privileged list
   is a hand-kept copy (`config.rs:395-410`); `methods.list` does not
   send the privileged flag (`methods.rs:262-267`).
3. low-medium: injected keys reach grabs without an "injected" mark, so
   a client with `input.inject` can answer a confirm question meant for
   a physical key.
4. low: a confined main-socket connection skips `check_paths`,
   `confine` and the claims check (`ipc/clients.rs:262,289`,
   `ipc.rs:1287`); shader compile errors leak file content.
5. low: the event connection of `choractl mcp` is not confined
   (`crates/choractl/src/events.rs:70-74`).
6. low: `load_view` has no confirm rule.
7. docs: `services/voice/README.md:343-350` is out of date.
Review round of 11-09 on 02cdeed..8435faa (five views; bugs: clean,
187 voice tests and the new core test pass; security: nothing high or
medium):

8. medium (docs): `services/voice/README.md:346` still says the agent
   does not ask before privileged methods (same as security finding 7,
   open in the security agent).
9. low: `tests/persona/session.sh` runs `rm -rf "$dir"` on exit also when
   `$dir` is a caller-supplied `PERSONA_DIR`; delete only what it made.
10. low: `tests/2d-guard.sh` hard-codes `/home/user/wt/base2d`,
   `/home/user/target-base`, `/home/user/guard-cache`; make them
   overridable; the base cache ignores changes of the scenes.
11. low: the privileged question is no longer a user `confirm` rule, so a
   user cannot switch it off per method; document it in `docs/voice.md`
   (or keep a rule).
12. low: `said` and `spoken` in `services/voice/src/config.rs` duplicate
   each other; `privileged_rule` builds a throwaway `Confirm`;
   `client.rs` test doc names "finding 3 of the security review" (a
   private reference); `input.rs` `KeyAction::Grab` doc merges two
   sentences; `tools.rs` doc of `privileged_rule` contradicts own
   settings methods; `smoke.sh` has dead `cp` lines; `act.py type` text
   starting with `-` needs `--`; an all-move `inject_input` is spoken
   "Shall I type or click: ?"; `docs/voice.md:271-277` is one run-on
   paragraph.
13. low (security): `privileged_rule` fails open for a method not yet in
   the set within a turn; the spoken params are never cut (a long
   question).

Not verifiable here: `agent-sandbox.sh` is not in the repo (a writable
`.git` in the sandbox would let an agent plant git hooks).

## Checked and rejected
