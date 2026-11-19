# copies-0: every copies row at 0 or narrowed with a reason

## r1

Lane: cloud, sonnet, 11-19. Branch claude/copies-0 from train 65 85764974: ed9d62a1 (the slice) and ee53e09c (one attribute, below). Head ee53e09c. Brief briefs/copies-0-r1.md.

### Step 1: copies rows before and after

Counted with the rule of `check_copies` in tools/cut-check, row by row (outside the `allow` prefixes). Rows not named here were at 0 with max 0 before and are still.
Focus-port had already taken the rows of the audit (Command::Focus, Event::FocusChanged, surface-created): they were at 0 on 85764974.

| row | before (count / max) | after (count / max) | how |
|---|---|---|---|
| UnixListener::bind in plugin tests | 8 / 6 (the only copies REPORT: "grew to 8") | 0 / 0 | the cells of assistant, notify, construct (2), tiling (2), xp wm.rs (2) connect to `FakeCore` |
| confirm.answer | 8 / 8 | 0 / 0 | `kit::ask::ANSWER_METHOD` in assistant (chat.rs, voice.rs x2) and voice (client.rs x4, tools.rs through `ask::answer_call`) |
| open entry of state.get built by hand (WM3) | 4 / 4 | 0 / 0 | the four state cells (construct, mc, tiling, xp) take the entry from the new `Surface::entry` |
| surface-created filter (`to != NAME`) | 1 / 1 (mc acts.rs:33) | 0 / 0 | filter dropped (see below) |
| own on_path (new row) | 4 / no row | 0 outside two kit files / 0 | xp session.rs and voice config.rs use `kit::desktop_entry::on_path`; kit launcher.rs keeps its test stub, allowed in the row with the reason |

`tools/cut-check` before: 19 reported, one of them a copies row. After: 18 reported (assets, names, known), no copies line. The names/assets lines are not copies rows and are not in this slice.

### What changed

- kit `testing::FakeCore`: `respond(f)` (an answer for a request, else the default), `record(f)` (one log line per request, returns a `Log`), `hold(binding)` (the `input.grab` of it fails REJECTED), `Core.grabs`. Two unit tests in crates/chora-kit/tests/fake_core.rs.
- The raw fakes of eight cells are gone. The cells keep their `Log` (type `Arc<Mutex<Vec<String>>>`) and their log lines, so no assert changed.
- `FakeCore` applies commands to a real scene; the old raw fakes took anything. Cells of tiling, xp and the construct start answer `scene.apply` with null through a responder (their windows are not in the scene). This is the old behaviour of those cells, not a new one.
- kit `wm::open::Surface::entry` (the state entry of a surface; `Open::state` uses it), with a unit test of the JSON shape.
- notify and assistant get `chora-kit` with `testing` as a dev-dependency.
- mc acts.rs: `Conn::claimant_at` already leaves the connection's own name out (`|o| o != me`), so the `to != NAME` guard was dead. New kit test `the_claimant_under_a_point_is_never_the_connection_itself` (FakeCore, two claims) pins it.
- tools/copies.toml: four rows to max 0, new row `fn on_path`, comments say why.

### Behaviour fixes found

None. Backlog: none.

### Checks (head ee53e09c)

- `tools/cut-check`: no copies REPORT; `tests/cut-check-selftest.sh` ok.
- `cargo test --workspace`: green.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `tests/merge-gate.sh --jobs 3`: run 2 on the head tree, 41 of 41 steps PASS, exit 0 (log dir /tmp/merge-gate.0q8FOT of the lane). Run 1 on ed9d62a1 stopped at `ci-check`: gate-0.8-plugin-runtime.sh found `"output.list"` in xp `wm_on_fake_core`, which had lost its `clippy::disallowed_methods` allow when its `Runtime::with` moved to `wm_on`. One attribute put back (ee53e09c); the grep passes. Run 2 was started before that commit was made (the first commit try failed the subject length hook), so it ran on the same tree as ee53e09c, uncommitted at the time; `git diff ed9d62a1 ee53e09c` is that one attribute.

### Notes for the PM

- Only xp, tiling and construct cells needed the `scene.apply` responder; mc, assistant, notify did not.
- The audit's other rows (Focus 21, FocusChanged 9, ...) were already 0 on this head: focus-port is in.
- Rank 4 of audit-f also named the four state.rs; the xp one is wm/state.rs. The "known-red" xp wm::state tests of audit section 0 pass on this head (also on 85764974).
- Not done: nothing of the brief. The 18 remaining cut-check REPORT lines are names/assets/known rows, not copies rows.
