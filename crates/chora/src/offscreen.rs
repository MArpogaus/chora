//! An output that no screen shows: its frames go into a texture, with
//! damage. The headless backend renders its outputs with it.
//!
//! A virtual output (D69) is such an output that a client adds on any
//! backend. It is an output to a window manager, with a `wl_output`, a
//! camera, a work area, an overlay root and layer surfaces. No list of
//! screens holds it: it lives in a list of its own, and the pointer never
//! enters it.

use std::time::Instant;

use chora_scene::{Scene, Uniform, ipc::RpcError};
use serde::Deserialize;
use serde_json::Value;
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            Bind, Offscreen, Texture,
            damage::OutputDamageTracker,
            gles::{GlesRenderer, GlesTexture},
        },
    },
    output::{Mode, Output, PhysicalProperties, Scale, Subpixel},
    reexports::{
        wayland_protocols::wp::presentation_time::server::wp_presentation_feedback,
        wayland_server::backend::GlobalId,
    },
    utils::{Buffer, Size},
    wayland::presentation::Refresh,
};

use crate::{
    backend::headless::{Spec, parse_specs},
    ipc::{params_of, rejected},
    output::frame_duration,
    render,
    state::State,
};

/// The most virtual outputs of a session.
const MAX_VIRTUALS: usize = 16;
/// The most pixels of a side of a virtual output.
const MAX_SIDE: i32 = 4096;
/// The most pixels of all virtual outputs together: 256 MiB of RGBA.
const MAX_PIXELS: i64 = 1 << 26;
/// The distance of the slots in the layout. The screens lie right of and
/// below the origin, the slots left of and above it, so no screen shows
/// the scene area of a virtual output by default.
const SLOT_STEP: i32 = 262_144;
/// The refresh rate of a virtual output in a session without a screen,
/// in mHz.
const REFRESH: i32 = 60_000;

/// A virtual output: the client that added it, its place and its global.
pub struct Virtual {
    owner: String,
    slot: usize,
    global: GlobalId,
    target: OutputTexture,
    /// The screens whose frames asked for a frame of the output since it
    /// rendered last. A screen that asks again starts the next frame, so
    /// the output renders once per frame of the fastest screen.
    asked: Vec<String>,
}

impl Virtual {
    /// The last frame of the output and its version, for a shader node
    /// of a client: only the owner reads it.
    pub fn frame(&self, reader: Option<&str>) -> Option<(GlesTexture, u64)> {
        if reader != Some(&self.owner) {
            return None;
        }
        Some((self.target.texture.clone()?, self.target.version))
    }

    /// Renders a frame and notes it for the frame callbacks.
    fn render(&mut self, parts: &mut render::Parts<'_>, gles: &mut GlesRenderer) {
        let output = self.output().clone();
        let view = render::View::in_scene(parts.scene, &output);
        match self.target.render(gles, parts, &view) {
            Ok(damaged) => parts.assets.rendered_virtuals.push((output, damaged)),
            Err(err) => tracing::warn!(output = output.name(), "{err}"),
        }
    }

    /// Whether a frame of a screen starts a new frame of the output.
    fn due(&mut self, screen: &str) -> bool {
        let known = self.asked.iter().any(|s| s == screen);
        if known {
            self.asked.clear();
        }
        let first = self.asked.is_empty();
        self.asked.push(screen.to_owned());
        first
    }

    /// The output.
    pub fn output(&self) -> &Output {
        self.target.output()
    }

    /// The client that added the output.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    fn pixels(&self) -> i64 {
        let size = self
            .output()
            .current_mode()
            .map_or((0, 0).into(), |m| m.size);
        i64::from(size.w) * i64::from(size.h)
    }
}

/// The parameters of `output.add` for a virtual output.
#[derive(Deserialize)]
struct VirtualAdd {
    spec: String,
    #[serde(rename = "virtual")]
    _is_virtual: bool,
}

/// The parameters of `output.remove`.
#[derive(Deserialize)]
struct OutputRemove {
    output: String,
}

fn limit(message: String) -> RpcError {
    RpcError::new(RpcError::LIMIT_REACHED, message)
}

impl State {
    /// The screens and, after them, the virtual outputs: for what a
    /// window manager sees of an output. A screen is in `outputs` only.
    pub fn all_outputs(&self) -> Vec<Output> {
        let virtuals = self.assets.virtuals.iter().map(Virtual::output);
        self.outputs.iter().chain(virtuals).cloned().collect()
    }

    /// `output.add` and `output.remove`: a virtual output on every
    /// backend, or the hotplug of the headless backend.
    pub fn ipc_output_hotplug(
        &mut self,
        conn: u64,
        method: &str,
        params: Value,
    ) -> Result<Value, RpcError> {
        if method == "output.add" && params.get("virtual").and_then(Value::as_bool) == Some(true) {
            return self.add_virtual(conn, &params_of(params)?);
        }
        let named = params.get("output").and_then(Value::as_str);
        if method == "output.remove"
            && named.is_some_and(|n| self.assets.virtuals.iter().any(|v| v.output().name() == n))
        {
            let OutputRemove { output } = params_of(params)?;
            return self.remove_virtual(conn, &output);
        }
        self.ipc_headless_output(conn, method, params)
    }

    /// Adds a virtual output of a free client of the main socket, in the
    /// first free slot, and answers with the output.
    fn add_virtual(&mut self, conn: u64, add: &VirtualAdd) -> Result<Value, RpcError> {
        let Some(owner) = self.ipc.name(conn).map(str::to_owned) else {
            return Err(rejected("send hello first"));
        };
        if self.ipc.confined(conn) {
            return Err(RpcError::new(
                RpcError::CLIENT_SOCKET_RULE,
                "a client socket or a confined connection adds no virtual output",
            ));
        }
        let [spec] = <[Spec; 1]>::try_from(parse_specs(&add.spec).map_err(rejected)?)
            .map_err(|_| rejected("output.add takes one output"))?;
        let slot = self.free_slot(&spec)?;
        let name = format!("VIRTUAL-{}", slot + 1);
        let output = Output::new(
            name,
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "chora".into(),
                model: "virtual".into(),
                serial_number: String::new(),
            },
        );
        let global = output.create_global::<State>(&self.display_handle);
        let mode = Mode {
            size: spec.size.into(),
            refresh: REFRESH,
        };
        let column = i32::try_from(slot + 1).unwrap_or(i32::MAX);
        let place = (-column * SLOT_STEP, -SLOT_STEP);
        let scale = Scale::Fractional(spec.scale);
        output.change_current_state(Some(mode), None, Some(scale), Some(place.into()));
        output.set_preferred(mode);
        self.assets.virtuals.push(Virtual {
            owner,
            slot,
            global,
            target: OutputTexture::new(output.clone()),
            asked: Vec::new(),
        });
        self.scene
            .set_output_owner(&output.name(), self.ipc.name(conn));
        self.output_changed();
        let info = self
            .output_info()
            .into_iter()
            .find(|i| i.name == output.name());
        Ok(serde_json::json!(info))
    }

    /// The first free slot for a virtual output of a spec, within the
    /// limits of size and count.
    fn free_slot(&self, spec: &Spec) -> Result<usize, RpcError> {
        let (w, h) = spec.size;
        if w > MAX_SIDE || h > MAX_SIDE {
            return Err(rejected(format!(
                "a side of a virtual output has at most {MAX_SIDE} pixels"
            )));
        }
        if spec.position.is_some() {
            return Err(rejected("the core picks the position of a virtual output"));
        }
        let virtuals = &self.assets.virtuals;
        let slot = (0..MAX_VIRTUALS)
            .find(|slot| virtuals.iter().all(|v| v.slot != *slot))
            .ok_or_else(|| limit(format!("at most {MAX_VIRTUALS} virtual outputs live")))?;
        let pixels: i64 = virtuals.iter().map(Virtual::pixels).sum();
        if pixels + i64::from(w) * i64::from(h) > MAX_PIXELS {
            return Err(limit(format!(
                "all virtual outputs together hold at most {MAX_PIXELS} pixels"
            )));
        }
        Ok(slot)
    }

    /// Removes a virtual output for its owner or for the admin socket: its
    /// layer surfaces close and its windows leave it.
    fn remove_virtual(&mut self, conn: u64, name: &str) -> Result<Value, RpcError> {
        if self.ipc.confined(conn) {
            return Err(RpcError::new(
                RpcError::CLIENT_SOCKET_RULE,
                "a client socket or a confined connection removes no virtual output",
            ));
        }
        let virtuals = &self.assets.virtuals;
        let index = virtuals
            .iter()
            .position(|v| v.output().name() == name)
            .ok_or_else(|| rejected(format!("no virtual output {name:?}")))?;
        let owner = virtuals[index].owner();
        if !self.ipc.admin(conn) && self.ipc.name(conn) != Some(owner) {
            return Err(rejected(format!("only its owner {owner:?} removes {name}")));
        }
        self.drop_virtual(index);
        Ok(Value::Null)
    }

    /// Removes the virtual outputs of a client that ended.
    pub fn drop_virtuals_of(&mut self, owner: &str) {
        let of_the_client = |v: &Virtual| v.owner() == owner;
        while let Some(index) = self.assets.virtuals.iter().position(of_the_client) {
            self.drop_virtual(index);
        }
    }

    /// Takes a virtual output out of the session and announces it.
    fn drop_virtual(&mut self, index: usize) {
        let gone = self.assets.virtuals.remove(index);
        let name = gone.output().name();
        crate::backend::tty::retire_global(&self.display_handle, &self.loop_handle, gone.global);
        self.leave_layout(gone.target.output());
        let _ = self.scene.release_camera(&name);
        self.scene.set_output_owner(&name, None);
        self.output_changed();
        self.update_focus();
    }

    /// Whether an output is a virtual one.
    pub fn is_virtual(&self, output: &Output) -> bool {
        self.assets.virtuals.iter().any(|v| v.output() == output)
    }

    /// Gives the virtual outputs the refresh rate of the fastest screen,
    /// which paces their frames.
    pub fn sync_virtual_refresh(&self) {
        let screens = self.outputs.iter().filter_map(Output::current_mode);
        let refresh = screens.map(|m| m.refresh).max().unwrap_or(REFRESH);
        for output in self.assets.virtuals.iter().map(Virtual::output) {
            let Some(old) = output.current_mode().filter(|m| m.refresh != refresh) else {
                continue;
            };
            let mode = Mode { refresh, ..old };
            output.change_current_state(Some(mode), None, None, None);
            output.set_preferred(mode);
            output.delete_mode(old);
        }
    }
}

/// The texture that the frames of an output go to.
pub struct OutputTexture {
    output: Output,
    damage_tracker: OutputDamageTracker,
    texture: Option<GlesTexture>,
    /// Counts the frames with damage, for the readers of the texture.
    version: u64,
    /// The damage of the last frame.
    #[cfg(test)]
    damage: Vec<smithay::utils::Rectangle<i32, smithay::utils::Physical>>,
}

impl OutputTexture {
    /// A texture for the frames of an output; the first render makes it.
    pub fn new(output: Output) -> Self {
        Self {
            damage_tracker: OutputDamageTracker::from_output(&output),
            output,
            texture: None,
            version: 0,
            #[cfg(test)]
            damage: Vec::new(),
        }
    }

    /// The output that renders into the texture.
    pub fn output(&self) -> &Output {
        &self.output
    }

    /// The texture of the last frame, at the size of the mode.
    #[cfg(test)]
    pub fn texture(&self) -> Option<&GlesTexture> {
        self.texture.as_ref()
    }

    /// The damage of the last frame.
    #[cfg(test)]
    pub fn last_damage(&self) -> &[smithay::utils::Rectangle<i32, smithay::utils::Physical>] {
        &self.damage
    }

    /// Renders the damaged parts of the output into the texture. Returns
    /// whether the frame had damage. A mode change makes a new texture,
    /// which redraws in full. The render waits for the GPU, so that the
    /// watchdog measures the work of the renderer, as on a screen.
    pub fn render(
        &mut self,
        gles: &mut GlesRenderer,
        parts: &mut render::Parts<'_>,
        view: &render::View,
    ) -> Result<bool, String> {
        let mode = self
            .output
            .current_mode()
            .ok_or("the output has no mode")?
            .size;
        let size: Size<i32, Buffer> = (mode.w, mode.h).into();
        let mut age = 1;
        if self.texture.as_ref().is_none_or(|t| t.size() != size) {
            self.texture = Some(
                gles.create_buffer(Fourcc::Abgr8888, size)
                    .map_err(|e| e.to_string())?,
            );
            age = 0;
        }
        let elements = render::output_elements(parts, gles, &self.output, view);
        let texture = self.texture.as_mut().ok_or("no texture")?;
        let mut framebuffer = gles.bind(texture).map_err(|e| e.to_string())?;
        let result = self
            .damage_tracker
            .render_output(gles, &mut framebuffer, age, &elements, [0.0, 0.0, 0.0, 1.0])
            .map_err(|e| format!("rendering failed: {e:?}"))?;
        let _ = result.sync.wait();
        let damaged = result.damage.is_some();
        self.version += u64::from(damaged);
        #[cfg(test)]
        {
            self.damage = result.damage.cloned().unwrap_or_default();
        }
        parts.assets.stats.frame(damaged);
        Ok(damaged)
    }
}

/// The virtual outputs that a visible shader node reads, in the scene or
/// in the overlay of a screen: the name of each with the owner of the
/// node.
fn read_outputs(scene: &Scene, screen: &str) -> Vec<(String, Option<String>)> {
    let lists = [scene.draw_list(), scene.overlay_draw_list(screen)];
    let mut read = Vec::new();
    for node in lists.iter().flatten().map(|placed| placed.node) {
        for uniform in node.props.uniforms.values() {
            if let Uniform::Output(source) = uniform {
                read.push((source.output.clone(), node.owner.clone()));
            }
        }
    }
    read
}

/// Renders the virtual outputs that a frame of a screen reads, before the
/// screen builds its elements, so that the reader shows a texture of the
/// same frame. An output without damage keeps its texture. The outputs
/// leave the list while they render: a virtual output shows no other one.
pub fn render_virtuals(parts: &mut render::Parts<'_>, gles: &mut GlesRenderer, screen: &Output) {
    let listed = |v: &Virtual| v.output() == screen;
    if parts.assets.virtuals.is_empty() || parts.assets.virtuals.iter().any(listed) {
        return;
    }
    let read = read_outputs(parts.scene, &screen.name());
    let mut virtuals = std::mem::take(&mut parts.assets.virtuals);
    for face in &mut virtuals {
        let wanted = (face.output().name(), Some(face.owner.clone()));
        if read.contains(&wanted) && face.due(&screen.name()) {
            face.render(parts, gles);
        }
    }
    parts.assets.virtuals = virtuals;
}

/// Tells the clients on the virtual outputs that rendered with the last
/// frame of a screen: presentation feedback after damage, and the frame
/// callbacks.
pub fn present_virtuals(state: &mut State) {
    for (output, damaged) in std::mem::take(&mut state.assets.rendered_virtuals) {
        if damaged {
            let mut feedback = render::take_feedback(state, &output);
            feedback.presented(
                state.clock.now(),
                Refresh::fixed(frame_duration(&output)),
                0,
                wp_presentation_feedback::Kind::Vsync,
            );
        }
        render::send_frames(state, &output);
    }
}

/// Renders a frame of an output with `render`, then tells the clients, the
/// lock and the watchdog: presentation feedback after damage, and the
/// frame callbacks.
pub fn render_and_present(
    state: &mut State,
    output: &Output,
    render: impl FnOnce(&mut State) -> Result<bool, String>,
) {
    let start = Instant::now();
    match render(state) {
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chora_scene::ipc::RpcError;
    use serde_json::{Value, json};
    use smithay::{
        backend::{allocator::Fourcc, renderer::ExportMem},
        utils::{Buffer, Point, Rectangle, Size},
    };

    use super::*;
    use crate::{
        backend::Backend,
        testing::{
            Core,
            wayland::{Toplevel, Wayland},
        },
    };
    use wayland_client::protocol::wl_shm::Format;

    fn render_into(state: &mut State, target: &mut OutputTexture) -> Result<bool, String> {
        let view = render::View::of(state, target.output());
        let State {
            backend,
            scene,
            windows,
            assets,
            lock,
            ..
        } = state;
        let Some(Backend::Headless(headless)) = backend else {
            panic!("a headless backend");
        };
        let mut parts = render::Parts {
            scene,
            windows,
            assets,
            lock: lock.as_ref(),
            depth: 0,
        };
        target.render(headless.gles(), &mut parts, &view)
    }

    /// The first row of the texture, as RGBA.
    fn first_row(state: &mut State, target: &OutputTexture) -> Vec<[u8; 4]> {
        let Some(Backend::Headless(headless)) = &mut state.backend else {
            panic!("a headless backend");
        };
        let texture = target.texture().expect("a texture");
        let size: Size<i32, Buffer> = texture.size();
        let gles = headless.gles();
        let mapping = gles
            .copy_texture(texture, Rectangle::from_size(size), Fourcc::Abgr8888)
            .expect("a copy");
        let rgba = gles.map_texture(&mapping).expect("a mapping");
        let width = usize::try_from(size.w).expect("a width");
        rgba[..width * 4].as_chunks::<4>().0.to_vec()
    }

    fn core_with_left_half_red(size: (i32, i32)) -> (Core, Output) {
        let mut core = Core::with_renderer();
        let output = core.add_output("VIRTUAL-1", size);
        let mut wm = core.named("wm");
        let rect = json!({"commands": [{"create": {"id": "r", "kind": "rect",
            "parent": "root", "props": {"size": [size.0 / 2, size.1],
            "color": [1, 0, 0, 1]}}}]});
        wm.ok(&mut core, "scene.apply", rect);
        (core, output)
    }

    /// An output that the backend does not know renders its scene into a
    /// texture of the size of its mode: the left half red, the rest the
    /// clear color.
    #[test]
    fn an_output_renders_its_scene_into_a_texture_of_its_mode_size() {
        const IN_THE_RECT: usize = 8;
        const RIGHT_OF_THE_RECT: usize = 56;
        let (mut core, output) = core_with_left_half_red((64, 48));
        let mut target = OutputTexture::new(output);
        assert_eq!(render_into(&mut core.state, &mut target), Ok(true));
        let texture = target.texture().expect("a texture");
        assert_eq!(texture.size(), (64, 48).into());
        let row = first_row(&mut core.state, &target);
        assert_eq!(row[IN_THE_RECT], [255, 0, 0, 255], "the rect");
        assert_eq!(row[RIGHT_OF_THE_RECT], [0, 0, 0, 255], "the clear color");
    }

    #[test]
    fn render_into_redraws_only_damage() {
        let (mut core, output) = core_with_left_half_red((32, 32));
        let mut target = OutputTexture::new(output);
        let before = core.state.assets.stats.clone();
        let damaged: Vec<_> = (0..3)
            .map(|_| render_into(&mut core.state, &mut target))
            .collect();
        assert_eq!(damaged, [Ok(true), Ok(false), Ok(false)]);
        let after = core.state.assets.stats.clone();
        assert_eq!(after.frames - before.frames, 3);
        assert_eq!(after.damaged_frames - before.damaged_frames, 1);
    }

    /// A core with a window on an output that the scene shows.
    fn core_with_a_window() -> (Core, Output, Wayland, Toplevel) {
        let mut core = Core::with_renderer();
        let output = core.add_output("VIRTUAL-1", (64, 48));
        let mut wayland = Wayland::connect(&mut core);
        let window = wayland.toplevel(&mut core, "window");
        wayland.attach(&mut core, &window, (20, 20));
        let mut wm = core.named("wm");
        let attach = json!({"commands": [{"attach": {"surface": "s1", "parent": "root"}}]});
        wm.ok(&mut core, "scene.apply", attach);
        (core, output, wayland, window)
    }

    #[test]
    fn a_rendered_frame_sends_frame_callbacks() {
        let (mut core, output, mut wayland, window) = core_with_a_window();
        let mut target = OutputTexture::new(output.clone());
        for _ in 0..3 {
            wayland.frame(&mut core, &window);
            render_and_present(&mut core.state, &output, |state| {
                render_into(state, &mut target)
            });
            wayland.roundtrip(&mut core);
        }
        assert_eq!(wayland.configures(&window).frames, 3);
    }

    #[test]
    fn a_failed_render_still_sends_frame_callbacks() {
        let (mut core, output, mut wayland, window) = core_with_a_window();
        wayland.frame(&mut core, &window);
        render_and_present(&mut core.state, &output, |_| Err("no GPU".into()));
        wayland.roundtrip(&mut core);
        assert_eq!(wayland.configures(&window).frames, 1);
    }

    #[test]
    fn presentation_feedback_follows_damage_only() {
        let (mut core, output, mut wayland, window) = core_with_a_window();
        wayland.feedback(&mut core, &window);
        render_and_present(&mut core.state, &output, |_| Ok(false));
        wayland.roundtrip(&mut core);
        assert_eq!(wayland.presented(), 0, "no damage");
        render_and_present(&mut core.state, &output, |_| Ok(true));
        wayland.roundtrip(&mut core);
        assert_eq!(wayland.presented(), 1, "damage");
    }

    /// A pending lock is confirmed once every output showed a frame.
    #[test]
    fn a_rendered_frame_counts_as_shown_for_the_lock() {
        let (mut core, output, mut wayland, _window) = core_with_a_window();
        let _lock = wayland.lock(&mut core);
        assert!(!wayland.locked(), "no frame yet");
        render_and_present(&mut core.state, &output, |_| Ok(false));
        wayland.roundtrip(&mut core);
        assert!(wayland.locked());
    }

    #[test]
    fn the_watchdog_measures_the_render() {
        let (mut core, output, _wayland, _window) = core_with_a_window();
        let over_budget = frame_duration(&output) * 2;
        render_and_present(&mut core.state, &output, |_| {
            std::thread::sleep(over_budget);
            Ok(false)
        });
        assert!(core.state.watchdog.slow(&output.name()));
        render_and_present(&mut core.state, &output, |_| Ok(false));
        assert!(!core.state.watchdog.slow(&output.name()));
    }

    mod forward;

    /// The corner of the first slot in the layout.
    const SLOT_1: i32 = -262_144;

    fn add_virtual(core: &mut Core, owner: &mut crate::testing::Client, spec: &str) -> Value {
        owner.ok(core, "output.add", json!({"spec": spec, "virtual": true}))
    }

    /// A core without a backend, a virtual output of "construct" that came
    /// before the screen, and the screen.
    fn core_with_a_virtual_output() -> (Core, crate::testing::Client, Output) {
        let mut core = Core::new();
        let mut construct = core.named("construct");
        let added = add_virtual(&mut core, &mut construct, "32x32");
        assert_eq!(added["name"], "VIRTUAL-1");
        let screen = core.add_output("screen", (64, 48));
        (core, construct, screen)
    }

    fn names(outputs: &[Output]) -> Vec<String> {
        outputs.iter().map(Output::name).collect()
    }

    #[test]
    fn a_virtual_output_is_an_output_to_a_window_manager() {
        let (mut core, mut construct, _screen) = core_with_a_virtual_output();
        let list = construct.ok(&mut core, "output.list", json!({}));
        let listed: Vec<_> = list.as_array().expect("a list").iter().collect();
        assert_eq!(listed[0]["name"], "screen");
        assert_eq!(listed[0].get("virtual"), None, "a screen keeps its form");
        let face = listed[1];
        assert_eq!(
            (&face["name"], &face["virtual"], &face["owner"]),
            (&json!("VIRTUAL-1"), &json!(true), &json!("construct"))
        );
        assert_eq!(face["position"], json!([SLOT_1, SLOT_1]));
        assert_eq!(face["work_area"], json!([SLOT_1, SLOT_1, 32, 32]));
        let overlay = json!({"selector": "id:screen:VIRTUAL-1"});
        let found = construct.ok(&mut core, "scene.query", overlay);
        assert_eq!(found.as_array().map(Vec::len), Some(1), "an overlay root");
        let second = add_virtual(&mut core, &mut construct, "16x16@2");
        assert_eq!(second["name"], "VIRTUAL-2");
        assert_eq!(second["position"], json!([2 * SLOT_1, SLOT_1]));
        assert_eq!(second["size"], json!([8, 8]));
    }

    #[test]
    fn a_virtual_output_is_in_no_list_of_screens() {
        let (mut core, mut construct, screen) = core_with_a_virtual_output();
        assert_eq!(names(&core.state.outputs), ["screen"], "the pointer layout");
        assert_eq!(
            core.state.output_management.head_names(),
            ["screen"],
            "the heads of output management"
        );
        let grant = json!({"client": "construct", "capabilities": ["screen.capture"]});
        core.admin().ok(&mut core, "grants.set", grant);
        core.state.backend = Some(Backend::Headless(Box::new(
            crate::backend::headless::Headless::new().expect("a software renderer"),
        )));
        let picture = construct.ok(&mut core, "screen.capture", json!({}));
        assert_eq!(
            (&picture["width"], &picture["height"]),
            (&json!(64), &json!(48)),
            "the capture of the first screen"
        );
        let mut wayland = Wayland::connect(&mut core);
        let _lock = wayland.lock(&mut core);
        render_and_present(&mut core.state, &screen, |_| Ok(false));
        wayland.roundtrip(&mut core);
        assert!(wayland.locked(), "the lock waits for screens only");
    }

    #[test]
    fn the_pointer_never_enters_one() {
        let (mut core, mut construct, _screen) = core_with_a_virtual_output();
        let inside = Point::from((f64::from(SLOT_1) + 5.0, f64::from(SLOT_1) + 5.0));
        assert!(core.state.spot(inside).is_none(), "no spot");
        let clamped = core.state.clamp_to_outputs(inside);
        assert_eq!((clamped.x, clamped.y), (0.0, 0.0), "the nearest screen");
        let grant = json!({"client": "construct", "capabilities": ["input.inject"]});
        core.admin().ok(&mut core, "grants.set", grant);
        let onto = json!({"events": [
            {"pointer_move": {"output": "VIRTUAL-1", "position": [5.0, 5.0]}}]});
        let (_, error) = construct.fails(&mut core, "input.inject", onto);
        assert!(error.contains("no such output"), "{error}");
    }

    #[test]
    fn a_side_has_1_to_4096_pixels_and_the_core_picks_the_place() {
        let mut core = Core::new();
        let mut construct = core.named("construct");
        for bad in ["4097x8", "8x4097", "0x8", "8x8+0+0", "8x8,8x8"] {
            let add = json!({"spec": bad, "virtual": true});
            let (_, error) = construct.fails(&mut core, "output.add", add);
            assert!(!error.contains("headless"), "{bad}: {error}");
        }
        let wide = json!({"spec": "4097x8", "virtual": true});
        let (_, error) = construct.fails(&mut core, "output.add", wide);
        assert!(error.contains("4096"), "{error}");
        let no_flag = json!({"spec": "8x8", "virtual": "yes"});
        let (_, error) = construct.fails(&mut core, "output.add", no_flag);
        assert!(
            error.contains("headless"),
            "only true asks for one: {error}"
        );
    }

    fn listed(core: &mut Core, client: &mut crate::testing::Client) -> Vec<String> {
        let list = client.ok(core, "output.list", json!({}));
        let outputs = list.as_array().expect("a list").iter();
        outputs
            .map(|o| o["name"].as_str().expect("a name").to_owned())
            .collect()
    }

    #[test]
    fn the_outputs_of_an_owner_that_ended_go() {
        let (mut core, construct, _screen) = core_with_a_virtual_output();
        let mut reader = core.named("reader");
        let subscribe = json!({"events": ["output.removed"]});
        reader.ok(&mut core, "events.subscribe", subscribe);
        drop(construct);
        core.dispatch();
        assert_eq!(listed(&mut core, &mut reader), ["screen"]);
        assert_eq!(reader.event_names(&mut core), ["output.removed"]);
    }

    fn resuming(core: &mut Core) -> crate::testing::Client {
        let mut construct = core.connect();
        let hello = json!({"name": "construct", "resume": true});
        construct.ok(core, "hello", hello);
        construct
    }

    #[test]
    fn the_outputs_go_after_the_grace_time_and_a_resume_keeps_them() {
        const GRACE: Duration = Duration::from_secs(10);
        let mut core = Core::new();
        core.add_output("screen", (64, 48));
        let mut reader = core.named("reader");
        let mut construct = resuming(&mut core);
        add_virtual(&mut core, &mut construct, "32x32");
        drop(construct);
        core.dispatch();
        core.advance(GRACE.saturating_sub(Duration::from_millis(1)));
        let construct = resuming(&mut core);
        core.advance(GRACE);
        assert_eq!(listed(&mut core, &mut reader), ["screen", "VIRTUAL-1"]);
        drop(construct);
        core.dispatch();
        core.advance(GRACE);
        assert_eq!(listed(&mut core, &mut reader), ["screen"]);
    }

    #[test]
    fn a_confined_caller_neither_removes_nor_reads_one() {
        let mut faced = faced();
        let mut confined = faced.core.connect();
        let hello = json!({"name": "voice-mcp", "confined": true});
        confined.ok(&mut faced.core, "hello", hello);
        let remove = json!({"output": "VIRTUAL-1"});
        let (number, error) = confined.fails(&mut faced.core, "output.remove", remove);
        assert_eq!(number, RpcError::CLIENT_SOCKET_RULE, "{error}");
        let pane = pane(&face_shader(&faced.dir));
        let (number, error) = confined.fails(&mut faced.core, "scene.apply", pane);
        assert_eq!(number, RpcError::CLIENT_SOCKET_RULE, "{error}");
    }

    #[test]
    fn no_client_copies_a_virtual_output() {
        let (mut core, _construct, _screen) = core_with_a_virtual_output();
        let mut wayland = Wayland::connect(&mut core);
        assert!(!wayland.screencopy_offers(&mut core), "wlr-screencopy");
        assert!(!wayland.capture_session_offers(&mut core), "ext capture");
    }

    /// Output configuration turns a screen off by taking it out of the
    /// layout; a client still copies it, as before virtual outputs.
    #[test]
    fn a_screen_that_is_off_still_offers_a_copy() {
        let mut core = Core::new();
        let screen = core.add_output("screen", (64, 48));
        let mut wayland = Wayland::connect(&mut core);
        core.state.outputs.retain(|o| *o != screen);
        assert!(wayland.screencopy_offers(&mut core), "wlr-screencopy");
        assert!(wayland.capture_session_offers(&mut core), "ext capture");
    }

    /// A lock client makes a surface for every `wl_output`. The one of a
    /// virtual output gets its size, and no frame shows it.
    #[test]
    fn a_lock_surface_on_a_virtual_output_gets_its_size() {
        let (mut core, _construct, _screen) = core_with_a_virtual_output();
        let mut wayland = Wayland::connect(&mut core);
        let lock = wayland.lock(&mut core);
        let surface = wayland.lock_surface(&mut core, &lock);
        let configures = wayland.lock_configures(&surface);
        assert_eq!((configures.count, configures.size), (1, (32, 32)));
        assert!(core.state.lock_focus().is_none(), "no keyboard for it");
    }

    #[test]
    fn a_virtual_output_follows_the_refresh_of_the_fastest_screen() {
        let (mut core, _construct, screen) = core_with_a_virtual_output();
        let fast = Mode {
            size: (64, 48).into(),
            refresh: 120_000,
        };
        screen.change_current_state(Some(fast), None, None, None);
        core.state.output_changed();
        let face = core.state.all_outputs().pop().expect("the virtual output");
        assert_eq!(face.current_mode().map(|m| m.refresh), Some(120_000));
    }

    /// An `output.add` that fails at a limit whose message holds a word.
    fn hits_the_limit(core: &mut Core, owner: &mut crate::testing::Client, word: &str) {
        let add = json!({"spec": "1x1", "virtual": true});
        let (number, error) = owner.fails(core, "output.add", add);
        assert_eq!(number, RpcError::LIMIT_REACHED, "{error}");
        assert!(error.contains(word), "{error}");
    }

    #[test]
    fn the_17th_and_the_pixel_budget_fail() {
        let mut core = Core::new();
        let mut construct = core.named("construct");
        for n in 1..=16 {
            let added = add_virtual(&mut core, &mut construct, "8x8");
            assert_eq!(added["name"], format!("VIRTUAL-{n}"));
        }
        hits_the_limit(&mut core, &mut construct, "16");
        for n in 1..=16 {
            let remove = json!({"output": format!("VIRTUAL-{n}")});
            construct.ok(&mut core, "output.remove", remove);
        }
        for _ in 0..4 {
            add_virtual(&mut core, &mut construct, "4096x4096");
        }
        hits_the_limit(&mut core, &mut construct, "pixels");
        let list = construct.ok(&mut core, "output.list", json!({}));
        assert_eq!(list.as_array().map(Vec::len), Some(4));
    }

    #[test]
    fn only_the_owner_removes_it() {
        let (mut core, mut construct, _screen) = core_with_a_virtual_output();
        let remove = json!({"output": "VIRTUAL-1"});
        let mut other = core.named("xp");
        let (_, error) = other.fails(&mut core, "output.remove", remove.clone());
        assert!(error.contains("construct"), "{error}");
        let subscribe = json!({"events": ["output.removed"]});
        other.ok(&mut core, "events.subscribe", subscribe);
        construct.ok(&mut core, "output.remove", remove.clone());
        assert_eq!(other.event_names(&mut core), ["output.removed"]);
        let list = construct.ok(&mut core, "output.list", json!({}));
        assert_eq!(list.as_array().map(Vec::len), Some(1), "the screen stays");
        let again = add_virtual(&mut core, &mut construct, "32x32");
        assert_eq!(again["name"], "VIRTUAL-1", "the slot is free again");
        core.admin().ok(&mut core, "output.remove", remove);
        let (_, error) = construct.fails(&mut core, "output.remove", json!({"output": "screen"}));
        assert!(error.contains("headless"), "{error}");
    }

    const RED: u32 = 0xFFFF_0000;
    const BLUE: u32 = 0xFF00_00FF;
    /// The place and the side of the pane on the screen, and of the face.
    const PANE: (i32, i32) = (64, 32);
    const SIDE: i32 = 32;

    /// A shader file that shows the sampler `face`.
    fn face_shader(dir: &std::path::Path) -> String {
        let shader = dir.join("face.frag");
        std::fs::write(
            &shader,
            "precision mediump float;\nuniform sampler2D face;\nvarying vec2 v_uv;\n\
             void main() { gl_FragColor = texture2D(face, v_uv); }\n",
        )
        .expect("a shader file");
        shader.to_str().expect("a path").to_owned()
    }

    /// The commands of a pane of the face on the screen.
    fn pane(shader: &str) -> Value {
        let props = json!({"size": [SIDE, SIDE], "shader": shader,
            "uniforms": {"face": {"output": "VIRTUAL-1"}},
            "transform": {"translate": [PANE.0, PANE.1]}});
        json!({"commands": [
            {"create": {"id": "pane", "kind": "shader", "parent": "root", "props": props}}]})
    }

    /// A screen of 128x96 and its texture, a face of 32x32 of "construct"
    /// with a red window that fills it and a white corner above the window.
    struct Faced {
        core: Core,
        construct: crate::testing::Client,
        wayland: Wayland,
        window: Toplevel,
        screen: OutputTexture,
        dir: std::path::PathBuf,
    }

    fn faced() -> Faced {
        let mut core = Core::with_renderer();
        let screen = OutputTexture::new(core.add_output("screen", (128, 96)));
        let mut construct = core.named("construct");
        add_virtual(&mut core, &mut construct, "32x32");
        let mut wayland = Wayland::connect(&mut core);
        let window = wayland.toplevel(&mut core, "window");
        wayland.opaque(&window, (SIDE, SIDE));
        wayland.attach_filled(&mut core, &window, (SIDE, SIDE), (Format::Argb8888, RED));
        let at_face = json!({"translate": [SLOT_1, SLOT_1]});
        let corner = json!({"size": [8, 8], "color": [1, 1, 1, 1], "transform": at_face});
        let commands = json!({"commands": [
            {"attach": {"surface": "s1", "parent": "root"}},
            {"update": {"id": "s1", "props": {"transform": at_face}}},
            {"create": {"id": "corner", "kind": "rect", "parent": "root", "props": corner}},
        ]});
        construct.ok(&mut core, "scene.apply", commands);
        let dir = crate::testing::scratch_dir("chora-face");
        Faced {
            core,
            construct,
            wayland,
            window,
            screen,
            dir,
        }
    }

    impl Faced {
        fn show_pane(&mut self) {
            let pane = pane(&face_shader(&self.dir));
            self.construct.ok(&mut self.core, "scene.apply", pane);
        }

        /// Renders frames of the screen as a backend does.
        fn frames(&mut self, count: usize) {
            for _ in 0..count {
                let Self { core, screen, .. } = self;
                let output = screen.output().clone();
                render_and_present(&mut core.state, &output, |state| render_into(state, screen));
            }
        }

        /// A pixel of the screen after one more frame.
        fn pixel(&mut self, x: i32, y: i32) -> [u8; 4] {
            self.frames(1);
            let Some(Backend::Headless(headless)) = &mut self.core.state.backend else {
                panic!("a headless backend");
            };
            let texture = self.screen.texture().expect("a texture");
            let gles = headless.gles();
            let at = Rectangle::new((x, y).into(), (1, 1).into());
            let mapping = gles
                .copy_texture(texture, at, Fourcc::Abgr8888)
                .expect("a copy");
            let rgba = gles.map_texture(&mapping).expect("a mapping");
            [rgba[0], rgba[1], rgba[2], rgba[3]]
        }

        fn stats(&self) -> render::RenderStats {
            self.core.state.assets.stats.clone()
        }
    }

    #[test]
    fn a_flat_pane_shows_the_window_on_its_virtual_output() {
        let mut faced = faced();
        faced.show_pane();
        assert_eq!(faced.pixel(PANE.0 + 2, PANE.1 + 2), [255; 4], "the corner");
        assert_eq!(
            faced.pixel(PANE.0 + SIDE - 2, PANE.1 + SIDE - 2),
            [255, 0, 0, 255],
            "the window"
        );
        assert_eq!(faced.pixel(4, 4), [0, 0, 0, 255], "beside the pane");
    }

    #[test]
    fn a_static_virtual_output_renders_once() {
        let mut faced = faced();
        faced.show_pane();
        let start = faced.stats();
        faced.frames(2);
        let settled = faced.stats();
        faced.frames(60);
        let end = faced.stats();
        assert_eq!(settled.damaged_frames - start.damaged_frames, 2);
        assert_eq!(end.damaged_frames, settled.damaged_frames, "no damage");
        assert_eq!(end.offscreen(), settled.offscreen(), "no pass");
        assert_eq!(end.frames - settled.frames, 120, "the screen and the face");
    }

    impl Faced {
        /// Frames of the screen, each after a frame request of the window.
        fn frames_with_requests(&mut self, count: usize) {
            for _ in 0..count {
                let Self {
                    core,
                    wayland,
                    window,
                    ..
                } = self;
                wayland.frame(core, window);
                self.frames(1);
            }
            self.wayland.roundtrip(&mut self.core);
        }
    }

    #[test]
    fn an_unread_one_renders_nothing_and_gets_1_hz() {
        let mut faced = faced();
        let start = faced.stats();
        faced.frames_with_requests(60);
        assert_eq!(faced.stats().frames - start.frames, 60, "the screen only");
        assert_eq!(faced.wayland.configures(&faced.window).frames, 1);
        faced.core.state.time.advance(Duration::from_millis(1100));
        faced.frames_with_requests(60);
        let (waited, asked_at_the_send) = (59, 1);
        assert_eq!(
            faced.wayland.configures(&faced.window).frames,
            1 + waited + asked_at_the_send,
            "one more send after a second, which answers every request so far"
        );
    }

    #[test]
    fn a_read_one_gets_a_frame_callback_each_frame() {
        let mut faced = faced();
        faced.show_pane();
        for _ in 0..3 {
            let Faced {
                core,
                wayland,
                window,
                ..
            } = &mut faced;
            wayland.frame(core, window);
            faced.frames(1);
            faced.wayland.roundtrip(&mut faced.core);
        }
        assert_eq!(faced.wayland.configures(&faced.window).frames, 3);
    }

    #[test]
    fn a_commit_damages_only_its_pane() {
        let mut faced = faced();
        faced.show_pane();
        faced.frames(2);
        let Faced {
            core,
            wayland,
            window,
            ..
        } = &mut faced;
        wayland.attach_filled(core, window, (SIDE, SIDE), (Format::Argb8888, BLUE));
        faced.frames(1);
        let pane = Rectangle::new(PANE.into(), (SIDE, SIDE).into());
        let damage = faced.screen.last_damage().to_vec();
        assert!(!damage.is_empty(), "the commit shows");
        assert!(damage.iter().all(|d| pane.contains_rect(*d)), "{damage:?}");
        assert_eq!(
            faced.pixel(PANE.0 + SIDE - 2, PANE.1 + SIDE - 2),
            [0, 0, 255, 255]
        );
    }

    #[test]
    fn a_foreign_client_cannot_sample_it() {
        let mut faced = faced();
        let pane = pane(&face_shader(&faced.dir));
        let mut other = faced.core.named("xp");
        let (_, error) = other.fails(&mut faced.core, "scene.apply", pane.clone());
        assert!(error.contains("construct"), "{error}");
        let mut unknown = pane;
        unknown["commands"][0]["create"]["props"]["uniforms"]["face"]["output"] = json!("screen");
        let (_, error) = faced
            .construct
            .fails(&mut faced.core, "scene.apply", unknown);
        assert!(error.contains("no virtual output"), "{error}");
    }

    /// The name of a removed output goes to the next one. A pane that read
    /// the old one stays in the scene and shows nothing of the new owner.
    #[test]
    fn a_pane_of_a_former_owner_reads_nothing_of_the_next() {
        let mut faced = faced();
        faced.show_pane();
        let remove = json!({"output": "VIRTUAL-1"});
        faced.construct.ok(&mut faced.core, "output.remove", remove);
        let mut other = faced.core.named("xp");
        add_virtual(&mut faced.core, &mut other, "32x32");
        faced.frames(2);
        let shown = faced.pixel(PANE.0 + SIDE - 2, PANE.1 + SIDE - 2);
        assert_ne!(shown, [255, 0, 0, 255], "the window of the new owner");
        let start = faced.stats();
        faced.frames(3);
        assert_eq!(faced.stats().frames - start.frames, 3, "no render for it");
    }

    /// The pixels of a capture of the pane, as RGBA.
    fn pane_pixels(faced: &mut Faced) -> Vec<u8> {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let grant = json!({"client": "construct", "capabilities": ["screen.capture"]});
        faced.core.admin().ok(&mut faced.core, "grants.set", grant);
        let capture = json!({"node": "pane"});
        let picture = faced
            .construct
            .ok(&mut faced.core, "screen.capture", capture);
        let png = STANDARD
            .decode(picture["png"].as_str().expect("a picture"))
            .expect("base64");
        let mut reader = png::Decoder::new(std::io::Cursor::new(png))
            .read_info()
            .expect("a PNG file");
        let mut rgba = vec![0; reader.output_buffer_size().expect("a size")];
        reader.next_frame(&mut rgba).expect("a frame");
        rgba
    }

    #[test]
    fn a_pane_of_an_output_that_went_is_transparent() {
        let mut faced = faced();
        faced.show_pane();
        faced.frames(2);
        assert_eq!(pane_pixels(&mut faced)[..4], [255; 4], "the corner");
        let remove = json!({"output": "VIRTUAL-1"});
        faced.construct.ok(&mut faced.core, "output.remove", remove);
        faced.frames(2);
        let pixels = pane_pixels(&mut faced);
        assert!(pixels.as_chunks::<4>().0.iter().all(|p| *p == [0; 4]));
    }

    #[test]
    fn a_locked_session_renders_no_virtual_output() {
        let mut faced = faced();
        faced.show_pane();
        let _lock = faced.wayland.lock(&mut faced.core);
        let start = faced.stats();
        faced.frames(3);
        assert_eq!(faced.stats().frames - start.frames, 3, "the screen only");
    }
}
