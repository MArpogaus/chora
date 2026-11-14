# Ledger

One row per date: the totals of the day over all models from
`plans/autonomy/tokens.py DAY`, overwritten during the day. Rows of past
days are frozen. Budget week: Saturday to Friday; the weekly sum is the
sum of the percent of its rows.

| date | input | cache write | cache read | output | seen (cache read, output) | carry (cache read, output) | percent | note |
|---|---:|---:|---:|---:|---|---|---:|---|
| 2026-11-08 | 30 | 66,328 | 1,260,238 | 21,477 | 1260238,21477 | 0,0 | 0.5 | Bangkok date (the dates moved to Bangkok on 11-09); most of the planning falls on 11-09 Bangkok |
| 2026-11-09 | 6,892 | 10,074,150 | 618,932,798 | 2,548,749 | 618932798,2548749 | 0,0 | 14.6 | tokens.py at 17:33 Bangkok; weekly readings 3, 6, 8, 9, 11 and 13 % (whole week, incl. the planning); see the arithmetic below |

## 2026-11-09 arithmetic

- Day share: 19 % (charter section 1 item 1, week 0).
- The week reset at 2 % on 11-08, so 2 points of the weekly reading were
  already used before 11-09.
- User reading at 17:19 Bangkok: 13 % of the week. Used on 11-09 by the
  reading: 13 - 2 = 11 %. Left of the day's 19 %: 8 %.
  (`state.md` said "about 6 %" at 17:25; that took 19 - 13 and forgot
  the 2 %.)
- `tokens.py` at 17:19 gave 13.8 %, at 17:33 14.6 % (618,932,798 cache
  read, 2,548,749 output, opus-equivalent weights). It reads 2.8 to 3.6
  points above the reading minus 2 %, or 0.8 above the raw 13 %. The
  calibration (33 million cache read per percent) was fitted on whole
  week readings, so it may already include the 2 %; I cannot tell which
  from the data. If the estimate is right, 19 - 14.6 = 4.4 % is left.
  The ledger row holds the estimate; the user's reading is the check.
- Of the day's tokens, the PM session alone used about 5 points in 247
  turns between 04:15 and 08:00 UTC, when its context grew to 900,515
  tokens (reflection of the evening, point 3; counted by the reflection
  agent from the PM transcript, not by me).
- Work switches to haiku at 20 % of the day's share (reflection point 16).
