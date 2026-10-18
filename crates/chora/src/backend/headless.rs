//! The headless backend: a GLES renderer on the software EGL device of
//! Mesa, with virtual outputs and no screen, seat or input devices. `chora
//! --headless` runs a whole session on it, for tests and agents; the unit
//! tests render captures with it in the test process.
//!
//! A frame clock renders every output into a texture at 60 Hz, with damage,
//! so that frame callbacks, presentation feedback, transactions and the
//! watchdog work as on a screen. Input comes only through `input.inject`.

use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;
use smithay::{
    backend::{
        allocator::{Fourcc, dmabuf::Dmabuf},
        egl::{EGLContext, EGLDevice, EGLDisplay},
        renderer::{
            Bind, ImportDma, Offscreen,
            damage::OutputDamageTracker,
            gles::{GlesRenderer, GlesTexture},
        },
    },
    output::{Mode, Output, PhysicalProperties, Scale, Subpixel},
    reexports::{
        calloop::timer::{TimeoutAction, Timer},
        wayland_protocols::wp::presentation_time::server::wp_presentation_feedback,
        wayland_server::backend::GlobalId,
    },
    utils::{Buffer, Size},
    wayland::presentation::Refresh,
};

use chora_scene::ipc::RpcError;

use super::Backend;
use crate::{
    ipc::{params_of, rejected},
    output::frame_duration,
    render,
    state::State,
};

/// The refresh rate of the virtual outputs, in mHz.
const REFRESH: i32 = 60_000;
/// The answer of other backends to `output.add` and `output.remove`.
const ONLY_HEADLESS: &str = "only the headless backend adds and removes outputs";

/// The renderer and the virtual outputs of the headless backend.
pub struct Headless {
    renderer: GlesRenderer,
    outputs: Vec<Virtual>,
    /// The number of the last output, for the next name.
    last: u32,
}

/// A virtual output: the texture that its frames go to.
struct Virtual {
    output: Output,
    global: GlobalId,
    damage_tracker: OutputDamageTracker,
    texture: Option<GlesTexture>,
}

/// A virtual output of the command line or of `output.add`.
#[derive(Debug, PartialEq)]
pub struct Spec {
    pub size: (i32, i32),
    pub scale: f64,
    pub position: Option<(i32, i32)>,
}

/// Parses `WxH[@SCALE][+X+Y]`, comma separated, for example
/// `1280x800@1.5,1920x1080+1280+0`.
pub fn parse_specs(text: &str) -> Result<Vec<Spec>, String> {
    text.split(',').map(parse_spec).collect()
}

fn parse_spec(text: &str) -> Result<Spec, String> {
    let bad = || format!("{text:?} is no output WxH[@SCALE][+X+Y]");
    let (rest, position) = match text.split_once('+') {
        Some((rest, pos)) => {
            let (x, y) = pos.split_once('+').ok_or_else(bad)?;
            let pos = (x.parse().map_err(|_| bad())?, y.parse().map_err(|_| bad())?);
            (rest, Some(pos))
        }
        None => (text, None),
    };
    let (size, scale) = match rest.split_once('@') {
        Some((size, scale)) => (size, scale.parse::<f64>().map_err(|_| bad())?),
        None => (rest, 1.0),
    };
    let (w, h) = size.split_once('x').ok_or_else(bad)?;
    let size: (i32, i32) = (w.parse().map_err(|_| bad())?, h.parse().map_err(|_| bad())?);
    if size.0 < 1 || size.1 < 1 || size.0 > 16384 || size.1 > 16384 {
        return Err(format!("{text:?}: a side has 1 to 16384 pixels"));
    }
    if !(0.1..=10.0).contains(&scale) {
        return Err(format!("{text:?}: the scale is outside of 0.1 to 10"));
    }
    crate::output::logical_ok(size, scale).map_err(|e| format!("{text:?}: {e}"))?;
    Ok(Spec {
        size,
        scale,
        position,
    })
}

impl Headless {
    /// A renderer on the software device of Mesa, so that the pixels do
    /// not depend on a GPU.
    pub fn new() -> Result<Self, String> {
        let device = EGLDevice::enumerate()
            .map_err(|e| format!("no EGL devices: {e}"))?
            .find(EGLDevice::is_software)
            .ok_or("no software EGL device; the headless backend needs the Mesa drivers")?;
        // SAFETY: the display and its context live as long as the renderer,
        // and only the thread of the event loop uses them.
        #[allow(unsafe_code)]
        let renderer = unsafe {
            let display = EGLDisplay::new(device).map_err(|e| e.to_string())?;
            let context = EGLContext::new(&display).map_err(|e| e.to_string())?;
            GlesRenderer::new(context).map_err(|e| e.to_string())?
        };
        Ok(Self {
            renderer,
            outputs: Vec::new(),
            last: 0,
        })
    }

    /// The renderer.
    pub fn gles(&mut self) -> &mut GlesRenderer {
        &mut self.renderer
    }

    /// Whether the renderer can use a buffer of a client.
    pub fn import_dmabuf(&mut self, dmabuf: &Dmabuf) -> bool {
        self.renderer.import_dmabuf(dmabuf, None).is_ok()
    }
}

/// Starts the headless backend with its outputs and its frame clock.
pub fn init(state: &mut State, specs: &[Spec]) -> Result<(), Box<dyn std::error::Error>> {
    state.backend = Some(Backend::Headless(Box::new(Headless::new()?)));
    for spec in specs {
        add(state, spec)?;
    }
    let frame = Duration::from_secs_f64(1000.0 / f64::from(REFRESH));
    state
        .loop_handle
        .insert_source(Timer::immediate(), move |_, (), state| {
            render_all(state);
            TimeoutAction::ToDuration(frame)
        })?;
    Ok(())
}

/// Adds a virtual output; it renders from the next tick of the clock.
fn add(state: &mut State, spec: &Spec) -> Result<Output, String> {
    let Some(Backend::Headless(headless)) = &mut state.backend else {
        return Err(ONLY_HEADLESS.into());
    };
    headless.last += 1;
    let output = Output::new(
        format!("HEADLESS-{}", headless.last),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "chora".into(),
            model: "headless".into(),
            serial_number: String::new(),
        },
    );
    let global = output.create_global::<State>(&state.display_handle);
    let mode = Mode {
        size: spec.size.into(),
        refresh: REFRESH,
    };
    output.change_current_state(Some(mode), None, Some(Scale::Fractional(spec.scale)), None);
    output.set_preferred(mode);
    headless.outputs.push(Virtual {
        damage_tracker: OutputDamageTracker::from_output(&output),
        output: output.clone(),
        global,
        texture: None,
    });
    state.add_output(output.clone());
    if let Some(position) = spec.position {
        output.change_current_state(None, None, None, Some(position.into()));
    }
    state.outputs_configured();
    Ok(output)
}

/// The parameters of `output.add`.
#[derive(Deserialize)]
struct OutputAdd {
    spec: String,
}

/// The parameters of `output.remove`.
#[derive(Deserialize)]
struct OutputRemove {
    output: String,
}

impl State {
    /// `output.add` and `output.remove`: virtual outputs come and go, as
    /// monitors do, for tests of hotplug. Only the headless backend has
    /// them; one output stays.
    pub fn ipc_headless_output(
        &mut self,
        conn: u64,
        method: &str,
        params: Value,
    ) -> Result<Value, RpcError> {
        if self.ipc.name(conn).is_none() {
            return Err(rejected("send hello first"));
        }
        if !matches!(self.backend, Some(Backend::Headless(_))) {
            return Err(rejected(ONLY_HEADLESS));
        }
        if method == "output.add" {
            let OutputAdd { spec } = params_of(params)?;
            let [spec] = <[Spec; 1]>::try_from(parse_specs(&spec).map_err(rejected)?)
                .map_err(|_| rejected("output.add takes one output"))?;
            let output = add(self, &spec).map_err(rejected)?;
            let info = self
                .output_info()
                .into_iter()
                .find(|i| i.name == output.name());
            return Ok(serde_json::json!(info));
        }
        let OutputRemove { output: name } = params_of(params)?;
        let Some(Backend::Headless(headless)) = &mut self.backend else {
            return Err(rejected(ONLY_HEADLESS));
        };
        let index = headless
            .outputs
            .iter()
            .position(|v| v.output.name() == name)
            .ok_or_else(|| rejected(format!("no virtual output {name:?}")))?;
        if headless.outputs.len() == 1 {
            return Err(rejected("the last output stays"));
        }
        let gone = headless.outputs.remove(index);
        super::tty::retire_global(&self.display_handle, &self.loop_handle, gone.global);
        self.remove_output(&gone.output);
        self.outputs_configured();
        Ok(Value::Null)
    }
}

/// One tick of the frame clock: renders the damage of every output that
/// is on, and sends the frame callbacks.
fn render_all(state: &mut State) {
    let Some(Backend::Headless(headless)) = &state.backend else {
        return;
    };
    let outputs: Vec<Output> = headless
        .outputs
        .iter()
        .map(|v| v.output.clone())
        .filter(|o| state.outputs.contains(o))
        .collect();
    for output in outputs {
        render_output(state, &output);
    }
}

/// Renders one output, and tells the clients, the lock and the watchdog.
fn render_output(state: &mut State, output: &Output) {
    let start = std::time::Instant::now();
    match render_frame(state, output) {
        Ok(damaged) => {
            if damaged {
                let mut feedback = render::take_feedback(state, output);
                feedback.presented(
                    state.clock.now(),
                    Refresh::fixed(frame_duration(output)),
                    0,
                    wp_presentation_feedback::Kind::Vsync,
                );
            }
            let generation = state.lock_generation();
            state.frame_shown(output, generation);
        }
        Err(err) => tracing::warn!(output = output.name(), "{err}"),
    }
    state.frame_rendered(&output.name(), frame_duration(output), start.elapsed());
    render::send_frames(state, output);
}

/// Renders the damaged parts of an output into its texture. Returns
/// whether the frame had damage.
fn render_frame(state: &mut State, output: &Output) -> Result<bool, String> {
    let view = render::View::of(state, output);
    let State {
        backend,
        scene,
        windows,
        assets,
        lock,
        ..
    } = state;
    let Some(Backend::Headless(headless)) = backend else {
        return Ok(false);
    };
    let Some(target) = headless.outputs.iter_mut().find(|v| v.output == *output) else {
        return Ok(false);
    };
    let gles = &mut headless.renderer;
    let mode = output.current_mode().ok_or("the output has no mode")?.size;
    let size: Size<i32, Buffer> = (mode.w, mode.h).into();
    // A new texture after a mode change; its age 0 means a full redraw.
    let mut age = 1;
    if target
        .texture
        .as_ref()
        .is_none_or(|t| smithay::backend::renderer::Texture::size(t) != size)
    {
        target.texture = Some(
            gles.create_buffer(Fourcc::Abgr8888, size)
                .map_err(|e| e.to_string())?,
        );
        age = 0;
    }
    let mut parts = render::Parts {
        scene,
        windows,
        assets,
        lock: lock.as_ref(),
        depth: 0,
    };
    let elements = render::output_elements(&mut parts, gles, output, &view);
    let texture = target.texture.as_mut().expect("the texture was created");
    let mut framebuffer = gles.bind(texture).map_err(|e| e.to_string())?;
    let result = target
        .damage_tracker
        .render_output(gles, &mut framebuffer, age, &elements, [0.0, 0.0, 0.0, 1.0])
        .map_err(|e| format!("rendering failed: {e:?}"))?;
    // The watchdog measures the work of the renderer, as on a screen.
    let _ = result.sync.wait();
    Ok(result.damage.is_some())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::testing::Core;

    #[test]
    fn output_specs_parse() {
        assert_eq!(
            parse_specs("1280x800@1.5,1920x1080+1280+0").unwrap(),
            vec![
                Spec {
                    size: (1280, 800),
                    scale: 1.5,
                    position: None
                },
                Spec {
                    size: (1920, 1080),
                    scale: 1.0,
                    position: Some((1280, 0))
                },
            ]
        );
        assert_eq!(
            parse_spec("640x480@2+-640+10").unwrap().position,
            Some((-640, 10))
        );
        assert!(parse_spec("1638x1638@0.1").is_ok(), "16380 logical");
        for bad in [
            "",
            "1280",
            "1280x",
            "0x800",
            "1280x800@0",
            "1280x800+5",
            "axb",
            "16384x16384@0.1",
            "1700x1700@0.1",
        ] {
            assert!(parse_spec(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn other_backends_refuse_virtual_outputs() {
        let mut core = Core::new();
        core.add_output("test-1", (64, 48));
        let mut wm = core.named("wm");
        for (method, params) in [
            ("output.add", json!({"spec": "64x48"})),
            ("output.remove", json!({"output": "test-1"})),
        ] {
            let (_, error) = wm.fails(&mut core, method, params);
            assert!(error.contains(ONLY_HEADLESS), "{method}: {error}");
        }
    }

    #[test]
    fn virtual_outputs_come_go_and_change() {
        let mut core = Core::with_renderer();
        let mut wm = core.named("wm");
        let first = wm.ok(&mut core, "output.add", json!({"spec": "64x48"}));
        assert_eq!(first["name"], "HEADLESS-1");
        let second = wm.ok(&mut core, "output.add", json!({"spec": "32x32@2+0+48"}));
        assert_eq!(
            (second["position"].clone(), second["size"].clone()),
            (json!([0, 48]), json!([16, 16]))
        );
        let scaled = json!({"output": "HEADLESS-1", "scale": 2.0, "transform": "90"});
        let info = wm.ok(&mut core, "output.configure", scaled);
        assert_eq!(info["size"], json!([24, 32]));
        wm.ok(&mut core, "output.remove", json!({"output": "HEADLESS-2"}));
        let (_, error) = wm.fails(&mut core, "output.remove", json!({"output": "HEADLESS-1"}));
        assert!(error.contains("last output"), "{error}");
        render_all(&mut core.state);
    }
}
