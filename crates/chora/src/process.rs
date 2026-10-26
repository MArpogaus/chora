//! The one program the core starts: the startup command (D30). It gets
//! the environment of the core, with `WAYLAND_DISPLAY` and `CHORA_SOCKET`
//! of this compositor and without the variables of a host session. Every
//! other program comes from `choractl run` outside the core (D42); the
//! core forks nothing else (gate check 7 of 0.7).

use std::{
    ffi::OsStr,
    path::Path,
    process::{Child, Command},
};

/// Starts the startup command with the environment of this compositor.
pub fn start(argv: &[String], wayland: &OsStr, socket: &Path) -> std::io::Result<Child> {
    let (program, rest) = argv
        .split_first()
        .ok_or_else(|| std::io::Error::other("no program"))?;
    let mut command = Command::new(program);
    command
        .args(rest)
        .env("WAYLAND_DISPLAY", wayland)
        .env(chora_scene::ipc::SOCKET_ENV, socket);
    for variable in chora_scene::ipc::HOST_VARIABLES {
        command.env_remove(variable);
    }
    command.spawn()
}

/// Waits for the started child in a thread, so that it never stays a
/// zombie.
pub fn reap(mut child: Child) {
    std::thread::spawn(move || child.wait());
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    /// Fork and exec sites of production code.
    const FORK_NEEDLES: [&str; 5] = ["Command::new", "posix_spawn", "execv(", "execve(", "fork("];

    /// Every `.rs` file under `dir`, recursively.
    fn sources(dir: &Path) -> Vec<std::path::PathBuf> {
        let mut found = Vec::new();
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                found.extend(sources(&path));
            } else if path.extension().is_some_and(|e| e == "rs") {
                found.push(path);
            }
        }
        found
    }

    /// Gate check 7 of 0.7: no fork, `Command::new` or spawn of a process
    /// in `crates/chora/src` except the startup command in this file. The
    /// production code of a file ends at its first test module.
    #[test]
    fn the_core_forks_nothing_but_its_startup_command() {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut sites = Vec::new();
        for path in sources(&src) {
            if path.file_name().is_some_and(|n| n == "process.rs") {
                continue; // the startup command of the core lives here
            }
            let text = fs::read_to_string(&path).unwrap();
            let production = production_code(&text);
            sites.extend(
                FORK_NEEDLES
                    .iter()
                    .filter(|needle| production.contains(**needle))
                    .map(|needle| format!("{}: {needle}", path.display())),
            );
        }
        assert!(sites.is_empty(), "sites: {sites:#?}");
    }

    /// The lines of production code: a file ends where a test module
    /// opens, not at a lone `#[cfg(test)]` that decorates one item.
    fn production_code(text: &str) -> String {
        let opens = |l: &str| {
            let t = l.trim_start();
            (t.starts_with("mod ") || t.starts_with("pub mod "))
                && (t.contains("tests") || t.contains("testing"))
        };
        let lines: Vec<&str> = text.lines().collect();
        for i in 0..lines.len() {
            if !lines[i].contains("#[cfg(test)]") {
                continue;
            }
            if opens(lines[i]) {
                return lines[..i].join("\n");
            }
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim().is_empty() {
                j += 1;
            }
            if j < lines.len() && opens(lines[j]) {
                return lines[..i].join("\n");
            }
        }
        text.to_string()
    }

    #[test]
    fn production_code_cuts_at_test_modules_only() {
        let text =
            "fn a() {}\n#[cfg(test)]\nfn gated() {}\n#[cfg(test)]\nmod tests {}\nCommand::new\n";
        assert_eq!(
            production_code(text),
            "fn a() {}\n#[cfg(test)]\nfn gated() {}",
            "the scan keeps decorated items and stops at the module"
        );
    }
}
