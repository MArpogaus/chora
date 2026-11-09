# Ledger

One row per date: the totals of the day over all models from
`plans/autonomy/tokens.py DAY`, overwritten during the day. Rows of past
days are frozen. Budget week: Saturday to Friday; the weekly sum is the
sum of the percent of its rows.

| date | input | cache write | cache read | output | seen (cache read, output) | carry (cache read, output) | percent | note |
|---|---:|---:|---:|---:|---|---|---:|---|
| 2026-11-08 | 30 | 66,328 | 1,260,238 | 21,477 | 1260238,21477 | 0,0 | 0.5 | Bangkok date (the dates moved to Bangkok on 11-09); most of the planning falls on 11-09 Bangkok |
| 2026-11-09 | 5,186 | 7,855,512 | 500,137,770 | 1,937,262 | 500137770,1937262 | 0,0 | 11.8 | weekly readings 3, 6, 8, 9 and 11 % (whole week, incl. the planning) |
