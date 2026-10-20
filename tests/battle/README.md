# Battle tests

Workflows W1-W8 that drive a real headless chora session end to end and
report every surprise as a finding in `findings.md` (git-ignored,
regenerated per run). Design friction that needs an API or doc change
goes to [FRICTION.md](FRICTION.md).

Run one workflow:

    RUN=/tmp/w1.log bash tests/battle/w1.sh

Run the overnight driver (all workflows per pass, small-fix sub-agents
per new finding, ends after two clean passes; it waits until
`target/review-loop/state.txt` holds `PHASE=battle`):

    nohup bash tests/battle/battle.sh &

Needs the debug binaries (`cargo build --workspace`), `chora-probe` with
`--server-deco`, and ~2 GB free memory. Helper scripts kill leftovers
(`kill-session.sh`) and stop the drivers (`kill-loop.py`). Review and
fix sub-agents must exclude `services/voice/` — the owner's prototype.
Session mechanics (driver, heartbeat, sub-agent audit): skill
`local-pi-session`.
