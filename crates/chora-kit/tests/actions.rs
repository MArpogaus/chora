//! One YAML binding gives one `System` action (KIT2): the spellings of
//! the shared actions are the same in every look, and a look's own names
//! go to the look.

use chora_kit::actions::{Action, Step, System, Turn};
use chora_kit::settings::Common;
use chora_kit::volume::Change;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
enum Own {
    NextScene,
    Scene(String),
}

type Bound = Action<Own>;

#[allow(clippy::unwrap_used, reason = "a helper of tests")]
fn one(binding: &str) -> Bound {
    let common: Common<Own> =
        serde_saphyr::from_str(&format!("bindings:\n  K: {binding}\n")).unwrap();
    assert_eq!(common.bindings.len(), 1);
    common.bindings["K"].clone()
}

#[test]
fn one_binding_gives_one_system_action() {
    let table: [(&str, System); 15] = [
        ("lock", System::Lock),
        ("power", System::Power),
        ("screenshot", System::Screenshot),
        ("keys", System::Keys),
        ("chat", System::Chat),
        ("terminal", System::Terminal),
        ("launcher", System::Launcher),
        ("close", System::Close),
        ("drag", System::Drag),
        ("{run: fuzzel}", System::Run("fuzzel".into())),
        ("{give: xp}", System::Give("xp".into())),
        ("{volume: up}", System::Volume(Change::Up)),
        ("{volume: {set: 30}}", System::Volume(Change::Set(30))),
        ("{desk: next}", System::Desk(Step::Next)),
        ("{cube: left}", System::Cube(Turn::Left)),
    ];
    for (yaml, system) in table {
        assert_eq!(one(yaml), Action::System(system), "{yaml}");
    }
}

#[test]
fn a_name_of_the_look_goes_to_the_look() {
    assert_eq!(one("next_scene"), Action::Look(Own::NextScene));
    assert_eq!(
        one("{scene: agenda}"),
        Action::Look(Own::Scene("agenda".into()))
    );
}

#[test]
fn an_action_keeps_its_spelling_through_yaml() {
    for yaml in [
        "lock",
        "{run: foot}",
        "{volume: mute}",
        "{desk: previous}",
        "next_scene",
    ] {
        let action = one(yaml);
        let text = serde_saphyr::to_string(&action).unwrap();
        let back: Bound = serde_saphyr::from_str(&text).unwrap();
        assert_eq!(back, action, "{yaml} -> {text}");
    }
}

#[test]
fn a_mistake_names_the_mistake() {
    let err = serde_saphyr::from_str::<Common<Own>>("bindings:\n  K: sideways\n").unwrap_err();
    assert!(err.to_string().contains("sideways"), "{err}");
    let err =
        serde_saphyr::from_str::<Common<Own>>("bindings:\n  K: {volume: sideways}\n").unwrap_err();
    assert!(err.to_string().contains("sideways"), "{err}");
    let err =
        serde_saphyr::from_str::<Common<Own>>("bindings:\n  K: lock\nextra: 1\n").unwrap_err();
    assert!(err.to_string().contains("extra"), "{err}");
}

#[test]
fn the_common_settings_default_off() {
    let common: Common<Own> = serde_saphyr::from_str("bindings: {}\n").unwrap();
    assert!(!common.test_accept_injected_keys);
}

#[test]
fn the_shared_scripts_run_lock_power_and_screenshot() {
    assert_eq!(System::Lock.script(), Some(("power", &["lock"][..])));
    assert_eq!(System::Power.script(), Some(("power", &[][..])));
    assert_eq!(System::Screenshot.script(), Some(("screenshot", &[][..])));
    assert_eq!(System::Chat.script(), None);
}
