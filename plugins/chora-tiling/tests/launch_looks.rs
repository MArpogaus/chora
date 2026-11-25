//! rtm1-2: `--launch looks` against a core whose `clients.list` fails shows
//! one notice and exits with a failure.

use chora_kit::testing::FakeCore;

#[test]
fn a_failed_clients_list_shows_one_notice() {
    let core = FakeCore::start();
    core.refuse_next("clients.list");
    let log = core.record(|method, params| {
        (method == "methods.call" || method == "clients.list").then(|| format!("{method} {params}"))
    });
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_chora-tiling"))
        .args(["--launch", "looks"])
        .env(chora_scene::ipc::SOCKET_ENV, core.socket())
        .env("LC_ALL", "C")
        .env_remove("LANGUAGE")
        .env_remove("LANG")
        .status()
        .unwrap();
    assert!(!status.success());
    let lines = log.lock().unwrap().clone();
    let shown = lines
        .iter()
        .filter(|l| l.starts_with("methods.call") && l.contains("notify.show"))
        .count();
    assert_eq!(shown, 1, "notices shown: {lines:?}");
    assert!(
        lines
            .iter()
            .any(|l| l.contains("Other desktops not available")),
        "{lines:?}"
    );
}
