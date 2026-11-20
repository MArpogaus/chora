# mc-p2-fix (RC 2 and shared bar; P2 plan row 3 FIX mc)

## r1

Lane: cloud, branch claude/mc-p2-fix from 79c8d2b5. Step 0: `git fetch --unshallow`, `cargo check --workspace` green. The merge of origin/claude/autonomous-dev-session-3w-i33pg7 was already up to date. Gate (`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,keys,chat-keys,doors,mc-paths,mc-sofa,mc-handover,matrix,layout`): all 13 cells PASS, no red cell; the run says "partial run: nothing is pushed" as it should for a subset. Branch pushed at 17d9a798. Captures in reports/mc-p2-fix/.

| item | red | fix | test | gate cell | capture |
|---|---|---|---|---|---|
| mc-42 BLOCK | e772e0ca | b9198b8e | cell terminal-back | mc-paths | before-mc-42-cook-011.png, after-mc-42-terminal-back.png |
| mc-43 MAJOR | 916e6294 | b9198b8e | doors mc: no toggle of the column over the probe window, keys-key runs on a window page; terminal-back also presses Super+F1 there | doors, mc-paths | before-mc-43-sofa-014.png, after-mc-43-terminal-back-keys.png |
| mc-44 MAJOR | 4a268cc4 | 12f97cd3 | cell launcher-none | mc-sofa | none (fuzzel stand-in rows) |
| mc-45 verify | none | none | guard 17d9a798, cell chat-focus | mc-paths | nurse 011 (read) |

### Causes

- mc-42, mc-43: a program that mc adopts (Foot from the launcher, the cook's page 7) gets a scene without head or panels, and `shown` hid both sides for such a scene. So no column: no doors (Programs, Lock, Power, Screenshot, Volume), and the key list, which lives in the column, had no place to draw while mc counted it open. Second cause of mc-42: the window held the keyboard after every turn, so plain Left and Escape went to Foot.
- mc-44: mc ran the real fuzzel (`run: fuzzel`), which closes without a word for a name it does not know; tiling and xp run the kit list (`chora_kit::launcher`, the twins xp-2 and ti-2).

### What changed

1. The column shows beside the window of every scene that exists (`region/items.rs`, `shown`). The window box shrinks to the text block, as for an app scene since mc-6.
2. Keys on a window page. mc cannot read a key that the core gives to a window, and the core refuses a plain-key grab to mc (`input.grab-any`, api.md). The rule is therefore about who holds the keyboard:
   - Super keys (Super+Right, Super+Left, Super+Return, a started program): the window has the keyboard, as before. mc takes only bindings with Super, so the shell keeps Left, Escape, Ctrl+g and every key that a client gets in normal typing (cell terminal-keys unchanged and green).
   - Plain keys of a remote (Right, Left, Escape, Home) that turn to a window page: the page keeps the keyboard (a node of mc), so the next Left, Escape or Right still turns or goes back.
   - Return on such a page gives the window the keyboard; a click on the window does too (core).
   Why (POLS, remote path): a person who walks the pages with the arrows is never trapped by a page; a person who typed into a window is never robbed of a key. Limit: once the window holds the keyboard, a remote without Super cannot leave it by key; the doors of the column are clickable and Super+F1 draws the list beside the window. If the PM wants a key out for that case, it needs a media or consumer key in the core's grab list (XF86Back, XF86HomePage); that is a core change, not taken here.
3. `Host::run` of mc turns `fuzzel` into `chora-mc --launch fuzzel` (`acts.rs`, `main.rs`): installed programs as rows, text that matches nothing shows the list again under `Nothing found for "TEXT"`, a command of an installed program runs. Configs with `run: fuzzel` get it too. The doors cell launcher-chat of mc now reads the row `Chat with a helper` as tiling and xp do.
4. Docs: `docs/configuration.md`, mc `order` item, has the rule of items 1 to 3.

### mc-45

Nurse shot 011 (after Escape) shows the chat closed and the page with its column; shots 012 and 013 show the chat open again, which is the end state that check.py read (holder None because the holder was the chat of the host). Cell chat-focus (Super+a, a request, Escape): state.get names `mc` as the holder, Right turns the page. Not a bug; the cell stays as a guard (green on the first run, no red commit).

### Open, not done here

- The page of a window does not say how to type into it after a plain-key turn (Return). The short key list names Super with the arrows on a window page. A row `Return  Type in the window` on such a page would say it; not built (PM to rule).
- Before captures of mc-43 and mc-42 are the persona captures of the round (cook 011, sofa 014), not a rebuilt old tree; the red cells are the proof of the old behaviour.
- check.py contracts T2, T4:volume (sofa), T6 (nurse) were not rerun with a persona; the cells above are the proof.
