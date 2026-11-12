//! The HUD folders that chora ships load, and their `chat:` blocks hold
//! the values of the look they belong to (design/hud-sprint.md, H8 to
//! H10).

use std::path::{Path, PathBuf};

use chora_kit::hud::{self, Anchor, ChatStyle, Hud, Source};

fn repo(below: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(below)
}

fn shipped(below: &str) -> Hud {
    hud::load(&repo(below)).unwrap_or_else(|errors| panic!("{below}: {errors:?}"))
}

#[test]
fn the_xp_folder_loads_with_the_balloon_chat() {
    let xp = shipped("flavors/xp/assets/assistant");
    assert_eq!(xp.font, "Tahoma");
    assert_eq!(xp.chat.style, ChatStyle::Bubble);
    assert_eq!(xp.chat.font.as_deref(), Some("Tahoma"));
    assert_eq!(xp.chat.size, 14);
    assert_eq!(xp.chat.border, Some([0x7f, 0x9d, 0xb9]));
    assert_eq!(xp.chat.place(), (Anchor::BottomRight, 340));
    assert_eq!(
        xp.chat.contrast_errors(&hud::Chat::default()),
        Vec::<String>::new()
    );
}

#[test]
fn the_colours_of_every_shipped_folder_read_well() {
    for below in [
        "plugins/chora-assistant/huds/neutral",
        "flavors/xp/assets/assistant",
        "plugins/chora-mc/hud",
        "plugins/chora-construct/hud",
    ] {
        let folder = shipped(below);
        assert_eq!(
            folder.contrast_errors(&hud::Chat::host()),
            Vec::<String>::new(),
            "{below}"
        );
    }
    let mc = shipped("plugins/chora-mc/hud");
    assert!(
        mc.panels.iter().all(|panel| panel.source != Source::Agents),
        "mc draws the helper rows in its own column"
    );
    let states: Vec<_> = mc
        .panels
        .iter()
        .filter(|panel| panel.source == Source::Voice)
        .collect();
    assert_eq!(states.len(), 2, "the state word of mc, light and dark");
    assert!(states.iter().all(|state| state.colours.accent.is_some()));
}

#[test]
fn the_xp_folder_is_one_balloon_of_the_paperclip() {
    let xp = shipped("flavors/xp/assets/assistant");
    let by = |source: Source| xp.panels.iter().find(|panel| panel.source == source);
    assert!(by(Source::Agents).is_none(), "no rows beside the paperclip");
    let clip = by(Source::Voice).expect("the paperclip");
    let line = by(Source::Say).expect("the sentence");
    assert_eq!(clip.action, Some(hud::Action::Chat));
    assert_eq!(line.action, Some(hud::Action::Chat));
    assert_eq!(
        xp.chat.style,
        hud::ChatStyle::Bubble,
        "the chat is the balloon"
    );
    assert_eq!(
        (line.fill, line.border),
        (None, None),
        "the sentence wears the look of the chat"
    );
}

/// The ring of the construct folder has its state word beside it
/// (construct style rule 14): the ring alone never names the state.
#[test]
fn the_construct_folder_names_the_voice_state_beside_its_ring() {
    let construct = shipped("plugins/chora-construct/hud");
    let ring = (construct.panels.iter())
        .find(|panel| panel.source == Source::Voice)
        .expect("a voice panel");
    assert_eq!(ring.text.as_deref(), Some("{state}"));
}

/// The kit lets a colour pass at 4.5:1 for every folder; mc reads at three
/// metres and holds 7:1 for each colour of its panels and each colour of
/// its pictures on the fill of the panel.
#[test]
fn the_state_of_mc_reads_at_seven_to_one_in_both_themes() {
    let folder = repo("plugins/chora-mc/hud");
    let mc = shipped("plugins/chora-mc/hud");
    for panel in mc.panels.iter().filter(|p| p.source == Source::Voice) {
        let fill = panel.fill.expect("the state panel has a fill");
        for (key, colour) in [
            ("text", panel.colours.text),
            ("accent", panel.colours.accent),
        ] {
            let ratio = hud::contrast(colour.expect(key), fill);
            assert!(ratio >= 7.0, "{} {key}: {ratio:.2}:1", panel.id);
        }
        let picture = panel.picture.as_deref().unwrap();
        let svg = std::fs::read_to_string(folder.join(picture.split('#').next().unwrap())).unwrap();
        let accent = panel.colours.accent.unwrap();
        let hex = format!("#{:02x}{:02x}{:02x}", accent[0], accent[1], accent[2]);
        for (at, _) in svg
            .match_indices("fill=\"#")
            .chain(svg.match_indices("stroke=\"#"))
        {
            let from = svg[at..].find('#').unwrap() + at;
            let found = &svg[from..from + 7];
            let rgb: Vec<u8> = (0..3)
                .map(|i| u8::from_str_radix(&found[1 + 2 * i..3 + 2 * i], 16).unwrap())
                .collect();
            let rgb = [rgb[0], rgb[1], rgb[2]];
            let grey = rgb.iter().max().unwrap() - rgb.iter().min().unwrap() < 0x20;
            if rgb == fill || grey {
                continue;
            }
            assert_eq!(found, hex, "{}: a mark in a colour of its own", panel.id);
            assert!(hud::contrast(rgb, fill) >= 7.0, "{} {found}", panel.id);
        }
    }
}
