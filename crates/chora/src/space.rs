//! The planes of 3D contexts (D75).
//!
//! A group with a size and a `perspective` d is a 3D context: the eye sits
//! at distance d in front of its centre and looks along +z, and a point
//! shows with w = 1 + z / d. The scene composes that projection into the
//! transforms of the descendants. Here the planes of a context get their
//! drawing order: each plane is clipped at w = 1/64, and the crate
//! `plane-split` splits planes that cross and sorts the pieces from back to
//! front. Planes on one geometric plane keep the order of the scene. The
//! hit test walks the same pieces from the front, so that input reaches
//! the nearest plane that shows at a point.

use std::{
    cell::Cell,
    collections::HashMap,
    sync::{Arc, Mutex, PoisonError},
};

use chora_scene::{Placed, Transform};
use euclid::default::{Point3D, Vector3D};
use plane_split::{BspSplitter, Clipper, Plane, Polygon};

/// The least w of a point of a context that shows.
pub const NEAR_W: f64 = 1.0 / 64.0;
/// The most planes of a context that the splitter sorts; beyond them the
/// pieces sort by the depth of their centres. Planes that cross split each
/// other, so the pieces grow fast with the planes.
const MAX_SPLIT: usize = 64;
/// The most pieces per plane that a split may give; beyond them the pieces
/// sort by the depth of their centres.
const PIECES_PER_PLANE: usize = 4;
/// The log told that a context has too many planes to split.
static TOLD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What the order needs of a plane of a context.
pub struct Flat {
    /// The size of the plane in its own coordinates.
    pub size: [f64; 2],
    /// Whether the plane draws as one unit, as a mesh does: it never
    /// splits, and draws whole at the place of its back piece.
    pub unit: bool,
}

/// A piece of a plane of a context, in drawing order.
pub struct Piece {
    /// The index of the plane in the list of the context.
    pub plane: usize,
    /// `None` for a whole plane, which draws and hits as it would outside
    /// the context.
    pub part: Option<Part>,
}

/// The part of a plane that a split or the near clip leaves.
pub struct Part {
    /// The corners in the coordinates of the plane.
    pub corners: [[f64; 2]; 4],
    /// The corners as the list of the context shows them.
    pub shown: [[f64; 2]; 4],
}

/// An order that [`Orders`] keeps, with the fingerprint of what it was made
/// from.
type Kept = (Vec<u64>, Arc<Vec<Piece>>);

/// The orders of the contexts of a scene, kept between frames: the draw
/// and the hit test of a context share one order for as long as its planes
/// keep their sizes, their transforms and their order in the list.
#[derive(Default)]
pub struct Orders {
    kept: Mutex<HashMap<String, Kept>>,
}

impl Orders {
    /// The order of a context as [`order`] gives it, from the last call
    /// when the context looks the same. `computed` counts the calls that
    /// ran the splitter.
    pub fn get(
        &self,
        computed: &Cell<u64>,
        context: &Placed<'_>,
        list: &[Placed<'_>],
        flat: impl Fn(&Placed<'_>) -> Option<Flat>,
    ) -> Arc<Vec<Piece>> {
        let flats: Vec<Option<Flat>> = list.iter().map(flat).collect();
        let key = fingerprint(context, list, &flats);
        let mut kept = self.kept.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((same, pieces)) = kept.get(&context.node.id)
            && *same == key
        {
            return Arc::clone(pieces);
        }
        computed.set(computed.get() + 1);
        let pieces = Arc::new(order_flats(context, list, flats));
        kept.insert(context.node.id.clone(), (key, Arc::clone(&pieces)));
        pieces
    }

    /// Forgets the contexts that `keep` refuses.
    pub fn retain(&self, mut keep: impl FnMut(&str) -> bool) {
        let mut kept = self.kept.lock().unwrap_or_else(PoisonError::into_inner);
        kept.retain(|id, _| keep(id));
    }
}

/// What [`order`] reads of a context: its place and eye, and of each entry
/// its content size, whether it is a unit, and its transform.
fn fingerprint(context: &Placed<'_>, list: &[Placed<'_>], flats: &[Option<Flat>]) -> Vec<u64> {
    let props = &context.node.props;
    let mut key = vec![props.perspective.map_or(0, f64::to_bits)];
    let [w, h] = props.size.unwrap_or_default();
    key.extend([w.to_bits(), h.to_bits()]);
    key.extend(context.transform.matrix().iter().map(|v| v.to_bits()));
    for (placed, flat) in list.iter().zip(flats) {
        match flat {
            Some(flat) => {
                key.extend([
                    flat.size[0].to_bits(),
                    flat.size[1].to_bits(),
                    flat.unit.into(),
                ]);
                key.extend(placed.transform.matrix().iter().map(|v| v.to_bits()));
            }
            None => key.push(u64::MAX),
        }
    }
    key
}

/// The planes of a context in pieces from back to front. `context` is the
/// context as its list places it, `list` its [`chora_scene::Scene::context_list`],
/// and `flat` tells what each entry of the list needs; an entry without it
/// draws nothing. A plane that is no polygon in the space of the eye, as
/// under a matrix of its own with a w part, draws whole behind the others.
#[cfg(test)]
pub fn order(
    context: &Placed<'_>,
    list: &[Placed<'_>],
    flat: impl Fn(&Placed<'_>) -> Option<Flat>,
) -> Vec<Piece> {
    order_flats(context, list, list.iter().map(flat).collect())
}

fn order_flats(context: &Placed<'_>, list: &[Placed<'_>], flats: Vec<Option<Flat>>) -> Vec<Piece> {
    let whole = |plane| Piece { plane, part: None };
    let Some(Eye {
        frame,
        unproject,
        d,
    }) = Eye::of(context)
    else {
        return (0..list.len())
            .filter(|i| flats[*i].is_some())
            .map(whole)
            .collect();
    };
    let near = Plane::from_unnormalized(Vector3D::new(0.0, 0.0, 1.0 / d), 1.0 - NEAR_W);
    let Ok(Some(near)) = near else {
        return Vec::new();
    };
    let mut near_clip = Clipper::new();
    near_clip.add(near);
    let mut out = Vec::new();
    let mut planes = Vec::with_capacity(list.len());
    let mut projected = Vec::new();
    for (index, (placed, flat)) in list.iter().zip(flats).enumerate() {
        let Some(flat) = flat else {
            planes.push(None);
            continue;
        };
        let Some(polygon) = eye_polygon(&unproject.then(&placed.transform), flat.size, index)
        else {
            out.push(whole(index));
            planes.push(None);
            continue;
        };
        let clipped = polygon.points.iter().any(|p| 1.0 + p.z / d < NEAR_W);
        let pieces = if clipped {
            near_clip.clip(polygon.clone()).to_vec()
        } else {
            vec![polygon.clone()]
        };
        projected.extend(pieces.iter().filter_map(|p| project(&frame, p)));
        planes.push(Some((flat, polygon, clipped, placed.transform)));
    }
    let sorted = sort(projected, planes.iter().flatten().count());
    out.extend(pieces(&sorted, &planes, &unproject));
    out
}

/// A plane of a context that the order sorts: what it needs, its polygon
/// in the space of the eye, whether the near plane clips it, and its
/// transform in the list of the context.
type Sorted = Option<(Flat, Polygon<usize>, bool, Transform)>;

/// The pieces of sorted polygons: a plane that drew as one unit draws
/// whole at the place of its back piece, a plane that the sort split or
/// the near plane clipped draws in parts, and any other plane whole.
fn pieces(sorted: &[Polygon<usize>], planes: &[Sorted], unproject: &Transform) -> Vec<Piece> {
    let mut count = vec![0_usize; planes.len()];
    for piece in sorted {
        count[piece.anchor] += 1;
    }
    let mut out = Vec::with_capacity(sorted.len());
    for piece in sorted {
        let index = piece.anchor;
        let Some((flat, polygon, clipped, transform)) = &planes[index] else {
            continue;
        };
        if flat.unit {
            if std::mem::take(&mut count[index]) > 0 {
                out.push(Piece {
                    plane: index,
                    part: None,
                });
            }
            continue;
        }
        let part = (*clipped || count[index] > 1)
            .then(|| part((unproject, transform), polygon, flat.size, piece));
        out.push(Piece { plane: index, part });
    }
    out
}

/// The space of the eye of a placed context: the transform from it to the
/// coordinates of the list of the context, its inverse, and the distance
/// of the eye. In the space of the eye, the eye sits at distance d in front
/// of the centre of the context, before its projection.
struct Eye {
    frame: Transform,
    unproject: Transform,
    d: f64,
}

impl Eye {
    fn of(context: &Placed<'_>) -> Option<Self> {
        let frame = context.transform.then(&context.node.projection()?);
        Some(Self {
            unproject: frame.inverse()?,
            frame,
            d: context.node.props.perspective?,
        })
    }
}

/// The depth and the facing of a plane of a context, for its shaders: the
/// distance from the eye to the centre of the plane over d, which is 1 at
/// 1:1, and the cosine of the angle between the front of the plane and the
/// way to the eye, negative from behind.
pub fn view(context: &Placed<'_>, placed: &Placed<'_>, [w, h]: [f64; 2]) -> Option<(f64, f64)> {
    let Eye { unproject, d, .. } = Eye::of(context)?;
    let [cw, ch] = context.node.props.size?;
    let eye_space = unproject.then(&placed.transform);
    let at = |u: f64, v: f64| point(&eye_space, [u, v]);
    let (origin, right, down) = (at(0.0, 0.0)?, at(w, 0.0)?, at(0.0, h)?);
    let centre = at(w / 2.0, h / 2.0)?;
    let to_eye = Point3D::new(cw / 2.0, ch / 2.0, -d) - centre;
    let front = -(right - origin).cross(down - origin);
    let facing = front.normalize().dot(to_eye.normalize());
    let depth = to_eye.length() / d;
    (depth.is_finite() && facing.is_finite()).then_some((depth, facing))
}

/// Whether a point lies in a convex polygon of either winding.
pub fn contains(corners: &[[f64; 2]; 4], [x, y]: [f64; 2]) -> bool {
    let finite = [x, y]
        .iter()
        .chain(corners.iter().flatten())
        .all(|v| v.is_finite());
    if !finite {
        return false;
    }
    let mut sides = [false; 2];
    for (index, [ax, ay]) in corners.iter().enumerate() {
        let [bx, by] = corners[(index + 1) % corners.len()];
        let cross = (bx - ax) * (y - ay) - (by - ay) * (x - ax);
        if cross > 0.0 {
            sides[0] = true;
        } else if cross < 0.0 {
            sides[1] = true;
        }
    }
    !(sides[0] && sides[1])
}

/// The point of the plane z = 0 at `(u, v)` after a transform, if it lies
/// in front of the projection of the transform.
fn point(transform: &Transform, [u, v]: [f64; 2]) -> Option<Point3D<f64>> {
    let m = transform.matrix();
    let at = |row: usize| m[row] * u + m[4 + row] * v + m[12 + row];
    let w = at(3);
    (w > 0.0).then(|| Point3D::new(at(0) / w, at(1) / w, at(2) / w))
}

/// A plane in the coordinates of the eye: before the projection of the
/// context. `None` when it is edge on or reaches behind its own
/// projection.
fn eye_polygon(eye_space: &Transform, [w, h]: [f64; 2], index: usize) -> Option<Polygon<usize>> {
    let corners = [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]];
    let points = [
        point(eye_space, corners[0])?,
        point(eye_space, corners[1])?,
        point(eye_space, corners[2])?,
        point(eye_space, corners[3])?,
    ];
    Polygon::from_points(points, index)
}

/// A polygon of the eye in the coordinates of the list of the context,
/// with z over w as its depth, so that the rays from the eye run along z.
fn project(projection: &Transform, polygon: &Polygon<usize>) -> Option<Polygon<usize>> {
    let m = projection.matrix();
    let points = polygon.points.map(|p| {
        let w = m[3] * p.x + m[7] * p.y + m[11] * p.z + m[15];
        Point3D::new(
            (m[0] * p.x + m[4] * p.y + m[8] * p.z + m[12]) / w,
            (m[1] * p.x + m[5] * p.y + m[9] * p.z + m[13]) / w,
            (m[2] * p.x + m[6] * p.y + m[10] * p.z + m[14]) / w,
        )
    });
    Polygon::from_points(points, polygon.anchor)
}

/// The projected pieces of `planes` planes from back to front. The
/// splitter takes the pieces one by one and gives up when they split into
/// more than `PIECES_PER_PLANE` per plane; then, and for more than
/// `MAX_SPLIT` planes, the pieces sort by the depth of their centres.
fn sort(pieces: Vec<Polygon<usize>>, planes: usize) -> Vec<Polygon<usize>> {
    if planes > MAX_SPLIT {
        return by_centres(pieces);
    }
    let view = Vector3D::new(0.0, 0.0, -1.0);
    let most = (PIECES_PER_PLANE * planes).max(pieces.len());
    let mut splitter = BspSplitter::new();
    for piece in &pieces {
        splitter.add(piece.clone());
        if splitter.sort(view).len() > most {
            return by_centres(pieces);
        }
    }
    splitter.sort(view).to_vec()
}

/// Pieces from back to front by the depth of their centres, without a
/// split, for a context whose planes are too many or split too much.
fn by_centres(mut pieces: Vec<Polygon<usize>>) -> Vec<Polygon<usize>> {
    if !TOLD.swap(true, std::sync::atomic::Ordering::Relaxed) {
        tracing::warn!(
            "a 3D context has more than {MAX_SPLIT} planes or splits too much; \
             its planes sort by their centres"
        );
    }
    let depth = |p: &Polygon<usize>| p.points.iter().map(|q| q.z).sum::<f64>();
    pieces.sort_by(|a, b| depth(b).total_cmp(&depth(a)));
    pieces
}

/// The part of a plane that a projected piece covers. Its corners come
/// from the inverse of the transform of the plane, as the hit test maps a
/// point, so the part draws where it takes input; the space of the eye
/// gives them where that inverse has none.
fn part(
    (unproject, transform): (&Transform, &Transform),
    polygon: &Polygon<usize>,
    [w, h]: [f64; 2],
    piece: &Polygon<usize>,
) -> Part {
    let m = unproject.matrix();
    let corners = piece.points.map(|p| {
        if let Some(at) = crate::render::plane_point(transform, (p.x, p.y).into()) {
            return [at.x, at.y];
        }
        let at = |row: usize| m[row] * p.x + m[4 + row] * p.y + m[8 + row] * p.z + m[12 + row];
        let eye = Point3D::new(at(0), at(1), at(2)) / at(3);
        let uv = polygon.untransform_point(eye);
        [uv.x * w, uv.y * h]
    });
    Part {
        corners,
        shown: piece.points.map(|p| [p.x, p.y]),
    }
}

#[cfg(test)]
mod tests {
    use super::contains;

    #[test]
    fn a_point_is_in_a_polygon_of_either_winding_and_with_a_repeated_corner() {
        let square = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        assert!(contains(&square, [5.0, 5.0]));
        assert!(!contains(&square, [15.0, 5.0]));
        let mut mirrored = square;
        mirrored.reverse();
        assert!(contains(&mirrored, [5.0, 5.0]));
        let triangle = [[0.0, 0.0], [10.0, 0.0], [10.0, 0.0], [0.0, 10.0]];
        assert!(contains(&triangle, [2.0, 2.0]));
        assert!(!contains(&triangle, [8.0, 8.0]));
    }

    #[test]
    fn a_point_that_is_not_finite_is_in_no_polygon() {
        let square = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        assert!(!contains(&square, [f64::NAN, 5.0]));
        assert!(!contains(&square, [5.0, f64::INFINITY]));
        let collapsed = [[f64::NAN; 2]; 4];
        assert!(!contains(&collapsed, [5.0, 5.0]));
    }
}
