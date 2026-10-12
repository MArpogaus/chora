//! The examples of the docs parse as the types of the API, so that the docs
//! and the code agree (#3). A comment `<!-- parses: KIND -->` before a code
//! block names the type.

use chora_scene::{Command, ipc::Inject};

use crate::view::{Manifest, View, parse_yaml};

const DOCS: [(&str, &str); 2] = [
    ("api.md", include_str!("../../../docs/api.md")),
    (
        "configuration.md",
        include_str!("../../../docs/configuration.md"),
    ),
];

/// The marked code blocks of a Markdown file: the kind and the code.
fn examples(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        let Some(kind) = line
            .strip_prefix("<!-- parses: ")
            .and_then(|l| l.strip_suffix(" -->"))
        else {
            continue;
        };
        assert!(
            lines.next().is_some_and(|l| l.starts_with("```")),
            "no code block after the mark {kind}"
        );
        let code: Vec<&str> = lines
            .by_ref()
            .take_while(|l| !l.starts_with("```"))
            .collect();
        out.push((kind.to_string(), code.join("\n")));
    }
    out
}

fn check(kind: &str, code: &str) -> Result<(), String> {
    match kind {
        "commands" => parse_yaml::<Vec<Command>>(code).map(drop),
        "command" => parse_yaml::<Command>(code).map(drop),
        "inject" => parse_yaml::<Inject>(code).map(drop),
        "view" => parse_yaml::<View>(code).map(drop),
        "manifest" => parse_yaml::<Manifest>(code).map(drop),
        other => Err(format!("unknown kind {other}")),
    }
}

#[test]
fn every_marked_example_parses() {
    let mut count = 0;
    for (file, text) in DOCS {
        for (kind, code) in examples(text) {
            if let Err(err) = check(&kind, &code) {
                panic!("an example of {file} is no {kind}: {err}\n{code}");
            }
            count += 1;
        }
    }
    assert!(count >= 7, "only {count} examples");
}
