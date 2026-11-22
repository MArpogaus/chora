# opt-in

## r1 (plan slice 20, rc 3: the opt-in gates green once, early)

Lane: cloud, claude/opt-in at 7a6c0eb8 (session head, clean tree). Brief: briefs/opt-in-r1.md.
Step 0: `git fetch --unshallow` (done), `cargo check --workspace` PASS (1m 46s),
`cargo build --workspace` PASS (3m 07s, dev profile). Tools present: ffmpeg, foot, fuzzel,
Pillow, prlimit. No script was changed; no product code was touched.

Result table (each script run once, sequentially, in the background; logs in /tmp/opt-in/):

| script | result | seconds | failing line |
|---|---|---|---|
| tests/gate-watchdog-starved.sh | PASS (2 ok) | 58 | - |
| tests/gate-mc-scale.sh | PASS (7 ok) | 21 | - |
| tests/gate-construct-keys.sh | PASS (18 ok) | 21 | - |
| tests/gate-construct-keys-persona.sh | PASS (1 ok, steps 0 to 27) | 15 | - |
| tests/gate-demo.sh | PASS (10 ok, 4 clips made, no SKIP) | 234 | - |
| tests/gate-0.9-shader-hang.sh (prlimit --as=6000000000) | PASS (2 ok) | 11 | - |

Seconds are wall time of the whole script, including the wait for /tmp/chora-gate.lock
(all six waited on it; the scripts' own gate_elapsed excludes that wait, I did not split it out).
No red script, so no second run. No SKIP or FAIL line in any log.

Findings
- oi-1 (look: brief path). The brief names tests/kill-session.sh; the file is
  tests/battle/kill-session.sh (it ends the sessions of workflow names, /tmp/bt-WF.core).
  Not needed: after the six runs no chora, foot or fuzzel process was left (pgrep empty).
  Log: none.
- oi-2 (nit, temp files). tests/gate-lib.sh session_start makes /tmp/gate-env.* with mktemp,
  and gate_cleanup does not remove them: 5 files were left after one run. /tmp/gate-config.*
  and /tmp/gate-demo.* are removed. Log: ls /tmp/gate-env.* after the runs.
- oi-3 (plan vs brief). plans/week-3.md slice 20 (line 92) lists the opt-in gates as
  battle, watchdog-starved, 0.9-shader-hang, mc-scale, construct-hud, construct-keys, demo.
  The brief runs construct-keys-persona instead and does not run battle or construct-hud.
  The slice's green is therefore not yet complete; the PM should confirm which list the
  candidate run uses. Log: n/a (plan text).
- oi-4 (margin, size). gate-demo's size check passed, but the clips are close to the 10 MB
  limit in the script header: cube GIF 9,492,797 bytes; construct-walk GIF 9,240,425 bytes.
  Whether 10 MB means 10,000,000 or 10 MiB is not stated. Log: /tmp/opt-in/gate-demo.log.
- oi-5 (nit, header). tests/gate-construct-keys.sh header is hard to read: the text runs
  "also under the default voice_hud: host;" and then "a give key without a window says so;
  Super+t starts ...", and a list of co-numbers mixes into it. Log: head -40 of the script.

## PM note r1 (08:16 UTC)
Checked: six PASS lines in the table, logs named; no product change, so no review (rule 7 needs none for a run without code).
oi-3 answered: construct-hud runs in every merge gate (step `hud`, t107 PASS 78 s); battle is the sim-long lane (r1 running, 3 h loop). The candidate run uses the plan list plus construct-keys-persona.
oi-1: brief error by the PM (path tests/battle/kill-session.sh); fixed in the template for the candidate brief.
oi-2 (gate-env temp files) and oi-4 (GIF margin 9.49 MB of 10 MB) to backlog DB21 (harness).
Slice 20: green once on 7a6c0eb8 (early); the candidate run repeats it.
