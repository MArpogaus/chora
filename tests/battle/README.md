# Battle tests

Workflows W1-W8 that drive a real headless chora session end to end and
report every surprise as a finding in `findings.md` (git-ignored,
regenerated per run). Design friction that needs an API or doc change
goes to [FRICTION.md](FRICTION.md).

Run one workflow:

    RUN=/tmp/w1.log bash tests/battle/w1.sh

W9-W12 are the plugin days (tiling, xp, mc, and mc with tiling plus
confined agents). They write their own config home under `/tmp`, put
captures and log copies in `SHOTS` (default `target/battle-round`),
note the RSS of the core and the plugins, and stop at the first death
of the core. W10 reads `/dev/zero` as a wallpaper and kills chora-xp
past 1 GB. Run with the binaries of another target dir:

    BIN=$PWD/target/wt/debug CHORA_BIN=$PWD/target/wt/debug RUN=/tmp/w9.log bash tests/battle/w9.sh

W16 runs the real `chora-voice` beside `chora-mc` against fake ASR, TTS
and LLM servers (`w16-fakes.py`, a Python copy of the test-only fakes of
`services/voice/src/fake.rs`). The mic plays a recorded sentence on
request, the speaker logs what it plays, and the ASR and the LLM follow a
script that `w16.sh` pushes. It needs no GPU, no audio and no network.

Run the overnight driver (all workflows per pass, small-fix sub-agents
per new finding, ends after two clean passes; it waits until
`target/review-loop/state.txt` holds `PHASE=battle`):

    nohup bash tests/battle/battle.sh &

Needs the debug binaries (`cargo build --workspace`), `chora-probe` with
`--server-deco`, and ~2 GB free memory. `kill-session.sh WF...` kills the
leftovers of the sessions of these workflows only, by the pid of their
core, never by process name: other agents run sessions on the same host.
`kill-loop.py` stops the drivers. Review and
fix sub-agents must exclude `services/voice/` — the owner's prototype.
Session mechanics (driver, heartbeat, sub-agent audit): skill
`local-pi-session`.
