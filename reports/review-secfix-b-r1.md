# Review secfix-b r1 (sonnet, read only, 10:19 UTC)

Base caf9c80d, head c2a882ac (4 commits, 8 files, +82/-24).

- rab-60 closed: `without_keys` (config.rs:407) env_remove of every `*_api_key_env`; coder (coder.rs:304), ACP agent (agents.rs:1347), chora_mcp (tools.rs:89); reload via set_keys (chat.rs:504) and Agents::configure. Red tests fail at 2751e09e, pass at head.
- rab-61 closed: config.rs:1144 probes run_program with an arbitrary argv; test fails at b175a65b, passes at head; check runs on reload.
- LOW: AGENT_SANDBOX_UNSET is an unused opt-in knob; no test of the reload wiring; only declared key names are stripped (document in docs/voice.md); no known-issues lines for rab-60/61.

Verdict: MERGE-OK (no open HIGH or MEDIUM). LOW items to backlog.
