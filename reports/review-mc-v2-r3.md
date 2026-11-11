# Review: mc v2, round 3 (merge check)

Independent check of `claude/mc-v2` at d9a8a72 (binaries in
`/home/user/wt/mc-v2/target/debug`, built 21:56, after the HEAD commit),
2026-11-11. No code changed. Round 2: `reports/review-mc-v2-r2.md`.
Captures: `media/2026-11-12/review-mc-v2-r3/` (`mc-*` 1280x800 light,
`tiling-*`, `xp-*`, `cNN-*` zoomed crops).

How: `tests/persona/session.sh mc 1280x800` with `CHORA_BIN`,
`CHORA_TEST_FLAGS=--test-injected-press-is-real`, `PERSONA_DIR` and
`PERSONA_KEEP` in a scratch dir, the private config of
`tests/gate-mc-host.sh` (`voice_hud: host`, `hud: mc`, mc folder in
`XDG_DATA_HOME`), `tests/fake-voice.py` with a transcript and Alt+y/Alt+n;
driving by `act.py` only; geometry read with `choractl query`; each session
ended with `session.sh --stop`, the fifo holder killed by pid.

## Verdict: MERGE

No MAJOR left on a main path. The Esc path works at any speed a person can
reach; only an injected Esc under about 70 ms after the click is lost.

## Round 2 items re-checked

- **R1 (MAJOR) fixed.** Scene 2 lays the empty texts out in cells: lead
  `[160,168,641,112]`, weather `[160,351]`, day `[562,351]`, mail
  `[160,533]`, overnight `[160,716]`; no two visible mc texts overlap
  (checked over all 31 visible texts). `mc-01-scene-2.png`.
- **R2 fixed.** With a two-line field the card is `[888,430,336,228]`
  (bottom 658), the hint starts at 664. `mc-02-card-two-line-field.png`.
  Fixed in tiling too (card bottom 705, hint 714, `tiling-01-question.png`).
- **Hover marks fixed (R9, R10).** Yes, No, Stop (host `-mark` rects at
  opacity 1) and Esc Close (`mc-r1-right-chat-close-mark`) each show the
  mark under the pointer. Yes gets a 2 px border (`c01-hover-yes.png`).
- **R6 fixed.** Super+a after Esc reopens the one helper's chat with the
  draft kept (`mc-06-reopen-after-esc.png`).

## Esc right after a click on Yes

Five tries as asked (click Yes, Esc with no pause, `done` 0.3 s later):
4 of 5 closed. Then 63 more tries (the last ten with a timeline poll every 50 ms):

| Esc after the click | `done` | closed |
|---|---|---|
| no pause (act.py back to back) | after Esc, +0.3 s | 13 of 18 |
| no pause | +0.1 s or +5 s | 21 of 21 |
| +0.15 to +0.5 s | +0.2 to +2 s | 24 of 24 |

Every failure is a pair where the click process started 119 to 148 ms
before the Esc process ended (some pairs that fast pass too), which leaves
well under 100 ms between the release and the Esc. In those runs the Esc is
lost: the card goes, the answer arrives (`yes` every time), and the chat
stays open; it never closes and reopens. A person moving a hand from the
mouse to Esc takes well over 150 ms, and every try at 150 ms or more
closed. So this is not a race a person would hit. The 1-2 s settles in the
gate hide it, which is fine for a gate; a note in the gate about the reason
would help (COSMETIC).

## First look at the builder's `mc-v2-r3` captures

Calm, readable, the card sits in the column without covering anything.
What is still short of perfect (none blocks):

- **N1 MINOR [mc] Scene 2 drops the Reading panel at 1280 wide.**
  `mc-s-morning-panels-reading` and its empty text "Nothing to read yet."
  have no rects (cell `[2,2,2,1]` of `scenes/morning.yaml`), so five of six
  panels show, while the page has two large empty bands (y 380 to 530 and
  560 to 715). With data the reading list would not show either.
- **N2 COSMETIC [mc]** The lead "No briefing yet. The collector has not
  run." leaves "run." alone on line 2 (widow).
- **N3 COSMETIC [host]** "Stop" still stands alone on a line under
  "Esc  Close" (`c02-stop-own-line.png`).
- **N4 COSMETIC [host]** The card detail wraps "(412 / bytes)" mid-phrase;
  the field is still the boxed sans (R8, known).

## Other looks

`tiling-01-question.png`, `xp-01-question.png`: card, hint and field in
order, Alt+y answered (`91 yes`) and Esc closed the chat in both
(`*-02-after-esc.png`). xp now scrolls by whole messages (the speaker line
stays with its text; R11 orphan gone). No regression seen.
