# helper-greet

## r1

Lane helper-greet-r1, sonnet high, cloud, branch claude/helper-greet-r1 from d40142e4. Closes xp-47 (rpx1-2).
Step 0 green: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`, `run.py --help`
(it has no help: it runs the chapters).

Shas: red d3870732 (tests only), fix 0a0fca76 (head of claude/helper-greet-r1).

### Cause and fix

`greet_step` picks the greeting once at `GREET_AFTER`; with the agent up later the "no helper is set up" sentence stayed in
`said` and `chat_toggled` gave it to the chat on every open (`balloon ||` let an expired sentence through).
Fix in plugins/chora-assistant/src/voice.rs, no new message, no new timer:
- `greeting_follows_helper`, driven by the state changes that already exist (`voice.state`, a helper's state, the
  first answer of `ask_state`): while the no-helper greeting is still within its time and the agent or a helper is
  there, it is said again as the plain greeting for the time left.
- `chat_toggled` on open hands over only a sentence still within its time.

### Red (d3870732, `cargo test -p chora-assistant`)

```
the chat holds: An old sentence.                                  (a_balloon_sentence_older_than_its_close_time_never_enters_the_chat)
the chat holds: Hi, I am your helper, but no helper is set up yet, ...   (an_open_chat_after_a_helper_connected_does_not_get_the_no_helper_sentence)
assertion `left != right` failed: still says that no helper is set up   (the_greeting_without_a_helper_is_gone_when_the_voice_agent_connects)
test result: FAILED. 0 passed; 3 failed
```

Green at 0a0fca76: `test result: ok. 252 passed; 0 failed`.

### xp tour (headless, `python3 demo/tour/run.py xp`, voice.state at 6 s)

```
known act 31 is green: drop known (Super held, drags on the desktop to turn the cube: ...)
chapter xp: 31 acts, 30 green, 0 inv, 1 known, 0 red, 0 todo, 0 unchecked
tour: record: 0 red, 0 broken
```

x1: balloon "Hi, I am your helper. Click me to ask for something. ..." (plain greeting, after the agent came). x5 and x6: chat
"Atlas — Idle" / "Atlas — Done" with the real transcript, no "no helper" line; no capture json of the run holds
"no helper is set up". (The "known act 31 is green" line is the old one, not this change: STRAY-SUPER's.)

### Full gate

`CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`, all cells PASS, no red; last lines:

```
tiling-bar PASS   160s
mc-host    PASS    18s
e2e        PASS   264s
all steps passed; no --push, nothing is pushed
EXIT 0
```

`tools/sleep-ratchet`: ok (sleep 779, sleep-all 930). `pgrep -fa 'chora --headless'`: empty.
Not done: a welcome balloon wording (xp-30) and the wait for the voice agent stay as they were.

## PM review r1 (diff 124 lines; seeded, PM under 200)
Base d40142e4, head 0a0fca76; 2 files, +124/-4.
| Check | Result |
|---|---|
| Red first | d3870732 tests only (main.rs +96), 3 failed pasted; fix 0a0fca76 after |
| (a) greeting follows the agent | greeting_follows_helper on connect, helper state, voice state; no new type, no new timer (KISS) |
| (b) chat open after connect | no "no helper" line; test an_open_chat_after_a_helper_connected... |
| (c) stale balloon sentence | chat_toggled takes only `until > now`; test a_balloon_sentence_older... |
| xp tour x1, x5, x6 | clean (report) |
| Full gate | EXIT 0; sleep-ratchet ok |
Findings: none. Note (not a finding): the redone greeting is the plain one, it does not name the helper; the brief allows it.
Verdict (computed): 0 open MAJOR -> MERGE-OK 0a0fca76. Next train.
