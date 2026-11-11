//! The HUD folders that chora ships load, and their `chat:` blocks hold
//! the values of the look they belong to (design/hud-sprint.md, H8 to
//! H10).

use std::path::{Path, PathBuf};

use chora_kit::hud::{self, Anchor, ChatStyle, Hud, Source, When};

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
    let rows = mc
        .panels
        .iter()
        .find(|panel| panel.source == Source::Agents)
        .expect("the rows of mc");
    assert!(rows.colours.accent.is_some() && rows.colours.muted.is_some());
}

#[test]
fn the_xp_tray_shows_rows_only_while_a_helper_works_and_no_chat_is_open() {
    let xp = shipped("flavors/xp/assets/assistant");
    let tray = xp
        .panels
        .iter()
        .find(|panel| panel.source == Source::Agents && panel.rows.is_some())
        .expect("a panel of helper rows");
    let shows = |agents: &str, chat: &str| {
        let context = hud::Context {
            wm: "xp".into(),
            agents: agents.into(),
            chat: chat.into(),
            ..hud::Context::default()
        };
        When::matches(&tray.when, &context)
    };
    assert!(shows("working", "none"));
    assert!(!shows("none", "none"));
    assert!(!shows("working", "open"));
    assert_eq!(tray.action, Some(hud::Action::Chat));
}
