# Review ux r2: claude/ux-r1 at 90513573 (lane commits ea2947fe..HEAD, first parent, 8 commits)

Red first, re-run here on `git archive` copies of the test commits (no checkout): 22e28034 fails its 4 M1
tests (sends 1), cc806b4b fails its 2 m2 tests, 9bed45ef fails the m3 audit test. Each fix follows its red.
m6 (89b715b6) and c1 (12477d48) are test-only. At HEAD: chora-assistant 214/214 ok, the m3 audit test and
give::keys_row ok. The mc door cells were not re-run (the builder reports the merge gate green in one run).

## M1: closed, by type
`Session::held` is gone; the text lives in `Mode::List(picks, Option<String>)` (chat.rs:333). Every exit
from the list replaces the mode and so drops it: hide and close_view (Esc, FieldEvent::Closed) set Closed
(chat.rs:865-880), show(Some) and the only-helper path go through open_chat, which sets Chat (:717), and a
second list (open_list, also the reconnect path :1320) writes `List(picks, None)` and clears the field (:758).
The only send_draft of a held text is in pick (:845-860), reached by a click or a number in the list. The
other send_draft (:927, host.rs:402) is the Send button, an act of the person. show sends nothing (:699).
Voice question balloon and chat.show RPC both go through show: no send. No path is left.

## MINOR
m1 chat.rs:875-880 / :717 the draft the field got back stays in the field after Escape or a hide, and
   open_chat clears the field only for another helper (`!same`). So when the last helper opens again (a
   question balloon, chat.show, a click on its row), "rm -rf build" stands in that helper's field as its
   draft. Nothing is sent without Enter, so it is no M1 leak. It is still a surprise: a question balloon
   opens on its own, the person types an answer and it is appended to the old list text ("rm -rf buildyes")
   and goes out with Enter. It also contradicts docs/hud.md:360 "The text ends with the list". Fix (cheap):
   in hide/close_view, when the mode was `List(_, Some(_))`, call `link.clear()`. Test: list text, Esc,
   show(last helper): field cleared. Earliest gate: a chat.rs unit test with the fake Link (`cleared`).
m2 accepted: "ask atlas" and "atlas" after a text are text (chat.rs:982-986); "hallo9", "9", "0" fall
   outside 1..=len and come back with pick_first, i.e. "Pick a helper first" (voice.md:1507). Without a held
   text a name still picks (`before.is_none()`), as the docs say. KISS reuse of the notice is fine.

## COSMETIC
c1 region.rs:642 `whole` no longer asks for a side region (the old code had `side &&`). A `keys` content
   placed in the header or footer draws nothing there, but with all keys shown it would now empty that line
   (clock, head). Misconfiguration only; add `!region.is_line() &&`.
c2 tests/doors/mc.sh:74 the probe exclusion is by the suffix `-probe`. Item ids are `mc-rN-REGION-NAME`, so
   only a content named `*-probe` would be hidden by it; excluding descendants of the group `mc-probe` (or
   opacity 0) would be exact. No real node of today is hidden.
c3 mc.sh:88 the date regex is English-shaped (`WEEKDAY D MONTH`); a German header ("15." form) would fail.
   Fine while the door runs in English; note it next to the regex.

## Checked, no finding
m3: only the region that holds the keys content gives way (region.rs:641-648); Left and Drawer keep their
items (audit.rs:296); docs configuration.md:707 and mc-style.md:119 match. m6: header_clear waits up to 6 s
for two equal readings 0.25 s apart, then 10 readings over ~1 s; any overlap or a date that is not
`\S+ D MONTH(  HH:MM)?` fails, and a fully cut date (" ") gives "no date in the header". shorten() always
ends a cut with "…" (region.rs:2116), which the regex rejects, so "NOVEM…" fails. The probe copies sit
under an opacity-0 group (region.rs:1071) and are no words of the screen: the exclusion is right.
r1 items for regressions: the lane's commits touch only chat.rs/view.rs (type change), region.rs items(),
audit.rs, give.rs tests, mc.sh and docs. mc-30 app_lines, sh-10 view.rs header/Stop, sh-5 capsule and
sh-7 keys words are unchanged; their unit tests pass at HEAD. Tiling/kit changes in the diff come from the
merge c2d0a116, not reviewed.

BLOCK: none. MAJOR: none.
Verdict: MERGE-OK
