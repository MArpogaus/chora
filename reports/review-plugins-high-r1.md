# Review plugins high r1 (claude/plugins-high a7e5921c..961a24ed), opus, read-only, 11-14 21:1x UTC

Verdict: MERGE-OK. XP-B-01, MC-01, NO-01 fixed on every path; red tests real; small fixes.
| id | sev | file:line | text |
|---|---|---|---|
| NO-02 | MINOR | plugins/chora-notify/src/card.rs:209 | the ellipsis can stick out past MAX_TEXT. |
| NO-03 | MINOR | card.rs:243 | the summary is not capped (slow, not quadratic). |
| NO-04 | MINOR | card.rs:393 | no test cuts through a multibyte character. |
