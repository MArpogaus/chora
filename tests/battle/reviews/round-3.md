Final finding after full review of all four diff slices plus cross-checks against the docs:

**Finding**

- `services/voice/agent.py:33-34` (as of HEAD, `6d492ab`) — **breaks** (voice prototype): the default `LLM_URL` is `http://localhost:8080/v1` and the default `LLM_MODEL` is `gufo`, but the launcher `services/voice/gufo.sh` binds the LLM server to port **8888** (gufo.sh:41, its own usage text says so) and registers it under the name `gufo-llm` (`--served-model-name "gufo-$KIND"`) — with stock defaults the agent can never reach its LLM.
  Repro: `./services/voice/gufo.sh llm $MODELS_DIR/<gguf>`; run `services/voice/agent.py` and speak a turn → `POST http://localhost:8080/v1/chat/completions` gets connection refused (server is on 8888); set `LLM_URL` to 8888 and the request names model `gufo`, which the server does not serve (`gufo-llm`).

not fixed: finding is entirely inside `services/voice/`, which this session must never touch; the uncommitted working-tree files there were left as they are.

Notes (not in the v0.5.4..HEAD scope, flagging only): the two uncommitted files in the working tree are a mid-fix of exactly this — they correct the port but still leave the three names mutually inconsistent (agent wants `gufo-llm`, gufo.sh now serves `gufo`, TTS session still hardcodes `gufo-tts`), plus stray debug `print`s (`agent.py:92` sits above the docstring, demoting it).

Everything else checked clean: IPC caps (128/16/8, 1 MiB line, 8 MiB output, 10 s grace, 64 client sockets), grab lifecycle incl. the Shift rule and `grab-any` revoke, claim routing, `remove`+`to` handover, client-socket restrictions, meta/transform limits, watchdog blame rules, headless limits, activation-token window, binding parse/display symmetry, desktop-entry field-code stripping, and the docs (api/architecture/configuration/decisions/roadmap) against the code.
