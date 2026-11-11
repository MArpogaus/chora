//! `record.step`, `record.frame` and `record.stop`: a recording with a
//! stepped scene clock, for demo media and tests. While a recording runs,
//! the animations of the scene and the `time` of the shaders stand still
//! between the steps and move by exactly one frame time with each step,
//! so that the frames do not depend on the speed of the renderer.
//!
//! A frame shows the pointer as the injection mark of the screen, as a
//! plain arrow, or not at all. The mark itself stays on the screen: only
//! the picture of a recording, which a person asked for on the admin
//! socket, may leave it out.
//!
//! The methods work only on the admin socket and only on the headless
//! backend: a recording stops the animations of the whole session and
//! shows every output. The recording ends with `record.stop` or with its
//! connection.

use chora_scene::ipc::{Capture, RpcError};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    backend::Backend,
    capture::Pixels,
    ipc::{params_of, rejected},
    state::State,
};

/// The highest frame rate of a recording.
const MAX_FPS: u32 = 240;

/// A running recording: the connection that steps it and the count of
/// its steps.
pub struct Recording {
    conn: u64,
    fps: u32,
    frame: u64,
}

impl Recording {
    /// The scene time since the start of the recording, in milliseconds.
    #[allow(clippy::cast_precision_loss, reason = "a count of frames")]
    fn time_ms(&self) -> f64 {
        self.frame as f64 * 1000.0 / f64::from(self.fps)
    }
}

/// The parameters of `record.step`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    fps: u32,
}

/// The parameters of `record.frame`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Frame {
    #[serde(default)]
    output: Option<String>,
    #[serde(default)]
    pointer: Pointer,
}

/// How a frame shows the pointer.
#[derive(Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Pointer {
    /// As the screen does: no cursor, and the mark of injected input.
    #[default]
    Mark,
    /// A plain arrow at the pointer, and no mark.
    Arrow,
    /// Neither.
    None,
}

/// The outline of the arrow in logical pixels, from its tip.
const ARROW: [[f64; 2]; 7] = [
    [0.0, 0.0],
    [0.0, 16.0],
    [4.0, 12.5],
    [7.0, 18.5],
    [9.5, 17.3],
    [6.5, 11.5],
    [11.5, 11.5],
];

/// Whether a point lies inside the arrow.
fn in_arrow(x: f64, y: f64) -> bool {
    let mut inside = false;
    for (i, a) in ARROW.iter().enumerate() {
        let b = ARROW[(i + 1) % ARROW.len()];
        if (a[1] > y) != (b[1] > y) && x < a[0] + (y - a[1]) * (b[0] - a[0]) / (b[1] - a[1]) {
            inside = !inside;
        }
    }
    inside
}

/// Draws a white arrow with a black outline of one logical pixel, with
/// its tip at a point of the picture in logical pixels.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "pixels inside the picture"
)]
fn draw_arrow(pixels: &mut Pixels, scale: f64, tip: [f64; 2]) {
    let (left, top) = ((tip[0] - 2.0) * scale, (tip[1] - 2.0) * scale);
    let (right, bottom) = ((tip[0] + 14.0) * scale, (tip[1] + 21.0) * scale);
    let columns = left.max(0.0) as u32..(right.max(0.0) as u32).min(pixels.width);
    for row in top.max(0.0) as u32..(bottom.max(0.0) as u32).min(pixels.height) {
        for column in columns.clone() {
            let x = (f64::from(column) + 0.5) / scale - tip[0];
            let y = (f64::from(row) + 0.5) / scale - tip[1];
            let near = [-1.0, 0.0, 1.0]
                .iter()
                .any(|dx| [-1.0, 0.0, 1.0].iter().any(|dy| in_arrow(x + dx, y + dy)));
            let color = match (in_arrow(x, y), near) {
                (true, _) => [255, 255, 255, 255],
                (false, true) => [0, 0, 0, 255],
                (false, false) => continue,
            };
            let at = (row as usize * pixels.width as usize + column as usize) * 4;
            pixels.rgba[at..at + 4].copy_from_slice(&color);
        }
    }
}

impl State {
    /// `record.step`, `record.frame` and `record.stop`.
    pub fn ipc_record(
        &mut self,
        conn: u64,
        method: &str,
        params: Value,
    ) -> Result<Value, RpcError> {
        if !["record.step", "record.frame", "record.stop"].contains(&method) {
            return Err(RpcError::new(
                RpcError::METHOD_NOT_FOUND,
                format!("no method {method:?}"),
            ));
        }
        if !self.ipc.admin(conn) {
            return Err(RpcError::new(
                RpcError::ADMIN_SOCKET_ONLY,
                format!("{method} works only on the admin socket of choractl"),
            ));
        }
        if !matches!(self.backend, Some(Backend::Headless(_))) {
            return Err(rejected(format!(
                "{method} works only on the headless backend"
            )));
        }
        if self.recording.as_ref().is_some_and(|r| r.conn != conn) {
            return Err(rejected("another connection records"));
        }
        match method {
            "record.step" => self.record_step(conn, &params_of(params)?),
            "record.frame" => {
                if self.lock.is_some() {
                    return Err(rejected("the session is locked"));
                }
                self.record_frame(params_of(params)?)
            }
            _ => {
                self.recording_closed(conn);
                Ok(Value::Null)
            }
        }
    }

    /// The picture of an output with the pointer as the frame wants it.
    /// Without the mark, the scene renders once without the mark of
    /// injected input; the mark stays on the screen.
    fn record_frame(&mut self, frame: Frame) -> Result<Value, RpcError> {
        let capture = Capture {
            output: frame.output,
            node: None,
            max_side: None,
        };
        if frame.pointer == Pointer::Mark {
            return self.picture(&capture);
        }
        let first = self.outputs.first().map(smithay::output::Output::name);
        let shown = capture.output.clone().or(first);
        let tip = (self.screen_spot(self.pointer_layout))
            .filter(|spot| Some(spot.output.name()) == shown)
            .filter(|_| frame.pointer == Pointer::Arrow)
            .map(|spot| [spot.local.x, spot.local.y]);
        let with_mark = self.scene.clone();
        self.scene.remove_injection();
        let picture = self.picture_with(&capture, |pixels, scale| {
            if let Some(tip) = tip {
                draw_arrow(pixels, scale, tip);
            }
        });
        self.scene = with_mark;
        picture
    }

    /// One step: the scene time moves by one frame of `fps`. The first
    /// step starts the recording at the scene time 0.
    fn record_step(&mut self, conn: u64, step: &Step) -> Result<Value, RpcError> {
        if !(1..=MAX_FPS).contains(&step.fps) {
            return Err(rejected(format!("fps is 1 to {MAX_FPS}")));
        }
        let recording = self.recording.get_or_insert(Recording {
            conn,
            fps: step.fps,
            frame: 0,
        });
        if recording.fps != step.fps {
            return Err(rejected(format!(
                "this recording runs at {} fps; record.stop ends it",
                recording.fps
            )));
        }
        recording.frame += 1;
        let (frame, time_ms) = (recording.frame, recording.time_ms());
        #[allow(clippy::cast_possible_truncation, reason = "seconds of a recording")]
        {
            self.assets.layers.clock = Some((time_ms / 1000.0) as f32);
        }
        self.animate_by(1000.0 / f64::from(step.fps));
        Ok(json!({"frame": frame, "time_ms": time_ms}))
    }

    /// Ends the recording of a connection: the clock runs again.
    pub fn recording_closed(&mut self, conn: u64) {
        if self.recording.as_ref().is_some_and(|r| r.conn == conn) {
            self.recording = None;
            self.assets.layers.clock = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use base64::{Engine, engine::general_purpose::STANDARD};
    use chora_scene::ipc::RpcError;
    use serde_json::{Value, json};

    use crate::testing::Core;

    fn opacity(core: &mut Core, wm: &mut crate::testing::Client) -> f64 {
        let nodes = wm.ok(core, "scene.query", json!({"selector": "id:r"}));
        nodes[0]["opacity"].as_f64().unwrap()
    }

    fn fading_rect(core: &mut Core) -> crate::testing::Client {
        let mut wm = core.named("wm");
        let props = json!({"size": [20, 10], "color": [1.0, 0.0, 0.0, 1.0]});
        let create = json!({"create": {"id": "r", "kind": "rect", "parent": "root",
            "props": props}});
        let animate = json!({"animate": {"id": "r", "props": {"opacity": 0.0},
            "duration": 1000.0}});
        wm.ok(core, "scene.apply", json!({"commands": [create]}));
        wm.ok(core, "scene.apply", json!({"commands": [animate]}));
        wm
    }

    fn red_of_first_pixel(picture: &Value) -> u8 {
        let png = STANDARD.decode(picture["png"].as_str().unwrap()).unwrap();
        let mut reader = png::Decoder::new(std::io::Cursor::new(png))
            .read_info()
            .unwrap();
        let mut rgba = vec![0; reader.output_buffer_size().unwrap()];
        reader.next_frame(&mut rgba).unwrap();
        rgba[0]
    }

    #[test]
    fn a_recording_works_only_on_the_admin_socket() {
        let mut core = Core::with_renderer();
        core.add_output("test-1", (64, 48));
        let mut agent = core.named("agent");
        for method in ["record.step", "record.frame", "record.stop"] {
            let (error_code, message) = agent.fails(&mut core, method, json!({"fps": 30}));
            assert_eq!(
                error_code,
                RpcError::ADMIN_SOCKET_ONLY,
                "{method}: {message}"
            );
        }
    }

    #[test]
    fn a_recording_works_only_on_the_headless_backend() {
        let mut core = Core::new();
        let mut admin = core.admin();
        let (_, message) = admin.fails(&mut core, "record.step", json!({"fps": 30}));
        assert!(message.contains("headless"), "{message}");
    }

    #[test]
    fn a_step_takes_a_frame_rate_within_its_limits() {
        let mut core = Core::with_renderer();
        let mut admin = core.admin();
        for fps in [json!(0), json!(241), json!(-30), json!("fast")] {
            admin.fails(&mut core, "record.step", json!({"fps": fps}));
        }
        let step = admin.ok(&mut core, "record.step", json!({"fps": 30}));
        assert_eq!(step["frame"], json!(1));
    }

    #[test]
    fn an_animation_moves_by_one_frame_time_per_step_and_not_between() {
        let mut core = Core::with_renderer();
        core.add_output("test-1", (64, 48));
        let mut admin = core.admin();
        admin.ok(&mut core, "record.step", json!({"fps": 30}));
        let mut wm = fading_rect(&mut core);
        std::thread::sleep(Duration::from_millis(40));
        core.dispatch();
        assert!((opacity(&mut core, &mut wm) - 1.0).abs() < 1e-9);
        for _ in 0..15 {
            admin.ok(&mut core, "record.step", json!({"fps": 30}));
        }
        let half = opacity(&mut core, &mut wm);
        assert!((half - 0.5).abs() < 1e-6, "after 15 of 30 frames: {half}");
        let step = admin.ok(&mut core, "record.step", json!({"fps": 30}));
        assert_eq!(step["frame"], json!(17));
        let ms = step["time_ms"].as_f64().unwrap();
        assert!((ms - 17.0 * 1000.0 / 30.0).abs() < 1e-6, "{ms}");
    }

    #[test]
    fn a_frame_shows_the_shader_time_of_the_steps() {
        let dir = crate::testing::scratch_dir("chora-record");
        let shader = dir.join("clock.frag");
        std::fs::write(
            &shader,
            "precision highp float;\nuniform float time;\n\
             void main() { gl_FragColor = vec4(time, 0.0, 0.0, 1.0); }\n",
        )
        .unwrap();
        let mut core = Core::with_renderer();
        core.add_output("test-1", (64, 48));
        let mut wm = core.named("wm");
        let props = json!({"size": [64, 48], "shader": shader});
        let create = json!({"create": {"id": "s", "kind": "shader", "parent": "root",
            "props": props}});
        wm.ok(&mut core, "scene.apply", json!({"commands": [create]}));
        let mut admin = core.admin();
        for _ in 0..15 {
            admin.ok(&mut core, "record.step", json!({"fps": 30}));
        }
        std::thread::sleep(Duration::from_millis(40));
        let picture = admin.ok(&mut core, "record.frame", json!({"output": "test-1"}));
        assert_eq!(picture["width"], json!(64));
        let red = red_of_first_pixel(&picture);
        assert!((127..=128).contains(&red), "time 0.5 as red: {red}");
        let again = admin.ok(&mut core, "record.frame", json!({}));
        assert_eq!(again["png"], picture["png"], "a frame without a step");
    }

    /// A shader node of the client `wm` that shows the time; the node
    /// lives as long as the client.
    fn clock_shader(core: &mut Core, name: &str) -> crate::testing::Client {
        let dir = crate::testing::scratch_dir(name);
        let shader = dir.join("clock.frag");
        std::fs::write(
            &shader,
            "precision highp float;\nuniform float time;\n\
             void main() { gl_FragColor = vec4(time * 0.01, 0.0, 0.0, 1.0); }\n",
        )
        .unwrap();
        let mut wm = core.named("wm");
        let props = json!({"size": [64, 48], "shader": shader});
        let create = json!({"create": {"id": "s", "kind": "shader", "parent": "root",
            "props": props}});
        wm.ok(core, "scene.apply", json!({"commands": [create]}));
        wm
    }

    #[test]
    fn the_stepped_time_belongs_to_one_core() {
        let mut recording = Core::with_renderer();
        recording.add_output("test-1", (64, 48));
        let _shown = clock_shader(&mut recording, "chora-record-a");
        let mut admin = recording.admin();
        for _ in 0..1500 {
            admin.ok(&mut recording, "record.step", json!({"fps": 30}));
        }
        let stepped = admin.ok(&mut recording, "record.frame", json!({}));
        let at_fifty = red_of_first_pixel(&stepped);
        assert!((126..=129).contains(&at_fifty), "50 s: {at_fifty}");
        let mut other = Core::with_renderer();
        other.add_output("test-1", (64, 48));
        let mut wm = clock_shader(&mut other, "chora-record-b");
        let grant = json!({"client": "wm", "capabilities": ["screen.capture"]});
        other.admin().ok(&mut other, "grants.set", grant);
        let running = wm.ok(&mut other, "screen.capture", json!({"output": "test-1"}));
        let red = red_of_first_pixel(&running);
        assert!(
            red < 100,
            "the other core shows its own time, not 50 s: {red}"
        );
    }

    fn pixel(picture: &Value, x: usize, y: usize) -> [u8; 4] {
        let png = STANDARD.decode(picture["png"].as_str().unwrap()).unwrap();
        let mut reader = png::Decoder::new(std::io::Cursor::new(png))
            .read_info()
            .unwrap();
        let mut rgba = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut rgba).unwrap();
        let at = (y * info.width as usize + x) * 4;
        [rgba[at], rgba[at + 1], rgba[at + 2], rgba[at + 3]]
    }

    #[test]
    fn a_frame_shows_an_arrow_for_the_pointer_and_no_injection_mark() {
        let mut core = Core::with_renderer();
        core.add_output("test-1", (64, 48));
        let mut admin = core.admin();
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        admin.ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        let to = json!({"events": [{"pointer_move": {"position": [20.0, 10.0]}}]});
        agent.ok(&mut core, "input.inject", to);
        let (yellow, white, black) = ([255, 255, 0, 255], [255, 255, 255, 255], [0, 0, 0, 255]);
        let marked = admin.ok(&mut core, "record.frame", json!({}));
        assert_eq!(pixel(&marked, 30, 20), yellow, "the mark, by default");
        assert_eq!(pixel(&marked, 22, 16), black, "no arrow, by default");
        let arrow = admin.ok(&mut core, "record.frame", json!({"pointer": "arrow"}));
        assert_eq!(pixel(&arrow, 30, 20), black, "no mark in the frame");
        assert_eq!(pixel(&arrow, 22, 16), white, "the arrow at the pointer");
        assert_eq!(pixel(&arrow, 19, 10), black, "left of the tip");
        let none = admin.ok(&mut core, "record.frame", json!({"pointer": "none"}));
        assert_eq!(pixel(&none, 30, 20), black);
        assert_eq!(pixel(&none, 22, 16), black);
        assert!(
            core.state.scene.node(chora_scene::INJECTION).is_some(),
            "the mark stays on the screen"
        );
        admin.fails(&mut core, "record.frame", json!({"pointer": "hand"}));
    }

    #[test]
    fn the_clock_runs_again_after_the_stop_and_after_the_connection() {
        let mut core = Core::with_renderer();
        core.add_output("test-1", (64, 48));
        let mut admin = core.admin();
        admin.ok(&mut core, "record.step", json!({"fps": 30}));
        assert!(core.state.recording.is_some());
        admin.ok(&mut core, "record.stop", json!({}));
        assert!(core.state.recording.is_none());
        admin.ok(&mut core, "record.step", json!({"fps": 30}));
        let mut other = core.admin();
        let (_, message) = other.fails(&mut core, "record.step", json!({"fps": 30}));
        assert!(message.contains("another"), "{message}");
        drop(admin);
        core.dispatch();
        assert!(core.state.recording.is_none());
        let mut wm = fading_rect(&mut core);
        std::thread::sleep(Duration::from_millis(40));
        core.dispatch();
        assert!(opacity(&mut core, &mut wm) < 1.0);
    }
}
