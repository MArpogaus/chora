//! The shader passes of a layer texture.

use std::collections::BTreeMap;

use chora_scene::{Mesh, Node, NodeKind, Transform, Uniform};
use smithay::{
    backend::renderer::{
        Bind, Color32F, Frame, Renderer, Texture,
        gles::{GlesRenderer, GlesTexture},
    },
    utils::Rectangle,
};

use super::{Layers, NEAR, Passed, Placing, clip_space, float_size, project, triangles};
use crate::{gl, render::RenderStats};

/// One pass of a layer texture through a shader.
#[derive(Debug)]
pub enum Pass<'a> {
    /// The built-in shape: rounded corners and a border, in pixels of the
    /// texture, the border color premultiplied.
    Shape {
        radius: f64,
        border: Option<(f64, [f64; 4])>,
    },
    /// A shader file with its uniforms; `textures` tells whether the pass
    /// reads the node textures, as the shader of a shader node does.
    File {
        shader: &'a str,
        uniforms: &'a BTreeMap<String, Uniform>,
        textures: bool,
    },
}

/// The passes of a layer, in order: the shape of a rounded rect, the
/// shader of a shader node, the effects, and the clip, so that nothing
/// shows outside the clip. `scale` is the number of texture pixels per
/// logical pixel.
pub fn passes(node: &Node, scale: f64) -> Vec<Pass<'_>> {
    let props = &node.props;
    let mut out = Vec::new();
    if node.kind == NodeKind::Rect && (props.radius > 0.0 || props.border.is_some()) {
        let border = props
            .border
            .map(|b| (b.width * scale, crate::render::premultiplied(b.color)));
        out.push(Pass::Shape {
            radius: props.radius * scale,
            border,
        });
    }
    if node.kind == NodeKind::Shader
        && let Some(shader) = &props.shader
    {
        out.push(Pass::File {
            shader,
            uniforms: &props.uniforms,
            textures: true,
        });
    }
    out.extend(props.effects.iter().map(|e| Pass::File {
        shader: &e.shader,
        uniforms: &e.uniforms,
        textures: false,
    }));
    if let Some(clip) = props.clip {
        out.push(Pass::Shape {
            radius: clip.radius * scale,
            border: None,
        });
    }
    out
}

/// A node texture that a shader reads: the name of its sampler, the
/// texture and its version.
pub type Source = (String, GlesTexture, u64);

/// Runs the passes of a layer node over its texture and returns the
/// texture of the last pass with its version. `version` is that of the
/// texture; passes that read no clock and see the same texture, the same
/// node textures and the same uniforms as before keep their last result
/// and its version. A pass whose shader fails is left out; its error goes
/// to `errors`.
pub fn apply_passes(
    gles: &mut GlesRenderer,
    (layers, errors, stats): (&mut Layers, &mut Vec<String>, &mut RenderStats),
    node: &Node,
    (texture, version): (GlesTexture, u64),
    (scale, sources): (f64, &[Source]),
) -> (GlesTexture, u64) {
    let wanted = passes(node, scale);
    if wanted.is_empty() {
        return (texture, version);
    }
    let all = wanted.len();
    let compiled: Vec<(Pass<'_>, Compiled)> = wanted
        .into_iter()
        .filter_map(|pass| {
            let compiled = program_of(gles, &pass, (scale, sources), errors)?;
            Some((pass, compiled))
        })
        .collect();
    let size = Texture::size(&texture);
    let size_key = (size.w, size.h);
    let timed = compiled
        .iter()
        .any(|(_, (program, ..))| program.reads_time());
    let signature = signature((scale, version, sources), &compiled);
    let key = (layers.view.clone(), node.id.clone());
    let last = layers.passed.get(&key).map(|last| {
        (
            last.reusable && last.signature == signature && last.size == size_key,
            last.slot,
            last.version,
            last.runs,
        )
    });
    if let Some((true, slot, version, _)) = last
        && !timed
        && let Some(result) = layers.existing(&node.id, slot, size)
    {
        return (result, version);
    }
    let runs = last.map_or(0, |(.., runs)| runs) + 1;
    let errors_before = errors.len();
    let (texture, slot, completed) =
        run_passes((gles, layers, errors, stats), &node.id, texture, &compiled);
    let result = hash_of(&(signature, runs));
    let reusable = !timed && errors.len() == errors_before && completed == all;
    let passed = Passed {
        size: size_key,
        signature,
        slot,
        version: result,
        runs,
        reusable,
    };
    layers.passed.insert(key, passed);
    (texture, result)
}

/// Draws each pass over the result of the one before; returns the last
/// texture, its slot and the number of passes that ran.
fn run_passes(
    (gles, layers, errors, stats): (
        &mut GlesRenderer,
        &mut Layers,
        &mut Vec<String>,
        &mut RenderStats,
    ),
    node: &str,
    mut texture: GlesTexture,
    compiled: &[(Pass<'_>, Compiled)],
) -> (GlesTexture, usize, usize) {
    let mut slot = 0;
    let mut completed = 0;
    for (_, (program, uniforms, extra)) in compiled {
        let size = Texture::size(&texture);
        let next = if slot == 1 { 2 } else { 1 };
        let Ok(mut target) = layers.texture((gles, stats), node, next, size) else {
            continue;
        };
        let seconds = layers.clock;
        match draw_pass(
            gles,
            program,
            &texture,
            &mut target,
            &PassInputs {
                uniforms,
                textures: extra,
                seconds,
            },
        ) {
            Ok(()) => {
                stats.passes += 1;
                texture = target;
                slot = next;
                completed += 1;
            }
            Err(err) => errors.push(err),
        }
    }
    (texture, slot, completed)
}

fn hash_of(value: &impl std::hash::Hash) -> u64 {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// A hash of what the passes draw: the passes with their uniforms and
/// programs, the version of the texture and those of the node textures.
fn signature(
    (scale, version, sources): (f64, u64, &[Source]),
    compiled: &[(Pass<'_>, Compiled)],
) -> u64 {
    use std::{
        fmt::Write,
        hash::{Hash, Hasher},
    };

    struct Feed(std::collections::hash_map::DefaultHasher);
    impl Write for Feed {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            self.0.write(text.as_bytes());
            Ok(())
        }
    }
    let mut feed = Feed(std::collections::hash_map::DefaultHasher::new());
    (scale.to_bits(), version).hash(&mut feed.0);
    for (_, texture, version) in sources {
        (texture.tex_id(), version).hash(&mut feed.0);
    }
    for (pass, (program, uniforms, extra)) in compiled {
        let _ = write!(feed, "{pass:?}{uniforms:?}");
        program.id().hash(&mut feed.0);
        extra.hash(&mut feed.0);
    }
    feed.0.finish()
}

/// A program with its uniforms and its node textures.
type Compiled = (gl::Program, BTreeMap<String, Uniform>, Vec<(String, u32)>);

/// The program of a pass, its uniforms and its node textures.
fn program_of(
    gles: &mut GlesRenderer,
    pass: &Pass<'_>,
    (scale, sources): (f64, &[Source]),
    errors: &mut Vec<String>,
) -> Option<Compiled> {
    match pass {
        Pass::Shape { radius, border } => {
            let program = gl::shape_program(gles).map_err(|e| errors.push(e)).ok()?;
            let (width, color) = border.unwrap_or((0.0, [0.0; 4]));
            let uniforms = BTreeMap::from([
                ("radius".to_string(), Uniform::Float(*radius)),
                ("border_width".to_string(), Uniform::Float(width)),
                ("border_color".to_string(), Uniform::Vector(color.to_vec())),
            ]);
            Some((program, uniforms, Vec::new()))
        }
        Pass::File {
            shader,
            uniforms,
            textures: reads,
        } => {
            let (program, error) = gl::effect_program(gles, shader);
            errors.extend(error);
            let extra = if *reads {
                sources
                    .iter()
                    .map(|(n, t, _)| (n.clone(), t.tex_id()))
                    .collect()
            } else {
                Vec::new()
            };
            let mut uniforms = (*uniforms).clone();
            uniforms.insert("scale".into(), Uniform::Float(scale));
            Some((program?, uniforms, extra))
        }
    }
}

/// What a pass draws with besides its source: the uniforms and the node
/// textures of its effect, and the `time` of a recording.
#[derive(Clone, Copy)]
struct PassInputs<'a> {
    uniforms: &'a BTreeMap<String, Uniform>,
    textures: &'a [(String, u32)],
    seconds: Option<f32>,
}

/// Draws a texture through a built-in program with its uniforms into
/// another one.
pub(crate) fn run_pass(
    gles: &mut GlesRenderer,
    program: &gl::Program,
    source: &GlesTexture,
    target: &mut GlesTexture,
    uniforms: &BTreeMap<String, Uniform>,
) -> Result<(), String> {
    let with = PassInputs {
        uniforms,
        textures: &[],
        seconds: None,
    };
    draw_pass(gles, program, source, target, &with)
}

/// Draws a texture through the shader of a pass into another one.
fn draw_pass(
    gles: &mut GlesRenderer,
    program: &gl::Program,
    source: &GlesTexture,
    target: &mut GlesTexture,
    with: &PassInputs<'_>,
) -> Result<(), String> {
    let PassInputs {
        uniforms,
        textures,
        seconds,
    } = *with;
    let size = Texture::size(target);
    let mut framebuffer = gles.bind(target).map_err(|e| e.to_string())?;
    let mut frame = gles
        .render(
            &mut framebuffer,
            (size.w, size.h).into(),
            smithay::utils::Transform::Normal,
        )
        .map_err(|e| e.to_string())?;
    let all = Rectangle::from_size((size.w, size.h).into());
    frame
        .clear(Color32F::TRANSPARENT, &[all])
        .map_err(|e| e.to_string())?;
    let quad = Mesh::flat(1, 1, [f64::from(size.w), f64::from(size.h)]);
    let at = Placing {
        transform: &Transform::IDENTITY,
        scale: 1.0,
        near: NEAR,
    };
    let vertices = project(&quad, &at).ok_or("an empty pass")?.vertices;
    let (clip, uv) = clip_space(&vertices, frame.projection());
    let call = gl::Draw {
        clip: &clip,
        uv: &uv,
        indices: &triangles(1, 1),
        texture: source.tex_id(),
        alpha: 1.0,
        size: float_size(source),
        uniforms: Some(uniforms),
        textures,
        seconds,
    };
    gl::draw(&mut frame, program, &call).map_err(|e| e.to_string())?;
    frame.finish().map(drop).map_err(|e| e.to_string())
}
