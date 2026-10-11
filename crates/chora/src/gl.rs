//! The raw GL calls of layers: the program of meshes, the programs of
//! effects, and drawing a texture through triangles with one of them.
//!
//! GL calls are `unsafe`; this module is the one place with them. The state
//! that changes here goes back to what Smithay sets before its own draws:
//! its draw calls bind their program, texture and arrays each time.
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
        gl.Uniform1i(program.tex, 0);
        gl.Uniform1f(program.alpha, call.alpha);
        gl.Uniform2f(program.size, call.size[0], call.size[1]);
        gl.Uniform1f(program.time, seconds);
        for (name, value) in call.uniforms.into_iter().flatten() {
            set_uniform(gl, program.id, name, value);
        }
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
        gl.BindTexture(ffi::TEXTURE_2D, 0);
    })
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
        Ok(program) => (Some(program), None),
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

fn read_source(path: &str) -> Result<String, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("{path}: {e}"))?;
    if meta.len() > MAX_SOURCE {
        return Err(format!("{path}: larger than {MAX_SOURCE} bytes"));
    }
    std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
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
