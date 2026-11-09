# Feedback inbox

One file per persona run: `DATE-RUN-PERSONA.md`. Each finding is one
section in this form:

    ## <short title>
    kind: issue | feature
    persona: <name, tech affinity 0 to 4, input>
    flavor: tiling | xp | mc | construct | <other>
    severity: high | medium | low   (the persona's guess)
    steps: <what the persona did, from the action log>
    expected: <what it expected>
    actual: <what happened>
    evidence: <capture path, action log lines>
    status: new

The PM sets `status` to one of: `verified`, `duplicate of <title>`,
`rejected: <reason>`, and after routing `fixed in <sha>`, `issue #N`,
`backlog`, `pitch`, `question N`. `triage.py` counts the findings that
are still `new`.
