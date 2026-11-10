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

use std::time::Duration;

/// A running recording: the connection that steps it, the scene time
/// since its start and the count of its steps.
#[expect(dead_code, reason = "the methods of the recording read them")]
pub struct Recording {
    pub conn: u64,
    pub at: Duration,
    pub frame: u64,
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
        nodes[0]["props"]["opacity"].as_f64().unwrap()
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
