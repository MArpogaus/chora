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

## Review r1

Lane: cloud, sonnet, 11-19. Head ee53e09c against base 85764974 (28 files, +365 -337). Brief briefs/review-copies-0-r1.md. Read-only on code.

Verdict: MERGE-OK (0 BLOCK, 0 MAJOR, 1 MINOR, 1 NIT).

### Checks

1. No behaviour change: holds. Each call site read old against new.
   - UnixListener fakes: the old fakes answered `hello`, the planted answers, `input.grab` with its binding, null for the rest, and logged every request (hello included). FakeCore does the same: `admit` logs before the refusal check, `input.grab` echoes the binding, the default is null. The tiling, xp and construct cells that applied commands add a `scene.apply` -> null responder, as the old fakes answered it.
   - xp `on_fake_core_refusing`: the old fake logged the refused grab, then failed it with REJECTED. `hold` logs, then fails with REJECTED.
   - confirm.answer: `ANSWER_METHOD` is "confirm.answer" (kit ask.rs:17); `ask::answer_call` is the old json literal.
   - open entry: `Surface::entry` is the old field list (id, kind word, modal, by); `Open::state` is the same map.
   - on_path: the kit `desktop_entry::on_path` is the body of the removed xp and voice copies.
   - mc acts.rs `to != NAME`: dead. `Conn::claimant_at` (kit give.rs:244, the only path) filters `|o| o != me` with `me = self.name()`, mc connects as `NAME`; `give_dragged` refuses `to == self.name()` too. The kit test `the_claimant_under_a_point_is_never_the_connection_itself` pins it with a smaller own claim over a larger other claim.
2. Kit helpers small and tested: respond/record/hold in fake_core.rs (two tests), `Surface::entry` (shape test in open.rs). `cargo test -p chora-kit --features testing`: all green.
3. Plugin tests keep their names and asserts: the diff touches no assert, only the fakes and the log type. `cargo test -p chora-construct -p chora-tiling -p chora-xp -p chora-notify -p chora-assistant -p chora-mc`: 243, 203, 178, 252, 22, 132 passed, 0 failed.
4. xp wm.rs allow: a grep exemption, not a hidden production call (see rc-1).
5. `tools/cut-check`: 18 reported (assets, names, known), no copies line. New `fn on_path` row: the one allowed copy is the kit launcher.rs test stub, its comment holds (a `matches!` over fixed names, reads no PATH).

`cargo check --workspace` green on the head.

### Findings

- rc-1, MINOR, plugins/chora-xp/src/wm.rs:971, the `clippy::disallowed_methods` allow on `wm_on_fake_core` suppresses no clippy lint (no `Runtime::with` or `Conn::connect_at` in that function any more; `wm_on` has them) and its reason names no disallowed call; it only opens the gate-0.8 grep for the `"output.list"` responder. The use is the one the grep documents ("a test on a fake core") and the same reason string stands on three construct cells, so it is not a hidden production method (the item is `#[cfg(test)]`). Fix: say so in the reason ("the fake core answers output.list, which the plugin-runtime grep names") or give FakeCore an `outputs(&[OutputInfo])` helper so no plugin test names the method.
- rc-2, NIT, crates/chora-kit/src/testing.rs:235, `admit` runs the recorders while it holds the model lock, so a recorder that locks `core.model()` deadlocks (the responders run after the unlock). Fix: one doc line on `record` ("the closure must not lock the model").

Not findings: debt-down also edits xp wm.rs (the train's merge); the 18 cut-check lines are not copies rows.
