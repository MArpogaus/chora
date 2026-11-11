//! The raw GL of backdrops: the copy of a part of the target of a frame
//! into a texture, the programs of the dual Kawase blur, and the draw of
//! the blurred texture through the shape of a rect.

use std::collections::BTreeMap;

use chora_scene::Uniform;
use smithay::{
    backend::renderer::{
        FrameContext,
        gles::{GlesError, GlesFrame, GlesRenderer, GlesTexture, ffi},
    },
    utils::{Buffer, Physical, Rectangle, Size},
};

use super::{Draw, Program, compile, draw};

/// A rectangle of the target of a frame in window coordinates: x and y
/// from the bottom left, width and height.
pub type Window = [i32; 4];

/// The downsample of the dual Kawase blur: the centre and four diagonal
/// taps `offset` texels of the source away, into a target of half the
/// size.
const DOWN: &str = "
precision mediump float;
uniform sampler2D tex;
uniform vec2 size;
uniform float offset;
varying vec2 v_uv;
void main() {
    vec2 d = offset / size;
    vec4 sum = texture2D(tex, v_uv) * 4.0;
    sum += texture2D(tex, v_uv - d);
    sum += texture2D(tex, v_uv + d);
    sum += texture2D(tex, v_uv + vec2(d.x, -d.y));
    sum += texture2D(tex, v_uv - vec2(d.x, -d.y));
    gl_FragColor = sum / 8.0;
}
";

/// The upsample of the dual Kawase blur: a ring of eight taps into a
/// target of twice the size.
const UP: &str = "
precision mediump float;
uniform sampler2D tex;
uniform vec2 size;
uniform float offset;
varying vec2 v_uv;
void main() {
    vec2 h = 0.5 * offset / size;
    vec4 sum = texture2D(tex, v_uv + vec2(-h.x * 2.0, 0.0));
    sum += texture2D(tex, v_uv + vec2(-h.x, h.y)) * 2.0;
    sum += texture2D(tex, v_uv + vec2(0.0, h.y * 2.0));
    sum += texture2D(tex, v_uv + vec2(h.x, h.y)) * 2.0;
    sum += texture2D(tex, v_uv + vec2(h.x * 2.0, 0.0));
    sum += texture2D(tex, v_uv + vec2(h.x, -h.y)) * 2.0;
    sum += texture2D(tex, v_uv + vec2(0.0, -h.y * 2.0));
    sum += texture2D(tex, v_uv + vec2(-h.x, -h.y)) * 2.0;
    gl_FragColor = sum / 12.0;
}
";

/// The glass: the blurred texture at the window position of each
/// fragment, saturated, inside the rounded box of the rect. It writes the
/// coverage as alpha, so that the blend replaces the frame behind instead
/// of adding to it. `node` is the size of the rect and `radius` its
/// corner radius, both in logical pixels; `grab` is the window rectangle
/// of the texture.
const GLASS: &str = "
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
uniform sampler2D tex;
uniform float alpha;
uniform vec4 grab;
uniform vec2 node;
uniform float radius;
uniform float saturation;
varying vec2 v_uv;
float box(vec2 p, vec2 half_size, float r) {
    vec2 q = abs(p) - half_size + vec2(r);
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}
void main() {
    vec4 color = texture2D(tex, (gl_FragCoord.xy - grab.xy) / grab.zw);
    float grey = dot(color.rgb, vec3(0.2126, 0.7152, 0.0722));
    color.rgb = clamp(mix(vec3(grey), color.rgb, saturation), 0.0, color.a);
    vec2 half_size = node * 0.5;
    float r = min(radius, min(half_size.x, half_size.y));
    float d = box(v_uv * node - half_size, half_size, r);
    float cover = clamp(0.5 - d, 0.0, 1.0) * alpha;
    gl_FragColor = vec4(color.rgb * cover, cover);
}
";

/// The built-in programs of backdrops of an EGL context.
#[derive(Clone, Copy)]
pub struct Programs {
    pub down: Program,
    pub up: Program,
    pub glass: Program,
}

/// The programs of backdrops, compiled on first use.
pub fn programs(gles: &mut GlesRenderer) -> Result<Programs, String> {
    if let Some(programs) = gles.egl_context().user_data().get::<Programs>() {
        return Ok(*programs);
    }
    let mut built = Vec::new();
    for source in [DOWN, UP, GLASS] {
        // SAFETY: compiles a fixed source on the current context.
        let program = gles
            .with_context(|gl| unsafe { compile(gl, source) })
            .map_err(|e| e.to_string())??;
        built.push(program);
    }
    let programs = Programs {
        down: built[0],
        up: built[1],
        glass: built[2],
    };
    gles.egl_context()
        .user_data()
        .insert_if_missing_threadsafe(|| programs);
    Ok(programs)
}

/// The window rectangle of a physical rectangle under the projection of a
/// frame, cut to its viewport; `None` when nothing is left.
pub fn window_rect(
    frame: &mut GlesFrame<'_, '_>,
    rect: Rectangle<i32, Physical>,
) -> Result<Option<Window>, GlesError> {
    let viewport = viewport(frame)?;
    let projection = *frame.projection();
    #[allow(clippy::cast_precision_loss)]
    let corners = [
        (rect.loc.x, rect.loc.y),
        (rect.loc.x + rect.size.w, rect.loc.y + rect.size.h),
    ]
    .map(|(across, down)| {
        let (across, down) = (across as f32, down as f32);
        let at = |row: usize| {
            projection[row] * across + projection[row + 3] * down + projection[row + 6]
        };
        let window = |row: usize| {
            viewport[row] as f32 + f32::midpoint(at(row), 1.0) * viewport[row + 2] as f32
        };
        [window(0), window(1)]
    });
    #[allow(clippy::cast_possible_truncation)]
    let edge = |row: usize| {
        let (low, high) = (viewport[row], viewport[row] + viewport[row + 2]);
        let (first, second) = (corners[0][row], corners[1][row]);
        let from = (first.min(second).round() as i32).clamp(low, high);
        let to = (first.max(second).round() as i32).clamp(low, high);
        (from, to - from)
    };
    let ((left, width), (bottom, height)) = (edge(0), edge(1));
    Ok((width > 0 && height > 0).then_some([left, bottom, width, height]))
}

fn viewport(frame: &mut GlesFrame<'_, '_>) -> Result<[i32; 4], GlesError> {
    // SAFETY: VIEWPORT writes four integers.
    frame.with_context(|gl| unsafe {
        let mut viewport = [0; 4];
        gl.GetIntegerv(ffi::VIEWPORT, viewport.as_mut_ptr());
        viewport
    })
}

/// A copy of a part of the target of a frame, and whether it has alpha.
pub type Grabbed = (GlesTexture, bool);

/// Copies a window rectangle of the target of a frame into a texture of
/// its size: into `kept` when it has that size and the alpha of the
/// target, else into a new texture. An output without alpha copies into
/// a texture without alpha, as GL wants.
pub fn grab(
    frame: &mut GlesFrame<'_, '_>,
    [x, y, w, h]: Window,
    kept: Option<&Grabbed>,
) -> Result<Grabbed, GlesError> {
    let size: Size<i32, Buffer> = (w, h).into();
    // SAFETY: ALPHA_BITS writes one integer.
    let alpha = frame.with_context(|gl| unsafe {
        let mut bits = 0;
        gl.GetIntegerv(ffi::ALPHA_BITS, &raw mut bits);
        while gl.GetError() != ffi::NO_ERROR {}
        bits > 0
    })?;
    let reuse = kept.filter(|(texture, had)| {
        smithay::backend::renderer::Texture::size(texture) == size && *had == alpha
    });
    if let Some((texture, _)) = reuse {
        let id = texture.tex_id();
        // SAFETY: copies into a texture of the same size and format.
        frame.with_context(|gl| unsafe {
            gl.BindTexture(ffi::TEXTURE_2D, id);
            gl.CopyTexSubImage2D(ffi::TEXTURE_2D, 0, 0, 0, x, y, w, h);
            gl.BindTexture(ffi::TEXTURE_2D, 0);
        })?;
        return Ok((texture.clone(), alpha));
    }
    let format = if alpha { ffi::RGBA } else { ffi::RGB };
    // SAFETY: makes and fills a new texture; the renderer takes it over.
    let id = frame.with_context(|gl| unsafe {
        let mut id = 0;
        gl.GenTextures(1, &raw mut id);
        gl.BindTexture(ffi::TEXTURE_2D, id);
        gl.CopyTexImage2D(ffi::TEXTURE_2D, 0, format, x, y, w, h, 0);
        gl.BindTexture(ffi::TEXTURE_2D, 0);
        id
    })?;
    let guard = frame.renderer();
    let gles: &GlesRenderer = guard.as_ref();
    // SAFETY: `id` is a texture of this context that nothing else owns.
    let texture = unsafe { GlesTexture::from_raw(gles, Some(format), !alpha, id, size) };
    Ok((texture, alpha))
}

/// What the glass draws: the blurred texture, with the window rectangle
/// that it came from in the uniform `grab`, the shape of the rect, and the
/// rectangles of the frame that it may change.
pub struct Glass<'a> {
    pub program: Program,
    pub texture: &'a GlesTexture,
    pub clip: &'a [f32],
    pub uv: &'a [f32],
    pub indices: &'a [u16],
    pub alpha: f32,
    pub uniforms: &'a BTreeMap<String, Uniform>,
    pub damage: &'a [Window],
}

/// Draws the glass into each damaged window rectangle, so that it
/// replaces the frame behind its shape, and restores the scissor and the
/// blend of the renderer.
pub fn glass(frame: &mut GlesFrame<'_, '_>, glass: &Glass<'_>) -> Result<(), GlesError> {
    let viewport = viewport(frame)?;
    let size = smithay::backend::renderer::Texture::size(glass.texture);
    #[allow(clippy::cast_precision_loss)]
    let call = Draw {
        clip: glass.clip,
        uv: glass.uv,
        indices: glass.indices,
        texture: glass.texture.tex_id(),
        alpha: glass.alpha,
        size: [size.w as f32, size.h as f32],
        uniforms: Some(glass.uniforms),
        textures: &[],
        seconds: None,
    };
    // SAFETY: plain state changes on the current context.
    frame.with_context(|gl| unsafe {
        gl.BlendFuncSeparate(ffi::ONE, ffi::ONE_MINUS_SRC_ALPHA, ffi::ZERO, ffi::ONE);
    })?;
    let mut result = Ok(());
    for [x, y, w, h] in glass.damage {
        // SAFETY: as above.
        frame.with_context(|gl| unsafe { gl.Scissor(*x, *y, *w, *h) })?;
        result = result.and(draw(frame, &glass.program, &call));
    }
    let [x, y, w, h] = viewport;
    // SAFETY: as above.
    frame.with_context(|gl| unsafe {
        gl.Scissor(x, y, w, h);
        gl.BlendFunc(ffi::ONE, ffi::ONE_MINUS_SRC_ALPHA);
    })?;
    result
}
