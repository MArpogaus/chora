//! The pipe to the host. The field has no connection to chora: the host
//! starts it with a pipe on stdin and stdout and without `CHORA_SOCKET`,
//! so the field, which reads text that it cannot trust, holds no right
//! that the host does not hand it line by line.
//!
//! The host writes one JSON object per line to stdin: `{"place": {...}}`,
//! `{"question": true}`, `{"keys": {"yes", "no"}}`, `{"say_failed":
//! "text"}`, `{"pick_first": "text"}`, `"clear"`, `"hide"` and `"quit"`. The field writes one per line to stdout:
//! `{"say": "text"}` and `{"event": "shown" | "keyboard" | "closed" |
//! "height", "data": {...}}`, `height` with the height of the field in
//! logical pixels when it changes. The end of stdin ends the field.

use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    os::fd::OwnedFd,
    sync::mpsc::{Receiver, Sender},
};

use chora_kit::ask::Keys;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::window::Place;

/// What the host tells the window.
#[derive(Debug)]
pub enum Msg {
    /// Show the field here.
    Place(Box<Place>),
    /// A question of the helper starts or ends.
    Question(bool),
    /// The keys that answer the question that waits.
    Keys(Keys),
    /// A text did not reach the helper.
    SayFailed(String),
    /// A text was typed before a helper was picked.
    PickFirst(String),
    /// Empty the draft: the chat goes to another helper.
    Clear,
    /// The text of a paste.
    Paste(String),
    /// Take the surface away and keep the draft.
    Hide,
    /// End the program.
    Quit,
}

/// What the window asks of the host.
#[derive(Debug)]
pub enum Request {
    /// Send a text to the helper.
    Say(String),
    /// Tell the host an event of the field.
    Emit(&'static str, Value),
}

/// A line of the host.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
enum Line {
    Place(Box<Place>),
    Question(bool),
    Keys { yes: String, no: String },
    SayFailed(String),
    PickFirst(String),
    Clear,
    Hide,
    Quit,
}

/// Tells the window loop something and wakes it.
pub struct Wake {
    msgs: Sender<Msg>,
    pipe: File,
}

impl Wake {
    /// A wake that sends over `msgs` and writes to `pipe`.
    #[must_use]
    pub fn new(msgs: Sender<Msg>, pipe: OwnedFd) -> Self {
        Self {
            msgs,
            pipe: File::from(pipe),
        }
    }

    /// Hands a message to the window loop.
    pub fn tell(&self, msg: Msg) {
        if self.msgs.send(msg).is_ok() {
            let _ = (&self.pipe).write_all(&[1]);
        }
    }
}

/// The message of a line of the host.
fn parse(line: &str) -> Result<Msg, serde_json::Error> {
    Ok(match serde_json::from_str::<Line>(line)? {
        Line::Place(place) => Msg::Place(Box::new(place.clamped())),
        Line::Question(waiting) => Msg::Question(waiting),
        Line::Keys { yes, no } => Msg::Keys(Keys { yes, no }),
        Line::SayFailed(text) => Msg::SayFailed(text),
        Line::PickFirst(text) => Msg::PickFirst(text),
        Line::Clear => Msg::Clear,
        Line::Hide => Msg::Hide,
        Line::Quit => Msg::Quit,
    })
}

/// Reads the lines of the host until its end, which ends the field.
pub fn read(input: impl std::io::Read, wake: &Wake) {
    let mut lines = BufReader::new(input).lines();
    while let Some(Ok(line)) = lines.next() {
        match parse(&line) {
            Ok(msg) => wake.tell(msg),
            Err(error) => eprintln!("chora-chat: a line of the host: {error}"),
        }
    }
    wake.tell(Msg::Quit);
}

/// The line that a request becomes.
fn render(request: Request) -> String {
    match request {
        Request::Say(text) => json!({"say": text}).to_string(),
        Request::Emit(event, data) => json!({"event": event, "data": data}).to_string(),
    }
}

/// Writes the requests to the host until the window ends or the host does.
pub fn write(requests: &Receiver<Request>, mut output: impl Write) {
    for request in requests {
        let sent = writeln!(output, "{}", render(request)).and_then(|()| output.flush());
        if sent.is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_of_the_host_is_a_message() {
        assert!(matches!(
            parse(r#"{"question": true}"#),
            Ok(Msg::Question(true))
        ));
        assert!(matches!(parse(r#""quit""#), Ok(Msg::Quit)));
        assert!(matches!(
            parse(r#"{"keys": {"yes": "Alt+y", "no": "Alt+n"}}"#),
            Ok(Msg::Keys(_))
        ));
        assert!(parse(r#"{"place": {"anchor": "top", "x": 1}}"#).is_err());
        assert!(parse("nonsense").is_err());
    }

    #[test]
    fn pick_first_is_a_line_of_the_host() {
        assert!(matches!(
            parse(r#"{"pick_first": "Hallo"}"#),
            Ok(Msg::PickFirst(text)) if text == "Hallo"
        ));
    }

    #[test]
    fn clear_is_a_line_of_the_host() {
        assert!(matches!(parse(r#""clear""#), Ok(Msg::Clear)));
    }

    #[test]
    fn hide_is_a_line_of_the_host() {
        assert!(matches!(parse(r#""hide""#), Ok(Msg::Hide)));
    }

    #[test]
    fn a_say_and_an_event_are_one_line_each() {
        let say = render(Request::Say("hi\nthere".into()));
        assert_eq!(say, r#"{"say":"hi\nthere"}"#);
        let event = render(Request::Emit("keyboard", json!({"on": true})));
        assert_eq!(event, r#"{"data":{"on":true},"event":"keyboard"}"#);
    }

    #[test]
    fn the_end_of_the_host_ends_the_field() {
        let (msgs, inbox) = std::sync::mpsc::channel();
        let (pipe, _keep) = rustix::pipe::pipe().expect("a pipe");
        let wake = Wake::new(msgs, pipe);
        read(&b"{\"question\": false}\n"[..], &wake);
        assert!(matches!(inbox.recv(), Ok(Msg::Question(false))));
        assert!(matches!(inbox.recv(), Ok(Msg::Quit)));
    }
}
