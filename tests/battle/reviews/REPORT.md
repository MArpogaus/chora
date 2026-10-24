# Overnight run — Review-Schleife 0.6 + Battle-Test

State lesen: `cat target/review-loop/state.txt` und
`tail -30 target/review-loop/log.txt` und `tail target/battletest/log.txt`.

## Phase 1 — Review (Runden über `git diff v0.5.4..HEAD`)

- Treiber: `target/review-loop/loop.sh` (Runde → Audit → Fix → nächste),
  endet mit `PHASE=battle` nach zwei CLEAN-Runden.
- Runde 1: 2 Findings. Fix `2c2125e` (remove-to Re-Check am Transaktionsende);
  Finding "decoding underflow" vom Reviewer selbst empirisch verworfen.
- Fix-Schritt 1 brachte zusätzlich `683145f` (max 16 Live-Outputs) und
  `6d492ab` (frühe Surface-Requests über 64 werden verworfen).
- Runde 2: "No findings to report", Audit sauber (10 Diff-Aufrufe, 0 Repo-Mutationen)
  → als CLEAN #1 gewertet. Zählung korrigiert: Reviewer durften nach /tmp schreiben.
- Ab Runde 3 läuft die Schleife mit korrigiertem Audit + CLEAN-Erkennung.

## Phase 2 — Battle (wartet auf PHASE=battle)

- Treiber: `target/battletest/battle.sh`, alle Pässe W1–W8, findet-Protokoll
  `target/battletest/findings.md`, Fix-Subagent pro Pass mit neuen Findings.
  Ende nach zwei kompletten grünen Pässen → `target/battletest/done.md`.
- Workflows: W1 Tiling, W2 XP-Session, W3 Zwei-WM-Drag, W4 Hotplug,
  W5 Last (snake + Limits), W6 Sandbox, W7 Notfall-Reset,
  W8 Übergröße (40 Fenster, 108 Layouts, 50 Hotplugs, 20 Plugin-Restarts).
- Vorab-Smoke alle acht grün. Fixtegel während des Smoke-Laufs:
  `1dbc003` probe `--server-deco` (XP ohne Server-Deko hat keine Titelknoten).
- Alle Design-Reibung: `target/battletest/FRICTION.md`
  (u. a. Transform-Matrix spaltens dominant, inject Auto-Release,
  SIGUSR1 entzieht Grants, Taskbar-Klick auf aktives Fenster,
  kein Knoten-Budget sondern Watchdog-Grenze bei ~14.6k Knoten,
  Sandbox-Grenzen des Containers).

## Verifiziert nach Bauart

- `cargo fmt`, `clippy -D warnings` grün; `cargo test --workspace` mit den
  9 vorbestehenden GPU-Fehlern (capture/headless/clients), unverändert.
- Nie gepusht, nie History umgeschrieben, `services/voice/` nicht angefasst.

## Aufraumen

- Battle-Infra lebt in `target/` (gitignore!) → nach dem Lauf nach
  `tests/battle/` umziehen und committen, sonst frisst sie der nächste
  `cargo clean`.
- Zombies: PID 1 ist `pi`, also bleibt jedes gekillte Kind ein Zombie.
