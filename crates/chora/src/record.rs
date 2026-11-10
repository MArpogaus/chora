//! `record.step`, `record.frame` and `record.stop`: a recording with a
//! stepped scene clock, for demo media and tests. While a recording runs,
//! the animations of the scene and the `time` of the shaders stand still
//! between the steps and move by exactly one frame time with each step,
//! so that the frames do not depend on the speed of the renderer.
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
                let Frame { output } = params_of(params)?;
                self.picture(&Capture {
                    output,
                    node: None,
                    max_side: None,
                })
            }
            _ => {
                self.recording_closed(conn);
                Ok(Value::Null)
            }
        }
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
        crate::gl::step_clock(Some((time_ms / 1000.0) as f32));
        self.animate_by(1000.0 / f64::from(step.fps));
        Ok(json!({"frame": frame, "time_ms": time_ms}))
    }

    /// Ends the recording of a connection: the clock runs again.
    pub fn recording_closed(&mut self, conn: u64) {
        if self.recording.as_ref().is_some_and(|r| r.conn == conn) {
            self.recording = None;
            crate::gl::step_clock(None);
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

    /// The `time` of the shaders is one per process: the tests that step
    /// it run one after the other.
    static STEPS: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn one_recording() -> std::sync::MutexGuard<'static, ()> {
        STEPS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

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
        let _one = one_recording();
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
        let _one = one_recording();
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
        let _one = one_recording();
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

    #[test]
    fn the_clock_runs_again_after_the_stop_and_after_the_connection() {
        let _one = one_recording();
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
