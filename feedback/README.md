# Feedback inbox

One file per persona run: `DATE-RUN-PERSONA.md`. Each finding is one
section in this form:

    ## <short title>
    kind: issue | feature | ux
    persona: <name, tech affinity 0 to 4, input>
    flavor: tiling | xp | mc | construct | <other>
    severity: high | medium | low   (the persona's guess)
    task: T1..T7 | X1.. | L1.. | M1.. | C1.. | none   (p2-plan task id)
    check: <gate cell, act or invariant id, e.g. I3, acts:xp-start, doors:lock> | none
    steps: <what the persona did, from the action log>
    expected: <what it expected>
    actual: <what happened>
    evidence: <capture path, action log lines>
    status: new

Every run file reports the UX/UI flaws the persona met, from its own
view, as findings of kind `ux` (hard to find, read, understand, reach
or undo; broken or inconsistent looks). A run that met none says so in
one line: `ux: none seen, because <reason>`. `triage.py` reports a run
file with neither as MISSING UX.

A high finding with `check: none` is not a BLOCK candidate until a check
exists (`triage.py` marks it HIGH, NO CHECK). Run files dated 2026-11-20 or
later need `task:` and `check:` on every finding (`triage.py --format`);
older files read both as `none`.

The PM sets `status` to one of: `verified`, `duplicate of <title>`,
`rejected: <reason>`, and after routing `fixed in <sha>`, `issue #N`,
`backlog`, `pitch`, `question N`. `triage.py` counts the findings that
are still `new`.
