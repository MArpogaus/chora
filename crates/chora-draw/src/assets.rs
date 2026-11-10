//! The asset folder of a block: where it is and what `values.yaml` holds.

use std::path::{Path, PathBuf};

use resvg::usvg::{self, ImageHrefResolver, ImageKind};
use serde::de::DeserializeOwned;
use zune_jpeg::JpegDecoder;

/// The largest side of a picture of a HUD folder, in logical pixels.
pub const MAX_SIDE: u32 = 2048;

/// The most nodes of an SVG picture of a HUD folder, `<use>` copies and
/// the nodes of patterns, masks and clip paths included.
pub const MAX_NODES: usize = 10_000;

/// The most images and filters of an SVG picture of a HUD folder.
pub const MAX_COSTLY: usize = 16;

/// What a walk over a tree counts.
#[derive(Default)]
struct Count {
    nodes: usize,
    costly: usize,
}

impl Count {
    fn group(&mut self, group: &usvg::Group) -> Result<(), String> {
        self.costly += group.filters().len();
        if let Some(clip) = group.clip_path() {
            self.group(clip.root())?;
        }
        if let Some(mask) = group.mask() {
            self.group(mask.root())?;
        }
        for node in group.children() {
            self.nodes += 1;
            if self.nodes > MAX_NODES {
                return Err(format!("more than {MAX_NODES} nodes"));
            }
            match node {
                usvg::Node::Group(inner) => self.group(inner)?,
                usvg::Node::Image(_) => self.costly += 1,
                usvg::Node::Text(text) => self.group(text.flattened())?,
                usvg::Node::Path(_) => {}
            }
            if self.costly > MAX_COSTLY {
                return Err(format!("more than {MAX_COSTLY} images and filters"));
            }
        }
        Ok(())
    }
}

/// Whether an SVG tree is cheap to draw: at most [`MAX_SIDE`] on each
/// side, at most [`MAX_NODES`] nodes, at most [`MAX_COSTLY`] images and
/// filters, and no pattern tile over [`MAX_SIDE`] on a side.
///
/// # Errors
///
/// The tree is larger or costlier; the message says how.
pub fn check_tree(tree: &usvg::Tree) -> Result<(), String> {
    let size = tree.size();
    #[allow(clippy::cast_precision_loss)]
    let limit = MAX_SIDE as f32;
    if size.width() > limit || size.height() > limit {
        return Err(format!(
            "picture of {} x {} px, at most {MAX_SIDE}",
            size.width(),
            size.height()
        ));
    }
    let mut count = Count::default();
    count.group(tree.root())?;
    for pattern in tree.patterns() {
        let tile = pattern.rect().transform(pattern.transform());
        if tile.is_none_or(|t| t.width() > limit || t.height() > limit) {
            return Err(format!("pattern tile over {MAX_SIDE} px"));
        }
        count.group(pattern.root())?;
    }
    Ok(())
}

/// The width and height of a PNG or JPEG file from its header.
#[must_use]
pub fn raster_size(data: &[u8]) -> Option<[u32; 2]> {
    if data.starts_with(b"\x89PNG") {
        let head = data.get(16..24)?;
        let word =
            |at: usize| u32::from_be_bytes([head[at], head[at + 1], head[at + 2], head[at + 3]]);
        return Some([word(0), word(4)]);
    }
    let mut decoder = JpegDecoder::new(std::io::Cursor::new(data));
    decoder.decode_headers().ok()?;
    let (w, h) = decoder.dimensions()?;
    Some([u32::try_from(w).ok()?, u32::try_from(h).ok()?])
}

/// The picture if it is a PNG or JPEG within [`MAX_SIDE`]. An SVG is
/// refused, so that pictures do not nest.
fn admit(kind: ImageKind) -> Option<ImageKind> {
    let fits = |[w, h]: [u32; 2]| w <= MAX_SIDE && h <= MAX_SIDE;
    let ok = match &kind {
        ImageKind::PNG(data) | ImageKind::JPEG(data) => raster_size(data).is_some_and(fits),
        ImageKind::SVG(_) | ImageKind::GIF(_) | ImageKind::WEBP(_) => false,
    };
    ok.then_some(kind)
}

/// The resolver of the `href` of an `<image>` in a HUD folder. It takes a
/// `data:` URI, or a relative path of a file inside `dir`, read through
/// [`chora_kit::hud::file`]. Everything else draws nothing: an absolute
/// path, `..`, `file:` and `http:`, a link out of the folder, a file over
/// the size limit, a picture over [`MAX_SIDE`], and an SVG picture. The default resolver
/// of usvg reads any path of the machine.
#[must_use]
pub fn href_resolver(dir: &Path) -> ImageHrefResolver<'static> {
    let dir = dir.to_path_buf();
    let data = ImageHrefResolver::default_data_resolver();
    ImageHrefResolver {
        resolve_data: Box::new(move |mime, bytes, options| {
            data(mime, bytes, options).and_then(admit)
        }),
        resolve_string: Box::new(move |href, _options| {
            if href.contains(':') {
                return None;
            }
            let bytes = chora_kit::hud::file(&dir, href).ok()?;
            let kind = if bytes.starts_with(b"\x89PNG") {
                ImageKind::PNG(bytes.into())
            } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
                ImageKind::JPEG(bytes.into())
            } else {
                return None;
            };
            admit(kind)
        }),
    }
}

/// The asset folder: the given one, else the installed one if it exists,
/// else `source` below the root of the source tree, two levels above the
/// directory of `exe` (`target/<profile>/NAME`). Falls back to the
/// installed path, so that the error of a missing folder names it.
pub fn resolve(
    given: Option<PathBuf>,
    installed: &Path,
    exe: Option<&Path>,
    source: &str,
) -> PathBuf {
    if let Some(dir) = given {
        return dir;
    }
    if installed.is_dir() {
        return installed.to_path_buf();
    }
    exe.and_then(Path::parent)
        .and_then(|dir| dir.join("../..").join(source).canonicalize().ok())
        .filter(|dir| dir.is_dir())
        .unwrap_or_else(|| installed.to_path_buf())
}

/// The `values.yaml` of an asset folder. A key that the file leaves out
/// takes the value that `T` compiles in.
///
/// # Errors
///
/// The file is missing or does not fit `T`; the message names it.
pub fn values<T: DeserializeOwned>(dir: &Path) -> Result<T, String> {
    let path = dir.join("values.yaml");
    let text = chora_kit::read_text(&path, chora_kit::MAX_TEXT)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    serde_saphyr::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Checks that each file of an asset folder is a picture that loads.
///
/// # Errors
///
/// A file is missing or broken; the message names it.
pub fn check_pictures(theme: &crate::theme::Theme, files: &[&str]) -> Result<(), String> {
    for file in files {
        let path = theme.path(file);
        theme
            .image_size(&path)
            .ok_or_else(|| format!("{}: no picture", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolution_order() {
        let root = std::env::temp_dir().join(format!("chora-draw-assets-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let installed = root.join("usr/notify/assets");
        let exe = root.join("tree/target/release/chora-notify");
        let source = root.join("tree/plugins/chora-notify/assets");
        let given = root.join("given");
        let resolve = |given, installed: &Path, exe| {
            resolve(given, installed, exe, "plugins/chora-notify/assets")
        };
        std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
        assert_eq!(resolve(None, &installed, Some(&exe)), installed);
        std::fs::create_dir_all(&source).unwrap();
        assert_eq!(
            resolve(None, &installed, Some(&exe)),
            source.canonicalize().unwrap()
        );
        std::fs::create_dir_all(&installed).unwrap();
        assert_eq!(resolve(None, &installed, Some(&exe)), installed);
        assert_eq!(resolve(Some(given.clone()), &installed, Some(&exe)), given);
        assert_eq!(resolve(None, &installed, None), installed);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn values_name_their_file_and_fill_the_gaps() {
        #[derive(serde::Deserialize)]
        #[serde(default, deny_unknown_fields)]
        struct Values {
            size: i32,
            font: String,
        }
        impl Default for Values {
            fn default() -> Self {
                Self {
                    size: 4,
                    font: "sans-serif".into(),
                }
            }
        }
        let dir = std::env::temp_dir().join(format!("chora-draw-values-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let err = values::<Values>(&dir).err().unwrap();
        assert!(err.contains("values.yaml"), "{err}");
        std::fs::write(dir.join("values.yaml"), "size: 9\n").unwrap();
        let got = values::<Values>(&dir).unwrap();
        assert_eq!((got.size, got.font.as_str()), (9, "sans-serif"));
        std::fs::write(dir.join("values.yaml"), "wide: 1\n").unwrap();
        assert!(values::<Values>(&dir).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
