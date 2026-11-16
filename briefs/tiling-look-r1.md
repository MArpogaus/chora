# tiling look r1 (rc 1): waybar in the cyberpunk style, wobbly on

Model: sonnet, effort high (design judgment across bar, borders and motion; owner cares about this look)
Lane: cloud, branch claude/tiling-look (session branch 66588880); merge the session branch first. One round.
Stop at 250 k context. Read findings/tiling.md (ti-21, ti-6, ti-19), design/looks/tiling*.md,
vision-card.md, dev-rules.md. Owner 11-16 09:01: "Waybar does not mach cyberpunk looks of remaining
style in tiling. Get inspiration from hyperland rices." and "Why no wobbely windows?"

1. ti-21 MAJOR: restyle the waybar of chora-tiling (plugins' waybar config and CSS) to the look
   of the rest of tiling: same palette tokens, glow and border language, font, island shapes.
   Study 3 to 5 well-known public Hyprland rices (r/unixporn, GitHub dotfiles); write down what
   you take from each in the report (ideas only; copy no file or asset whose licence you have not
   checked, and credit any you adapt). Both themes (Ink and Paper) and 1x, 2x.
2. Wobbly: on by default in tiling.yaml for floating windows and drags (crates/chora-kit
   wobbly.rs exists; xp has it on). A cell shows a drag of a floating window wobbles.
3. ti-6 and ti-19 (tooltips stuck after click) if the bar change touches them; nothing else.
Done when: a pixel cell pins the bar palette against the border colours, the wobbly cell passes,
tests/merge-gate.sh once. Before and after captures in the report. Report to the PM session
session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines) and as
reports/tiling-look-r1.md on claude/session-log. Rules as briefs/flakes-r2.md, push only
claude/tiling-look and that report; never end a turn to wait for a long background job.
