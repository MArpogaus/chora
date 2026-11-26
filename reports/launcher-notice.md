# launcher-notice

## r1 (cloud lane, base 53789bea6, head 53789bea6, no code change)

Result: rml1-2 as written (the launcher's notice connection is refused for lack of a grant) does not reproduce. No red test can be written against 53789bea6 for a grant, so there is no fix and no rule for docs/api.md. The brief is stopped here for the PM to decide.

Evidence (headless persona sessions, fuzzel stand-in that exits 1 with an err line, `<look> --launch fuzzel`; notice list = `choractl call notify state.get`):

| look | no chora-notify in the session | chora-notify running |
|------|--------------------------------|----------------------|
| mc | refused conn client="launcher" code=1; tell logs `notify.notify.show is no exported method (code 1)` | notice "Run could not start" is in `shown`, no refusal |
| xp | not run (xp persona starts chora-notify as a companion) | shown, no refusal |
| tiling | refused code=1, same log line | shown, no refusal |
| construct | not run: chora-construct has no `--launch`, same kit `tell` | not run |

Cause: code 1 is `RpcError::REJECTED`, not `MISSING_CAPABILITY` (4). `methods.call` for client `notify` answers "no exported method" when no chora-notify serves the session. The main socket is not confined, so `check_socket_grant` lets `launcher` through (grants.rs, methods.rs:92). Where the server runs, the notice shows over the unchanged 53789bea6.

Why it looked like a grant: the persona sessions of mc, tiling and construct (tests/persona/session.sh companions) start no chora-notify; only xp does. In the products every session starts it (flavor targets, README mc row, tests/flavor-notify.py). A plain `chora --headless` with the default config has no notice server either, so the notice has nowhere to go; `tell` already logs the reason on stderr.

Options for the PM: (a) close rml1-2 as a harness gap and add chora-notify to the persona companions of mc, tiling and construct (tests/persona/session.sh); (b) if a failed launcher must be visible without a notice server, that is a new design (a fallback surface), not a grant. Gate not run: nothing changed.
