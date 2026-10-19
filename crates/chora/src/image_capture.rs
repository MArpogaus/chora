//! `ext-image-copy-capture`: newer screenshot tools and screen sharing
//! capture outputs and toplevels (D37).
//!
//! The sources are outputs, through `ext-output-image-capture-source`, and
//! toplevels of `ext-foreign-toplevel-list`, through
//! `ext-foreign-toplevel-image-capture-source`. A session offers shm
//! buffers in ARGB8888 and XRGB8888 in the size of its source. The core
//! renders an output as on screen, and a toplevel as `screen.capture`
//! shows its node, without the cursor. A frame fails while the session is
//! locked. When the size of a source changes, the session gets the new
//! size, and the frame fails, so that the client allocates a new buffer.
//! Frames wait for the next tick of `copies`.

use std::time::Duration;

use smithay::{
    output::{Output, WeakOutput},
    reexports::wayland_server::{backend::ClientId, protocol::wl_shm},
    utils::Size,
    wayland::{
        foreign_toplevel_list::ForeignToplevelHandle,
        image_capture_source::{
            ImageCaptureSource, ImageCaptureSourceHandler, OutputCaptureSourceHandler,
            OutputCaptureSourceState, ToplevelCaptureSourceHandler, ToplevelCaptureSourceState,
        },
        image_copy_capture::{
            BufferConstraints, CaptureFailureReason, Frame, ImageCopyCaptureHandler,
            ImageCopyCaptureState, Session, SessionRef,
        },
    },
};

use crate::{
    copies::{self, Pictures},
    render,
    security::may_capture,
    shm,
    state::State,
};

/// The globals of the capture protocols, and the sessions that run.
pub struct ImageCapture {
    copy: ImageCopyCaptureState,
    outputs: OutputCaptureSourceState,
    toplevels: ToplevelCaptureSourceState,
    sessions: Vec<Session>,
}

impl ImageCapture {
    /// Creates the globals.
    pub fn new(display: &smithay::reexports::wayland_server::DisplayHandle) -> Self {
        Self {
            copy: ImageCopyCaptureState::new_with_filter::<State, _>(display, may_capture),
            outputs: OutputCaptureSourceState::new_with_filter::<State, _>(display, may_capture),
            toplevels: ToplevelCaptureSourceState::new_with_filter::<State, _>(
                display,
                may_capture,
            ),
            sessions: Vec::new(),
        }
    }
}

/// What a source shows.
enum Target {
    Output(WeakOutput),
    /// A surface node.
    Node(String),
}

impl State {
    fn target(&self, source: &ImageCaptureSource) -> Option<TargetNow> {
        match source.user_data().get::<Target>()? {
            Target::Output(weak) => weak.upgrade().map(TargetNow::Output),
            Target::Node(id) => self
                .windows
                .contains_key(id)
                .then(|| TargetNow::Node(id.clone())),
        }
    }

    /// The size of a source in pixels.
    fn source_size(&self, target: &TargetNow) -> Option<(u32, u32)> {
        match target {
            TargetNow::Output(output) => {
                let size = render::physical_size(output);
                Some((size.w.try_into().ok()?, size.h.try_into().ok()?))
            }
            TargetNow::Node(id) => self.node_capture_size(id),
        }
    }
}

/// A source that still shows something.
enum TargetNow {
    Output(Output),
    Node(String),
}

fn constraints((w, h): (u32, u32)) -> Option<BufferConstraints> {
    Some(BufferConstraints {
        size: Size::from((i32::try_from(w).ok()?, i32::try_from(h).ok()?)),
        shm: vec![wl_shm::Format::Argb8888, wl_shm::Format::Xrgb8888],
        dma: None,
    })
}

/// A frame that waits for the next tick of `copies`.
pub struct Waiting {
    session: SessionRef,
    frame: Frame,
}

impl Waiting {
    /// The client of the session.
    pub fn client(&self) -> Option<ClientId> {
        self.session.client().map(|c| c.id())
    }

    /// Writes the picture of the source into the buffer, or fails the
    /// frame. A source of a new size gives the session the new size first.
    pub fn serve(self, state: &mut State, pictures: &mut Pictures) {
        let Waiting { session, frame } = self;
        // A client that went gets nothing.
        if session.client().is_none() {
            return;
        }
        let target = match ready(state, &session) {
            Ok(target) => target,
            Err(reason) => {
                frame.fail(reason);
                return;
            }
        };
        match write(state, pictures, &target, &frame.buffer()) {
            Ok(()) => {
                let now: Duration = state.clock.now().into();
                frame.success(smithay::utils::Transform::Normal, None, now);
            }
            Err(err) => {
                tracing::warn!(%err, "an image copy frame failed");
                frame.fail(CaptureFailureReason::BufferConstraints);
            }
        }
    }
}

/// Writes the picture of a source of this tick into a buffer.
fn write(
    state: &mut State,
    pictures: &mut Pictures,
    target: &TargetNow,
    buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer,
) -> Result<(), String> {
    let picture = match target {
        TargetNow::Output(output) => state.output_picture(pictures, output),
        TargetNow::Node(id) => state.node_picture(pictures, id),
    }?;
    shm::write(buffer, picture.width, picture.height, &picture.rgba)
}

/// The source of a session, if a frame of it can capture now. A source of
/// a new size gives the session the new size first.
fn ready(state: &State, session: &SessionRef) -> Result<TargetNow, CaptureFailureReason> {
    let target = state
        .target(&session.source())
        .ok_or(CaptureFailureReason::Stopped)?;
    if state.lock.is_some() {
        return Err(CaptureFailureReason::Unknown);
    }
    let size = state.source_size(&target).and_then(constraints);
    if size.as_ref().map(|c| c.size) != session.current_constraints().map(|c| c.size) {
        if let Some(new) = size {
            session.update_constraints(new);
        }
        return Err(CaptureFailureReason::BufferConstraints);
    }
    Ok(target)
}

impl ImageCaptureSourceHandler for State {}

impl OutputCaptureSourceHandler for State {
    fn output_capture_source_state(&mut self) -> &mut OutputCaptureSourceState {
        &mut self.image_capture.outputs
    }

    fn output_source_created(&mut self, source: ImageCaptureSource, output: &Output) {
        source
            .user_data()
            .insert_if_missing_threadsafe(|| Target::Output(output.downgrade()));
    }
}

impl ToplevelCaptureSourceHandler for State {
    fn toplevel_capture_source_state(&mut self) -> &mut ToplevelCaptureSourceState {
        &mut self.image_capture.toplevels
    }

    fn toplevel_source_created(
        &mut self,
        source: ImageCaptureSource,
        toplevel: ForeignToplevelHandle,
    ) {
        let identifier = toplevel.identifier();
        let node = self
            .foreign_toplevels
            .iter()
            .find(|(_, handle)| handle.identifier() == identifier)
            .map(|(id, _)| id.clone());
        if let Some(node) = node {
            source
                .user_data()
                .insert_if_missing_threadsafe(|| Target::Node(node));
        }
    }
}

impl ImageCopyCaptureHandler for State {
    fn image_copy_capture_state(&mut self) -> &mut ImageCopyCaptureState {
        &mut self.image_capture.copy
    }

    fn capture_constraints(&mut self, source: &ImageCaptureSource) -> Option<BufferConstraints> {
        let target = self.target(source)?;
        constraints(self.source_size(&target)?)
    }

    fn new_session(&mut self, session: Session) {
        self.image_capture.sessions.push(session);
    }

    fn frame(&mut self, session: &SessionRef, frame: Frame) {
        let waiting = Waiting {
            session: session.clone(),
            frame,
        };
        if self.waiting_frames(waiting.client().as_ref()) >= copies::MAX_WAITING {
            waiting.frame.fail(CaptureFailureReason::Unknown);
            return;
        }
        self.copies.ext.push(waiting);
        self.arm_copies();
    }

    fn session_destroyed(&mut self, session: SessionRef) {
        self.image_capture.sessions.retain(|s| *s != session);
    }
}
