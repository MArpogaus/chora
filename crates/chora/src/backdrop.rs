//! Backdrops: a rect with `backdrop` draws the frame behind its shape
//! blurred, under its own fill, the glass of a panel.
//!
//! The element of a backdrop sits in the list of render elements right
//! behind the element of its rect. When the renderer draws it, the frame
//! already holds everything behind it: the element copies the part of
//! the frame under its box into a texture, blurs that with a dual Kawase
//! blur, and draws the result through the shape of the rect. So it sees
//! windows, node textures and 3D contexts as they show, and nothing in
//! front of it. The blur samples only inside the box of the rect, with
//! clamped edges, so that nothing drawn later leaks in.
//!
//! The blur runs only when something behind the box changed: before the
//! frame, [`Backdrops::settle`] compares the elements behind each
//! backdrop with those of its last frame, by id, commit, box and alpha,
//! and with the damage of a changed commit. A changed backdrop gets a new
//! commit, so that the damage tracker redraws its whole box behind it
//! first. A still screen blurs nothing and has no damage.

use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use chora_scene::{Node, Uniform};
use smithay::{
    backend::renderer::{
        Offscreen, Texture,
        element::{Element, Id, Kind, RenderElement},
        gles::{GlesError, GlesTexture},
        utils::{CommitCounter, DamageSet, OpaqueRegions},
    },
    utils::{Buffer as BufferCoords, Physical, Rectangle, Scale},
};

use crate::{
    gl::{
        self,
        backdrop::{Glass, Grabbed, Window},
    },
    mesh::{self, MeshRenderer, Shape},
};

/// How long a blur that no frame used keeps its textures.
const UNUSED_FOR: Duration = Duration::from_secs(2);
/// The most halvings of the blur.
const MAX_ITERATIONS: u32 = 6;

/// A backdrop by view, node and piece: 0 for a whole rect, the others for
/// the pieces of a plane of a 3D context.
type Key = (String, String, usize);

/// The textures of a blur, shared by the cache and the element of a
/// frame: the copy of the frame, its halvings, and the result at half the
/// size of the copy.
#[derive(Default)]
pub struct Blur {
    /// The next draw blurs again.
    stale: bool,
    /// The copy of the frame behind the box.
    grabbed: Option<Grabbed>,
    /// The halvings of the copy.
    levels: Vec<GlesTexture>,
    /// The blurred copy, and the window rectangle that it came from.
    result: Option<(GlesTexture, Window)>,
}

/// An element behind a backdrop as the last frame saw it.
#[derive(PartialEq)]
struct Behind {
    id: Id,
    commit: CommitCounter,
    geometry: Rectangle<i32, Physical>,
    alpha: u32,
}

/// What a backdrop saw in its last frame.
struct Seen {
    /// A hash of what the blur reads: the box and the halvings.
    reads: u64,
    /// A hash of what the glass draws besides: the shape, the alpha and
    /// the uniforms.
    look: u64,
    behind: Vec<Behind>,
}

struct Entry {
    blur: Arc<Mutex<Blur>>,
    id: Id,
    commit: CommitCounter,
    seen: Option<Seen>,
    used: Instant,
}

/// The blurs of the backdrops, kept between frames.
#[derive(Default)]
pub struct Backdrops {
    entries: HashMap<Key, Entry>,
}

/// The number of halvings and the offset of the taps of a dual Kawase
/// blur of a radius in pixels, for a box of a size: each halving doubles
/// the reach of the taps, so the radius is about `offset * 2^(n + 1)`.
/// At least two halvings, so that the result at half the size has been
/// through an upsample.
fn iterations(radius: f64, [w, h]: [i32; 2]) -> (u32, f64) {
    let mut n = 2;
    while n < MAX_ITERATIONS && radius / f64::from(1u32 << (n + 1)) > 3.0 {
        n += 1;
    }
    while n > 1 && w.min(h) >> n < 2 {
        n -= 1;
    }
    (n, (radius / f64::from(1u32 << (n + 1))).max(0.5))
}

impl Backdrops {
    /// Drops the blurs of nodes that are gone, and those that no frame used
    /// for a while.
    pub fn retain(&mut self, mut keep: impl FnMut(&str) -> bool) {
        let now = Instant::now();
        self.entries
            .retain(|(_, id, _), entry| keep(id) && now - entry.used < UNUSED_FOR);
    }

    /// The element of the backdrop of a rect in a view, through a shape:
    /// the whole rect, or a piece of its plane.
    pub fn element(
        &mut self,
        view: &str,
        (node, piece): (&Node, usize),
        shape: Shape,
        (alpha, scale): (f32, f64),
    ) -> Option<BackdropElement> {
        let backdrop = node.props.backdrop?;
        let size = node.props.size?;
        let key = (view.to_owned(), node.id.clone(), piece);
        let entry = self.entries.entry(key.clone()).or_insert_with(|| Entry {
            blur: Arc::default(),
            id: Id::new(),
            commit: CommitCounter::default(),
            seen: None,
            used: Instant::now(),
        });
        entry.used = Instant::now();
        let bounds = shape.bounds;
        let (iterations, offset) =
            iterations(backdrop.blur * scale, [bounds.size.w, bounds.size.h]);
        let uniforms = BTreeMap::from([
            ("node".to_string(), Uniform::Vector(size.to_vec())),
            ("radius".to_string(), Uniform::Float(node.props.radius)),
            (
                "saturation".to_string(),
                Uniform::Float(backdrop.saturation),
            ),
        ]);
        let reads = hash_of(&(
            [bounds.loc.x, bounds.loc.y, bounds.size.w, bounds.size.h],
            iterations,
            offset.to_bits(),
        ));
        let look = look_of(&shape, &uniforms, alpha);
        Some(BackdropElement {
            key,
            id: entry.id.clone(),
            commit: entry.commit,
            blur: entry.blur.clone(),
            shape,
            alpha,
            uniforms,
            iterations,
            offset,
            reads,
            look,
        })
    }

    /// Settles the backdrops of a list of elements, front first, before a
    /// frame: a backdrop whose elements behind changed under its box since
    /// its last frame gets a new commit and blurs again in its next draw.
    /// Returns the passes of those blurs.
    pub fn settle<E: Element>(
        &mut self,
        elements: &mut [E],
        backdrop: impl Fn(&mut E) -> Option<&mut BackdropElement>,
        scale: Scale<f64>,
    ) -> u64 {
        let mut passes = 0;
        for index in 0..elements.len() {
            let (head, behind) = elements.split_at_mut(index + 1);
            let Some(element) = backdrop(&mut head[index]) else {
                continue;
            };
            let Some(entry) = self.entries.get_mut(&element.key) else {
                continue;
            };
            let bounds = element.shape.bounds;
            let under: Vec<&E> = behind
                .iter()
                .filter(|e| e.geometry(scale).overlaps(bounds))
                .collect();
            let blurs = entry.seen.as_ref().is_none_or(|seen| {
                seen.reads != element.reads
                    || seen.behind.len() != under.len()
                    || seen
                        .behind
                        .iter()
                        .zip(&under)
                        .any(|(old, e)| moved(old, *e, scale, bounds))
            });
            let changed = blurs
                || entry
                    .seen
                    .as_ref()
                    .is_some_and(|seen| seen.look != element.look);
            if changed {
                entry.commit.increment();
            }
            if blurs {
                if let Ok(mut blur) = entry.blur.lock() {
                    blur.stale = true;
                }
                passes += 2 * u64::from(element.iterations) - 1;
            }
            entry.seen = Some(Seen {
                reads: element.reads,
                look: element.look,
                behind: under
                    .iter()
                    .map(|e| Behind {
                        id: e.id().clone(),
                        commit: e.current_commit(),
                        geometry: e.geometry(scale),
                        alpha: e.alpha().to_bits(),
                    })
                    .collect(),
            });
            element.commit = entry.commit;
        }
        passes
    }
}

/// Whether an element behind a backdrop shows otherwise under its box
/// than it did.
fn moved<E: Element>(
    old: &Behind,
    element: &E,
    scale: Scale<f64>,
    bounds: Rectangle<i32, Physical>,
) -> bool {
    let geometry = element.geometry(scale);
    if old.id != *element.id() || old.geometry != geometry || old.alpha != element.alpha().to_bits()
    {
        return true;
    }
    if old.commit == element.current_commit() {
        return false;
    }
    element
        .damage_since(scale, Some(old.commit))
        .iter()
        .any(|d| Rectangle::new(d.loc + geometry.loc, d.size).overlaps(bounds))
}

fn hash_of(value: &impl std::hash::Hash) -> u64 {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn look_of(shape: &Shape, uniforms: &BTreeMap<String, Uniform>, alpha: f32) -> u64 {
    let vertices: Vec<[u32; 5]> = shape
        .vertices
        .iter()
        .map(|([x, y, w], [u, v])| [x, y, w, u, v].map(|n| n.to_bits()))
        .collect();
    hash_of(&(
        vertices,
        &shape.indices,
        format!("{uniforms:?}"),
        alpha.to_bits(),
    ))
}

/// The backdrop of a rect in one frame.
pub struct BackdropElement {
    key: Key,
    id: Id,
    commit: CommitCounter,
    blur: Arc<Mutex<Blur>>,
    shape: Shape,
    alpha: f32,
    /// The uniforms of the glass: the size and the radius of the rect,
    /// and the saturation.
    uniforms: BTreeMap<String, Uniform>,
    iterations: u32,
    offset: f64,
    /// The hashes of [`Seen`].
    reads: u64,
    look: u64,
}

impl Element for BackdropElement {
    fn id(&self) -> &Id {
        &self.id
    }

    fn current_commit(&self) -> CommitCounter {
        self.commit
    }

    fn src(&self) -> Rectangle<f64, BufferCoords> {
        Rectangle::from_size(
            self.shape
                .bounds
                .size
                .to_f64()
                .to_logical(1.0)
                .to_buffer(1.0, smithay::utils::Transform::Normal),
        )
    }

    fn geometry(&self, _scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.shape.bounds
    }

    fn damage_since(
        &self,
        scale: Scale<f64>,
        commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        if commit.is_some_and(|seen| seen == self.commit) {
            return DamageSet::default();
        }
        DamageSet::from_slice(&[Rectangle::from_size(self.geometry(scale).size)])
    }

    fn opaque_regions(&self, _scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        OpaqueRegions::default()
    }

    fn alpha(&self) -> f32 {
        self.alpha
    }

    fn kind(&self) -> Kind {
        Kind::Unspecified
    }
}

impl<R: MeshRenderer> RenderElement<R> for BackdropElement {
    fn draw(
        &self,
        frame: &mut R::Frame<'_, '_>,
        _src: Rectangle<f64, BufferCoords>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        _opaque_regions: &[Rectangle<i32, Physical>],
        _cache: Option<&smithay::utils::user_data::UserDataMap>,
    ) -> Result<(), R::Error> {
        let frame = R::gles_frame(frame);
        let Ok(mut blur) = self.blur.lock() else {
            return Ok(());
        };
        if blur.stale || blur.result.is_none() {
            let Some(window) =
                gl::backdrop::window_rect(frame, self.shape.bounds).map_err(R::from_gles)?
            else {
                return Ok(());
            };
            let grabbed =
                gl::backdrop::grab(frame, window, blur.grabbed.as_ref()).map_err(R::from_gles)?;
            blur.grabbed = Some(grabbed);
            if let Err(err) = self.blur_passes(frame, &mut blur, window) {
                tracing::warn!(%err, "a backdrop failed");
                return Ok(());
            }
            blur.stale = false;
        }
        let Some((texture, window)) = &blur.result else {
            return Ok(());
        };
        let mut windows = Vec::with_capacity(damage.len());
        for rect in damage {
            let at = Rectangle::new(rect.loc + dst.loc, rect.size);
            windows.extend(gl::backdrop::window_rect(frame, at).map_err(R::from_gles)?);
        }
        let (clip, uv) = mesh::clip_space(&self.shape.vertices, frame.projection());
        let program = gl::backdrop::programs(frame_renderer(frame).as_mut())
            .map_err(|_| R::from_gles(GlesError::ShaderCompileError))?
            .glass;
        let mut uniforms = self.uniforms.clone();
        uniforms.insert(
            "grab".into(),
            Uniform::Vector(window.map(f64::from).to_vec()),
        );
        let glass = Glass {
            program,
            texture,
            clip: &clip,
            uv: &uv,
            indices: &self.shape.indices,
            alpha: self.alpha,
            uniforms: &uniforms,
            damage: &windows,
        };
        gl::backdrop::glass(frame, &glass).map_err(R::from_gles)
    }
}

/// The renderer of a frame, which restores the target of the frame when
/// it goes.
fn frame_renderer<'a, 'f, 'b>(
    frame: &'a mut smithay::backend::renderer::gles::GlesFrame<'f, 'b>,
) -> impl AsMut<smithay::backend::renderer::gles::GlesRenderer> + 'a
where
    'f: 'a,
{
    use smithay::backend::renderer::FrameContext;
    frame.renderer()
}

impl BackdropElement {
    /// Halves the copy of the frame `iterations` times and doubles it back
    /// up to half its size: the glass samples that result bilinearly, which
    /// spares the costliest pass at the full size.
    fn blur_passes(
        &self,
        frame: &mut smithay::backend::renderer::gles::GlesFrame<'_, '_>,
        blur: &mut Blur,
        window: Window,
    ) -> Result<(), String> {
        let mut guard = frame_renderer(frame);
        let gles = guard.as_mut();
        let programs = gl::backdrop::programs(gles)?;
        let mut sizes = Vec::with_capacity(self.iterations as usize);
        let (mut w, mut h) = (window[2], window[3]);
        for _ in 0..self.iterations {
            (w, h) = ((w + 1) / 2, (h + 1) / 2);
            sizes.push((w, h));
        }
        let fourcc = smithay::backend::allocator::Fourcc::Abgr8888;
        let mut targets = Vec::with_capacity(sizes.len());
        for (index, (w, h)) in sizes.iter().enumerate() {
            let size = (*w, *h).into();
            let kept = blur
                .levels
                .get(index)
                .filter(|t| Texture::size(*t) == size)
                .cloned();
            let texture = match kept {
                Some(texture) => texture,
                None => gles
                    .create_buffer(fourcc, size)
                    .map_err(|e| e.to_string())?,
            };
            targets.push(texture);
        }
        let Some((grabbed, _)) = blur.grabbed.clone() else {
            return Err("no copy of the frame".into());
        };
        let offset = BTreeMap::from([("offset".to_string(), Uniform::Float(self.offset))]);
        let mut source = grabbed;
        for target in &mut targets {
            mesh::run_pass(gles, &programs.down, &source, target, &offset)?;
            source = target.clone();
        }
        for target in targets.iter_mut().rev().skip(1) {
            mesh::run_pass(gles, &programs.up, &source, target, &offset)?;
            source = target.clone();
        }
        blur.result = Some((source, window));
        blur.levels = targets;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::iterations;

    /// Two halvings at the least, also for a small box, so that the
    /// smallest blur still blurs.
    #[test]
    fn a_blur_halves_at_least_twice() {
        for (radius, size) in [(2.0, [6, 6]), (2.0, [400, 300]), (24.0, [3, 3])] {
            assert!(iterations(radius, size).0 >= 2, "{radius} on {size:?}");
        }
        assert_eq!(iterations(24.0, [400, 300]).0, 2);
        assert_eq!(iterations(25.0, [400, 300]).0, 3);
        assert_eq!(iterations(512.0, [4000, 3000]).0, 6);
    }
}
