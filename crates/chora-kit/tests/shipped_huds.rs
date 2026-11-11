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
