//! The asset folder of a block: where it is and what `values.yaml` holds.

use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

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
