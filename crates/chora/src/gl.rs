//! The raw GL calls of layers: the program of meshes, the programs of
//! effects, and drawing a texture through triangles with one of them.
//!
//! GL calls are `unsafe`; this module is the one place with them. The other
//! `unsafe` code writes into shm buffers, in `shm`. The state that changes
//! here goes back to what Smithay sets before its own draws: its draw calls
//! bind their program, texture and arrays each time.
#![allow(unsafe_code)]

use std::{
    collections::{BTreeMap, HashMap},
    ffi::CString,
    sync::{Mutex, OnceLock},
    time::{Instant, SystemTime},
};

use chora_scene::Uniform;
use smithay::backend::renderer::gles::{Capability, GlesError, GlesFrame, GlesRenderer, ffi};

/// The largest shader file.
const MAX_SOURCE: u64 = 64 << 10;

/// A linked program with the locations of its inputs.
#[derive(Clone, Copy, Debug)]
pub struct Program {
    id: ffi::types::GLuint,
    position: ffi::types::GLint,
    uv: ffi::types::GLint,
    tex: ffi::types::GLint,
    alpha: ffi::types::GLint,
    size: ffi::types::GLint,
    time: ffi::types::GLint,
}

const VERTEX: &str = "
attribute vec4 position;
attribute vec2 uv;
varying vec2 v_uv;
void main() {
    gl_Position = position;
    v_uv = uv;
}
";

/// The fragment shader of a plain mesh.
const PLAIN: &str = "
precision mediump float;
uniform sampler2D tex;
uniform float alpha;
varying vec2 v_uv;
void main() {
    gl_FragColor = texture2D(tex, v_uv) * alpha;
}
";

/// The built-in pass of rounded corners, borders and clips: it keeps the
/// texture inside a rounded box of the size of the texture, and draws a
/// border along the inner edge of the box. `radius` and `border_width` are
/// in pixels of the texture, `border_color` is premultiplied.
const SHAPE: &str = "
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
uniform sampler2D tex;
uniform float alpha;
uniform vec2 size;
uniform float radius;
uniform float border_width;
uniform vec4 border_color;
varying vec2 v_uv;
float box(vec2 p, vec2 half_size, float r) {
    vec2 q = abs(p) - half_size + vec2(r);
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}
void main() {
    vec2 half_size = size * 0.5;
    float r = min(radius, min(half_size.x, half_size.y));
    float d = box(v_uv * size - half_size, half_size, r);
    vec4 color = texture2D(tex, v_uv);
    if (border_width > 0.0) {
        color = mix(border_color, color, clamp(0.5 - (d + border_width), 0.0, 1.0));
    }
    gl_FragColor = color * clamp(0.5 - d, 0.0, 1.0) * alpha;
}
";

/// What a draw call draws: triangles with clip positions and texture
/// coordinates, a texture, and the inputs of the program.
pub struct Draw<'a> {
    /// Four numbers per vertex.
    pub clip: &'a [f32],
    /// Two numbers per vertex.
    pub uv: &'a [f32],
    pub indices: &'a [u16],
    pub texture: ffi::types::GLuint,
    pub alpha: f32,
    /// The size of the texture in pixels.
    pub size: [f32; 2],
    /// The uniforms of an effect.
    pub uniforms: Option<&'a BTreeMap<String, Uniform>>,
    /// More textures by the names of their samplers, for the node
    /// textures of a shader node.
    pub textures: &'a [(String, ffi::types::GLuint)],
}

/// Draws triangles with a program into the target of a frame.
pub fn draw(
    frame: &mut GlesFrame<'_, '_>,
    program: &Program,
    call: &Draw<'_>,
) -> Result<(), GlesError> {
    // Smithay's instanced draws leave divisors on attribute indices.
    let instancing = frame.capabilities().contains(&Capability::Instancing);
    let count = i32::try_from(call.indices.len()).unwrap_or(0);
    let seconds = START.get_or_init(Instant::now).elapsed().as_secs_f32();
    frame.with_context(|gl| unsafe {
        // SAFETY: the arrays outlive the draw call; see the module notes.
        gl.UseProgram(program.id);
        gl.ActiveTexture(ffi::TEXTURE0);
        gl.BindTexture(ffi::TEXTURE_2D, call.texture);
        for parameter in [ffi::TEXTURE_MIN_FILTER, ffi::TEXTURE_MAG_FILTER] {
            gl.TexParameteri(ffi::TEXTURE_2D, parameter, ffi::LINEAR.cast_signed());
        }
        // Samples past an edge take the edge, not the opposite side.
        for parameter in [ffi::TEXTURE_WRAP_S, ffi::TEXTURE_WRAP_T] {
            gl.TexParameteri(ffi::TEXTURE_2D, parameter, ffi::CLAMP_TO_EDGE.cast_signed());
        }
        gl.Uniform1i(program.tex, 0);
        gl.Uniform1f(program.alpha, call.alpha);
        gl.Uniform2f(program.size, call.size[0], call.size[1]);
        gl.Uniform1f(program.time, seconds);
        for (name, value) in call.uniforms.into_iter().flatten() {
            set_uniform(gl, program.id, name, value);
        }
        bind_textures(gl, program.id, call.textures);
        gl.BindBuffer(ffi::ARRAY_BUFFER, 0);
        let attributes = [(program.position, 4, call.clip), (program.uv, 2, call.uv)];
        for (attribute, size, data) in attributes {
            let Ok(index) = u32::try_from(attribute) else {
                continue;
            };
            gl.EnableVertexAttribArray(index);
            if instancing {
                gl.VertexAttribDivisor(index, 0);
            }
            gl.VertexAttribPointer(index, size, ffi::FLOAT, ffi::FALSE, 0, data.as_ptr().cast());
        }
        gl.DrawElements(
            ffi::TRIANGLES,
            count,
            ffi::UNSIGNED_SHORT,
            call.indices.as_ptr().cast(),
        );
        for attribute in [program.position, program.uv] {
            if let Ok(index) = u32::try_from(attribute) {
                gl.DisableVertexAttribArray(index);
            }
        }
        for unit in 1..=u32::try_from(call.textures.len()).unwrap_or(0) {
            gl.ActiveTexture(ffi::TEXTURE0 + unit);
            gl.BindTexture(ffi::TEXTURE_2D, 0);
        }
        gl.ActiveTexture(ffi::TEXTURE0);
        gl.BindTexture(ffi::TEXTURE_2D, 0);
    })
}

/// Binds more textures to the units after 0 and points their samplers at
/// them; a name that the shader lacks does nothing.
unsafe fn bind_textures(
    gl: &ffi::Gles2,
    program: ffi::types::GLuint,
    textures: &[(String, ffi::types::GLuint)],
) {
    for ((name, texture), unit) in textures.iter().zip(1u32..) {
        let Ok(cname) = CString::new(name.as_str()) else {
            continue;
        };
        // SAFETY: plain GL calls on the current context.
        unsafe {
            gl.ActiveTexture(ffi::TEXTURE0 + unit);
            gl.BindTexture(ffi::TEXTURE_2D, *texture);
            for parameter in [ffi::TEXTURE_MIN_FILTER, ffi::TEXTURE_MAG_FILTER] {
                gl.TexParameteri(ffi::TEXTURE_2D, parameter, ffi::LINEAR.cast_signed());
            }
            for parameter in [ffi::TEXTURE_WRAP_S, ffi::TEXTURE_WRAP_T] {
                gl.TexParameteri(ffi::TEXTURE_2D, parameter, ffi::CLAMP_TO_EDGE.cast_signed());
            }
            let location = gl.GetUniformLocation(program, cname.as_ptr());
            gl.Uniform1i(location, unit.cast_signed());
        }
    }
    // SAFETY: as above.
    unsafe { gl.ActiveTexture(ffi::TEXTURE0) };
}

/// The time of the `time` uniform starts with the first draw.
static START: OnceLock<Instant> = OnceLock::new();

/// Sets a uniform of an effect by name; a name that the shader lacks does
/// nothing.
unsafe fn set_uniform(gl: &ffi::Gles2, program: ffi::types::GLuint, name: &str, value: &Uniform) {
    let Ok(cname) = CString::new(name) else {
        return;
    };
    #[allow(clippy::cast_possible_truncation)] // GL takes 32-bit floats
    let values: Vec<f32> = value.values().iter().map(|v| *v as f32).collect();
    // SAFETY: plain GL calls on the current context.
    unsafe {
        let location = gl.GetUniformLocation(program, cname.as_ptr());
        match values[..] {
            [x] => gl.Uniform1f(location, x),
            [x, y] => gl.Uniform2f(location, x, y),
            [x, y, z] => gl.Uniform3f(location, x, y, z),
            [x, y, z, w] => gl.Uniform4f(location, x, y, z, w),
            _ => {}
        }
    }
}

/// The program of plain meshes of the EGL context of a frame, compiled on
/// first use.
pub fn plain_program(frame: &mut GlesFrame<'_, '_>) -> Result<Program, GlesError> {
    if let Some(program) = frame.egl_context().user_data().get::<Program>() {
        return Ok(*program);
    }
    let program = frame
        .with_context(|gl| unsafe { compile(gl, PLAIN) })?
        .map_err(|_| GlesError::ShaderCompileError)?;
    frame
        .egl_context()
        .user_data()
        .insert_if_missing_threadsafe(|| program);
    Ok(program)
}

/// The built-in program of shapes of an EGL context.
#[derive(Clone, Copy)]
struct ShapeProgram(Program);

/// The built-in program of rounded corners, borders and clips, compiled on
/// first use.
pub fn shape_program(gles: &mut GlesRenderer) -> Result<Program, String> {
    if let Some(ShapeProgram(program)) = gles.egl_context().user_data().get::<ShapeProgram>() {
        return Ok(*program);
    }
    let program = gles
        .with_context(|gl| unsafe { compile(gl, SHAPE) })
        .map_err(|e| e.to_string())??;
    gles.egl_context()
        .user_data()
        .insert_if_missing_threadsafe(|| ShapeProgram(program));
    Ok(program)
}

/// The programs of effects of an EGL context by path, with the time of
/// the file that they came from.
type EffectPrograms = Mutex<HashMap<String, (Option<SystemTime>, Option<Program>)>>;

/// The program of an effect, compiled again when its file changed. A file
/// that fails keeps the last program that worked, and returns the error
/// once as well.
pub fn effect_program(gles: &mut GlesRenderer, path: &str) -> (Option<Program>, Option<String>) {
    let modified = std::fs::metadata(path).and_then(|m| m.modified()).ok();
    let programs = gles
        .egl_context()
        .user_data()
        .get_or_insert_threadsafe(EffectPrograms::default);
    let cached = programs.lock().ok().and_then(|p| p.get(path).copied());
    if let Some((time, program)) = cached
        && time == modified
    {
        return (program, None);
    }
    let compiled = read_source(path).and_then(|source| {
        gles.with_context(|gl| unsafe { compile(gl, &source) })
            .map_err(|e| e.to_string())?
    });
    let old = cached.and_then(|(_, p)| p);
    let (program, error) = match compiled {
        Ok(program) => {
            if let Some(old) = old {
                let _ = gles.with_context(|gl| unsafe { gl.DeleteProgram(old.id) });
            }
            (Some(program), None)
        }
        Err(error) => (old, Some(format!("{path}: {error}"))),
    };
    let programs = gles
        .egl_context()
        .user_data()
        .get_or_insert_threadsafe(EffectPrograms::default);
    if let Ok(mut programs) = programs.lock() {
        programs.insert(path.into(), (modified, program));
    }
    (program, error)
}

/// Compiles the shader of an effect and drops it again, to check it.
pub fn check_effect(gles: &mut GlesRenderer, path: &str) -> Result<(), String> {
    let source = read_source(path)?;
    let program = gles
        .with_context(|gl| unsafe { compile(gl, &source) })
        .map_err(|e| e.to_string())?
        .map_err(|log| format!("{path}: {log}"))?;
    gles.with_context(|gl| unsafe { gl.DeleteProgram(program.id) })
        .map_err(|e| e.to_string())
}

/// Reads a shader file. A file of `/proc` claims 0 bytes, so the read
/// stops after the limit itself.
fn read_source(path: &str) -> Result<String, String> {
    let (file, _) = crate::images::open_file(path, MAX_SOURCE)?;
    let mut source = String::new();
    std::io::Read::read_to_string(&mut std::io::Read::take(file, MAX_SOURCE + 1), &mut source)
        .map_err(|e| format!("{path}: {e}"))?;
    if source.len() as u64 > MAX_SOURCE {
        return Err(format!("{path}: larger than {MAX_SOURCE} bytes"));
    }
    Ok(with_derivatives(source))
}

/// Enables `GL_OES_standard_derivatives` for a shader that uses `fwidth`,
/// `dFdx` or `dFdy` and does not enable it itself. On a driver
/// without the extension, `enable` gives a warning, not an error. The line
/// goes after the `#version` line, which must come first, also after
/// comments; GLSL ES 3.00 has the functions without the extension.
fn with_derivatives(source: String) -> String {
    let uses = ["fwidth", "dFdx", "dFdy"]
        .iter()
        .any(|f| source.contains(f));
    if !uses || source.contains("GL_OES_standard_derivatives") {
        return source;
    }
    let line = "#extension GL_OES_standard_derivatives : enable\n";
    let version = source
        .match_indices("#version")
        .map(|(i, _)| i)
        .find(|&i| i == 0 || source[..i].trim_end_matches([' ', '\t']).ends_with('\n'));
    let Some(start) = version else {
        return format!("{line}{source}");
    };
    let end = source[start..]
        .find('\n')
        .map_or(source.len(), |n| start + n + 1);
    // The version is the token after `#version`, also after a tab.
    let token = source[start + "#version".len()..end]
        .split_whitespace()
        .next();
    if token != Some("100") {
        return source;
    }
    let (head, rest) = source.split_at(end);
    let newline = if head.ends_with('\n') { "" } else { "\n" };
    format!("{head}{newline}{line}{rest}")
}

/// Compiles and links a fragment shader with the vertex shader of meshes.
/// Returns the log of the compiler on an error.
unsafe fn compile(gl: &ffi::Gles2, fragment: &str) -> Result<Program, String> {
    // SAFETY: plain GL calls on the current context with valid strings.
    unsafe {
        let vertex = shader(gl, ffi::VERTEX_SHADER, VERTEX)?;
        let fragment = match shader(gl, ffi::FRAGMENT_SHADER, fragment) {
            Ok(fragment) => fragment,
            Err(log) => {
                gl.DeleteShader(vertex);
                return Err(log);
            }
        };
        let program = gl.CreateProgram();
        gl.AttachShader(program, vertex);
        gl.AttachShader(program, fragment);
        gl.LinkProgram(program);
        gl.DeleteShader(vertex);
        gl.DeleteShader(fragment);
        let mut ok = 0;
        gl.GetProgramiv(program, ffi::LINK_STATUS, &raw mut ok);
        if ok == 0 {
            let log = info_log(gl, program, false);
            gl.DeleteProgram(program);
            return Err(log);
        }
        let location = |name: &str, uniform: bool| {
            let name = CString::new(name).expect("no nul");
            if uniform {
                gl.GetUniformLocation(program, name.as_ptr())
            } else {
                gl.GetAttribLocation(program, name.as_ptr())
            }
        };
        Ok(Program {
            id: program,
            position: location("position", false),
            uv: location("uv", false),
            tex: location("tex", true),
            alpha: location("alpha", true),
            size: location("size", true),
            time: location("time", true),
        })
    }
}

/// Compiles one shader, or returns the log of the compiler.
unsafe fn shader(gl: &ffi::Gles2, kind: ffi::types::GLenum, source: &str) -> Result<u32, String> {
    let source = CString::new(source).map_err(|_| "a nul in the shader".to_string())?;
    // SAFETY: plain GL calls on the current context with a valid string.
    unsafe {
        let shader = gl.CreateShader(kind);
        gl.ShaderSource(shader, 1, &source.as_ptr(), std::ptr::null());
        gl.CompileShader(shader);
        let mut ok = 0;
        gl.GetShaderiv(shader, ffi::COMPILE_STATUS, &raw mut ok);
        if ok == 0 {
            let log = info_log(gl, shader, true);
            gl.DeleteShader(shader);
            return Err(log);
        }
        Ok(shader)
    }
}

/// The info log of a shader or a program.
unsafe fn info_log(gl: &ffi::Gles2, object: u32, shader: bool) -> String {
    let mut buffer = vec![0u8; 4096];
    let mut length = 0;
    // SAFETY: the buffer holds as many bytes as GL may write.
    unsafe {
        let size = i32::try_from(buffer.len()).unwrap_or(0);
        if shader {
            gl.GetShaderInfoLog(object, size, &raw mut length, buffer.as_mut_ptr().cast());
        } else {
            gl.GetProgramInfoLog(object, size, &raw mut length, buffer.as_mut_ptr().cast());
        }
    }
    buffer.truncate(usize::try_from(length).unwrap_or(0));
    String::from_utf8_lossy(&buffer).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::{MAX_SOURCE, read_source, with_derivatives};

    /// A file of `/proc` claims 0 bytes and passes the size check; the
    /// read stops after the limit all the same (#31).
    #[test]
    fn a_proc_file_reads_no_more_than_the_limit() {
        let path = "/proc/self/smaps";
        assert!(std::fs::read(path).unwrap().len() as u64 > MAX_SOURCE);
        let err = read_source(path).err().expect("the read stops");
        assert!(err.contains("larger than"), "{err}");
    }

    #[test]
    fn derivatives_come_after_the_version() {
        let plain = "void main() {}".to_string();
        assert_eq!(with_derivatives(plain.clone()), plain);
        let edge = "float e = fwidth(x);".to_string();
        assert!(with_derivatives(edge).starts_with("#extension GL_OES_standard_derivatives"));
        let versioned = "#version 100\nfloat e = dFdx(x);".to_string();
        assert_eq!(
            with_derivatives(versioned),
            "#version 100\n#extension GL_OES_standard_derivatives : enable\nfloat e = dFdx(x);"
        );
        let enabled = "#extension GL_OES_standard_derivatives : enable\nfwidth".to_string();
        assert_eq!(with_derivatives(enabled.clone()), enabled);
        let commented = "// MIT\n#version 100\nfwidth".to_string();
        assert_eq!(
            with_derivatives(commented),
            "// MIT\n#version 100\n#extension GL_OES_standard_derivatives : enable\nfwidth"
        );
        let es3 = "// MIT\n#version 300 es // was 100\nfwidth".to_string();
        assert_eq!(with_derivatives(es3.clone()), es3);
        let tab = "#version\t100\nfwidth".to_string();
        assert_eq!(
            with_derivatives(tab),
            "#version\t100\n#extension GL_OES_standard_derivatives : enable\nfwidth"
        );
    }
}
