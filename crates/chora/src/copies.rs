//! The pace of the screencopy protocols. A frame waits for the next tick,
//! one refresh later. Each tick renders each output or node once for all
//! the frames that wait for it, and each client has at most `MAX_WAITING`
//! frames waiting; a further one fails. So no client can make the core
//! render more than once per refresh and source.

use std::{collections::HashMap, rc::Rc, time::Duration};

use smithay::{
    output::Output,
    reexports::{
        calloop::timer::{TimeoutAction, Timer},
        wayland_server::backend::ClientId,
    },
};

use crate::{capture::Pixels, image_capture, screencopy, state::State};

/// The most frames of one client that wait for a tick.
pub const MAX_WAITING: usize = 4;

/// The frames that wait for the next tick.
#[derive(Default)]
pub struct Copies {
    pub wlr: Vec<screencopy::Copy>,
    pub ext: Vec<image_capture::Waiting>,
    /// Whether the timer of the next tick runs.
    armed: bool,
}

/// The pictures of one tick, by output or node.
pub type Pictures = HashMap<String, Result<Rc<Pixels>, String>>;

impl State {
    /// How many frames of a client wait.
    pub fn waiting_frames(&self, client: Option<&ClientId>) -> usize {
        let wlr = self
            .copies
            .wlr
            .iter()
            .filter(|c| c.client().as_ref() == client);
        let ext = self
            .copies
            .ext
            .iter()
            .filter(|w| w.client().as_ref() == client);
        wlr.count() + ext.count()
    }

    /// Starts the timer of the next tick, unless it runs.
    pub fn arm_copies(&mut self) {
        if self.copies.armed {
            return;
        }
        let millihertz = self
            .outputs
            .first()
            .and_then(Output::current_mode)
            .map_or(60_000, |m| m.refresh);
        let refresh =
            Duration::from_micros(1_000_000_000 / u64::from(millihertz.max(1_000).unsigned_abs()));
        let inserted =
            self.loop_handle
                .insert_source(Timer::from_duration(refresh), |_, (), state| {
                    state.copies.armed = false;
                    state.serve_copies();
                    TimeoutAction::Drop
                });
        match inserted {
            Ok(_) => self.copies.armed = true,
            Err(err) => tracing::warn!(?err, "no timer for screencopy frames"),
        }
    }

    /// Serves the frames that wait, with one picture per source.
    fn serve_copies(&mut self) {
        let wlr = std::mem::take(&mut self.copies.wlr);
        let ext = std::mem::take(&mut self.copies.ext);
        let mut pictures = Pictures::new();
        for copy in wlr {
            copy.serve(self, &mut pictures);
        }
        for waiting in ext {
            waiting.serve(self, &mut pictures);
        }
    }

    /// The picture of an output in this tick.
    pub fn output_picture(
        &mut self,
        pictures: &mut Pictures,
        output: &Output,
    ) -> Result<Rc<Pixels>, String> {
        let key = format!("output {}", output.name());
        if let Some(picture) = pictures.get(&key) {
            return picture.clone();
        }
        let picture = self.output_pixels(output).map(Rc::new);
        pictures.insert(key, picture.clone());
        picture
    }

    /// The picture of a node in this tick.
    pub fn node_picture(
        &mut self,
        pictures: &mut Pictures,
        id: &str,
    ) -> Result<Rc<Pixels>, String> {
        let key = format!("node {id}");
        if let Some(picture) = pictures.get(&key) {
            return picture.clone();
        }
        let picture = self.node_pixels(id).map(Rc::new);
        pictures.insert(key, picture.clone());
        picture
    }
}
