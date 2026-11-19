use super::{ExportMethod, json, runtime};

/// The methods; those that start programs or write files are privileged
/// (D62), because a caller with `methods.call` alone must not run commands;
/// so are those that write the words of the agent or a notification.
/// One entry per method.
#[allow(clippy::too_many_lines)]
pub(crate) fn exported_methods() -> Vec<ExportMethod> {
    let lines = json!({"type": "array", "items": {"type": "string"}});
    let panels = json!({"type": "array", "items": {"type": "object"}, "description":
        "each with id, cell [column, row, width, height], title and lines"});
    vec![
        runtime::method(
            "scene.show",
            "Shows a scene with a short cross-fade.",
            json!({"name": {"type": "string"}}),
            &["name"],
        ),
        runtime::method(
            "scene.list",
            "Answers the scenes and the one that shows.",
            json!({}),
            &[],
        ),
        runtime::privileged(runtime::method(
            "scene.save",
            "Saves a scene in its file and shows the change: a grid of panels, or an app.",
            json!({
                "name": {"type": "string"},
                "grid": {"type": "array", "items": {"type": "integer"}, "description": "columns and rows, by default [4, 3]"},
                "panels": panels,
                "app": {"type": "string", "description": "a shell command whose window fills the text block, or the name of a content of kind app, such as shell"},
            }),
            &["name"],
        )),
        runtime::privileged(runtime::method(
            "scene.rename",
            "Renames a scene: its file, its entry in the order and the scene.",
            json!({"name": {"type": "string"}, "to": {"type": "string"}}),
            &["name", "to"],
        )),
        runtime::privileged(runtime::method(
            "scene.move",
            "Moves a scene to the number to, from 1, and saves the order; the scenes \
             between move by one.",
            json!({"name": {"type": "string"}, "to": {"type": "integer", "minimum": 1}}),
            &["name", "to"],
        )),
        runtime::method(
            "panel.set",
            "Sets the title and the lines of a panel of a scene, until the next start.",
            json!({
                "scene": {"type": "string"},
                "panel": {"type": "string"},
                "title": {"type": "string"},
                "lines": lines,
            }),
            &["scene", "panel", "lines"],
        ),
        runtime::privileged(runtime::method(
            "hud.set",
            "Sets the HUD line: the state of the agent, such as listening, thinking or \
             working, and a short message; both empty hide it.",
            json!({"state": {"type": "string"}, "text": {"type": "string"}}),
            &[],
        )),
        runtime::privileged(runtime::method(
            "hud.say",
            "Shows the text in the HUD word by word over duration_ms, the time that the \
             voice takes to speak it; a state as in hud.set shows before it.",
            json!({
                "state": {"type": "string"},
                "text": {"type": "string"},
                "duration_ms": {"type": "number"},
            }),
            &["text", "duration_ms"],
        )),
        runtime::method(
            "pane.focus",
            "Gives the keyboard to the app of the scene NAME, else to its page; a plugin that \
             shows the page as a pane calls it while the user flies into the pane.",
            json!({"name": {"type": "string"}}),
            &["name"],
        ),
        runtime::privileged(runtime::method(
            "app.open",
            "Starts a shell command as an app in a full-screen scene of its own.",
            json!({"command": {"type": "string"}}),
            &["command"],
        )),
        runtime::method(
            "region.list",
            "Answers the regions around the page with what each shows, and the contents.",
            json!({}),
            &[],
        ),
        runtime::privileged(runtime::method(
            "region.toggle",
            "Shows or hides a region: header, footer, left, right or drawer; without shown \
             it flips. A docked app in it starts and takes the keyboard.",
            json!({"region": {"type": "string"}, "shown": {"type": "boolean"}}),
            &["region"],
        )),
        runtime::privileged(runtime::method(
            "region.put",
            "Puts a content into a region and slot over the rules, until the next start; \
             without a region the content follows the rules again.",
            json!({
                "content": {"type": "string"},
                "region": {"type": "string"},
                "slot": {"type": "integer"},
            }),
            &["content"],
        )),
        runtime::method(
            "content.set",
            "Sets the lines and the title of a text content, such as the notes.",
            json!({"content": {"type": "string"}, "title": {"type": "string"}, "lines": lines}),
            &["content", "lines"],
        ),
        runtime::method(
            "content.add",
            "Adds a line to a text content, such as a note; the oldest go after 12 lines.",
            json!({"content": {"type": "string"}, "line": {"type": "string"}}),
            &["content", "line"],
        ),
        runtime::privileged(runtime::method(
            "region.solo",
            "Shows a content alone in a region over any scene, such as the test overlay in \
             the drawer; without shown it flips. Answers shown.",
            json!({
                "content": {"type": "string"},
                "region": {"type": "string"},
                "shown": {"type": "boolean"},
            }),
            &["content", "region"],
        )),
        runtime::privileged(runtime::method(
            "agent.set",
            "Adds or changes an agent: its state (idle, listening, thinking, working, \
             speaking, question, blocked, done or asleep), its task, its last words, its \
             voice, its plan, its open question and the last lines of its transcript; a \
             missing field stays.",
            json!({
                "name": {"type": "string"},
                "state": {"type": "string"},
                "task": {"type": "string"},
                "said": {"type": "string"},
                "voice": {"type": "string"},
                "plan": lines.clone(),
                "question": {"type": "string"},
                "log": lines.clone(),
            }),
            &["name"],
        )),
        runtime::method(
            "agent.focus",
            "Shows the agents scene with one agent alone and its transcript; an empty \
             name shows all agents again.",
            json!({"name": {"type": "string"}}),
            &["name"],
        ),
        runtime::method(
            "agent.remove",
            "Removes an agent.",
            json!({"name": {"type": "string"}}),
            &["name"],
        ),
        runtime::method("agent.list", "Answers the agents.", json!({}), &[]),
        runtime::privileged(runtime::method(
            "notify.push",
            "Shows a notice through the notification server; a critical one stays until the person \
             closes it.",
            json!({
                "title": {"type": "string"},
                "body": {"type": "string"},
                "urgency": {"type": "string", "enum": ["low", "normal", "critical"]},
            }),
            &["title"],
        )),
        runtime::privileged(runtime::method(
            "notify.clear",
            "Closes the notices that mc showed.",
            json!({}),
            &[],
        )),
        chora_kit::give::method(),
        runtime::state_method(),
    ]
}
