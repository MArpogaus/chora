//! `choractl record`: frames of a headless session with a stepped clock.
//!
//! Each frame moves the scene time of the core by exactly one frame time
//! (`record.step`), hands that time to every plugin that exports
//! `clock.step`, and saves the picture of the output (`record.frame`).
//! The frames then show the same motion on a fast and on a slow
//! renderer. The methods of the core work only on the admin socket and
//! only on the headless backend.

use std::path::{Path, PathBuf};

use base64::{Engine, engine::general_purpose::STANDARD};
use chora_scene::ipc::{MethodCall, Picture};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::Client;

/// The method of a plugin that takes the time of a frame.
const CLOCK: &str = "clock.step";

/// What `choractl record` takes.
#[derive(clap::Args)]
pub struct Recording {
    /// Frames per second of the scene time.
    #[arg(long, default_value_t = 30)]
    fps: u32,
    /// How many frames.
    #[arg(long)]
    frames: u32,
    /// The directory of the files frame-00001.png and on.
    #[arg(long)]
    out: PathBuf,
    /// The output; the first one if not set.
    #[arg(long)]
    output: Option<String>,
    /// A file with one request per line, sent before the step of its
    /// frame: {"frame": N, "method": "input.inject", "params": {}}.
    #[arg(long)]
    script: Option<PathBuf>,
}

/// A request of a script, sent before the step of its frame.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Cue {
    /// The frame, counted from 1.
    frame: u32,
    method: String,
    #[serde(default)]
    params: Value,
}

/// The cues of a script: one JSON object per line, sorted by frame. Empty
/// lines are skipped.
pub fn parse_script(text: &str) -> Result<Vec<Cue>, String> {
    let mut cues = Vec::new();
    for (number, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let cue: Cue =
            serde_json::from_str(line).map_err(|e| format!("script line {}: {e}", number + 1))?;
        if cue.method.starts_with("record.") {
            return Err(format!(
                "script line {}: {} belongs to the recording itself",
                number + 1,
                cue.method
            ));
        }
        cues.push(cue);
    }
    cues.sort_by_key(|c| c.frame);
    Ok(cues)
}

/// The file of a frame, counted from 1.
pub fn frame_file(out: &Path, frame: u32) -> PathBuf {
    out.join(format!("frame-{frame:05}.png"))
}

/// The clients that export `clock.step`.
fn clocks(client: &mut Client) -> Result<Vec<String>, String> {
    let methods: Value = client
        .call("methods.list", Value::Null)
        .map_err(|e| e.to_string())?;
    let names = methods.as_array().into_iter().flatten();
    Ok(names
        .filter(|m| m["name"] == CLOCK)
        .filter_map(|m| m["client"].as_str().map(String::from))
        .collect())
}

fn step_clocks(client: &mut Client, clocks: &[String], ms: Option<f64>) -> Result<(), String> {
    for name in clocks {
        let call = MethodCall {
            client: name.clone(),
            method: CLOCK.into(),
            params: json!({"ms": ms}),
        };
        client
            .call::<_, Value>("methods.call", call)
            .map_err(|e| format!("{name} {CLOCK}: {e}"))?;
    }
    Ok(())
}

fn frames(client: &mut Client, recording: &Recording, clocks: &[String]) -> Result<(), String> {
    let script = match &recording.script {
        Some(path) => parse_script(
            &std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?,
        )?,
        None => Vec::new(),
    };
    let mut cues = script.into_iter().peekable();
    for frame in 1..=recording.frames {
        while let Some(cue) = cues.next_if(|c| c.frame <= frame) {
            client
                .call::<_, Value>(&cue.method, cue.params)
                .map_err(|e| format!("frame {frame}: {}: {e}", cue.method))?;
        }
        let step: Value = client
            .call("record.step", json!({"fps": recording.fps}))
            .map_err(|e| e.to_string())?;
        step_clocks(client, clocks, step["time_ms"].as_f64())?;
        let picture: Picture = client
            .call("record.frame", json!({"output": recording.output}))
            .map_err(|e| format!("frame {frame}: {e}"))?;
        let png = STANDARD
            .decode(&picture.png)
            .map_err(|e| format!("the picture is no base64: {e}"))?;
        let file = frame_file(&recording.out, frame);
        std::fs::write(&file, png).map_err(|e| format!("{}: {e}", file.display()))?;
    }
    Ok(())
}

/// Records the frames, then lets the clocks of the core and of the
/// plugins run again, also after a failure.
pub fn record(client: &mut Client, recording: &Recording) -> Result<(), String> {
    std::fs::create_dir_all(&recording.out)
        .map_err(|e| format!("{}: {e}", recording.out.display()))?;
    let clocks = clocks(client)?;
    let recorded = frames(client, recording, &clocks);
    let released = step_clocks(client, &clocks, None);
    let stopped = client
        .call::<_, Value>("record.stop", Value::Null)
        .map_err(|e| e.to_string());
    recorded?;
    released?;
    stopped?;
    crate::print(&json!({
        "frames": recording.frames,
        "fps": recording.fps,
        "out": recording.out,
        "clocks": clocks,
    }));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_script_gives_its_cues_sorted_by_frame() {
        let text = "{\"frame\": 9, \"method\": \"input.inject\", \"params\": {\"events\": []}}\n\n\
                    {\"frame\": 2, \"method\": \"methods.call\"}\n";
        let cues = parse_script(text).unwrap();
        assert_eq!(cues.iter().map(|c| c.frame).collect::<Vec<_>>(), vec![2, 9]);
        assert_eq!(cues[0].params, Value::Null);
    }

    #[test]
    fn a_script_names_the_line_of_a_mistake() {
        let err = parse_script("{\"frame\": 1, \"method\": \"a.b\"}\n{\"frame\": }").unwrap_err();
        assert!(err.starts_with("script line 2"), "{err}");
        let err = parse_script("{\"frame\": 1, \"method\": \"record.stop\"}").unwrap_err();
        assert!(err.contains("record.stop"), "{err}");
    }

    #[test]
    fn frames_have_names_that_sort() {
        let file = frame_file(Path::new("/tmp/out"), 42);
        assert_eq!(file, Path::new("/tmp/out/frame-00042.png"));
    }
}
