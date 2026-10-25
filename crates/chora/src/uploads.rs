//! The assets that clients hand to the core with `assets.put`. A
//! client in another mount namespace, for example an agent in a container,
//! cannot name a file that the core sees. The core keeps its files in
//! `$XDG_RUNTIME_DIR/chora/SOCKET.assets/CLIENT/`, with the client name in
//! hex, and answers with the absolute path for image nodes, shader nodes
//! and effects. The files of a client go when its nodes go, and all files
//! go when chora exits.

use std::{
    collections::HashMap,
    fmt::Write as _,
    fs,
    io::{self, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use chora_scene::ipc::{Asset, AssetPut, AssetRemove, RpcError};
use serde_json::Value;

use crate::{
    ipc::{params_of, rejected},
    state::State,
};

/// The largest file; its base64 fits in one IPC line.
const MAX_FILE: usize = 512 << 10;
/// The most files of one client.
const MAX_FILES: usize = 64;
/// The most bytes of all files.
const MAX_TOTAL: usize = 64 << 20;
/// The longest file name.
const MAX_NAME: usize = 64;
/// The longest client name with assets: twice as long in hex, it stays
/// below the limit of 255 bytes for a path part.
const MAX_CLIENT_NAME: usize = 120;

/// The files of the clients.
#[derive(Default)]
pub struct Uploads {
    /// The directory; `None` until the IPC socket opens.
    dir: Option<PathBuf>,
    /// The size of each file, by client and file name.
    files: HashMap<String, HashMap<String, usize>>,
}

impl Drop for Uploads {
    fn drop(&mut self) {
        if let Some(dir) = &self.dir {
            let _ = fs::remove_dir_all(dir);
        }
    }
}

impl Uploads {
    /// Uses a directory, and removes what a crashed chora left there.
    pub fn open(&mut self, dir: PathBuf) -> io::Result<()> {
        match fs::remove_dir_all(&dir) {
            Err(err) if err.kind() != io::ErrorKind::NotFound => return Err(err),
            _ => {}
        }
        self.dir = Some(dir);
        Ok(())
    }

    /// Stores a file of a client, or replaces it, and returns its path.
    /// A limit of the assets answers with the limit code.
    pub fn put(&mut self, client: &str, name: &str, data: &[u8]) -> Result<PathBuf, RpcError> {
        check_name(name).map_err(rejected)?;
        // In hex, the name of the directory must stay within a path part.
        if client.len() > MAX_CLIENT_NAME {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!("a client name of more than {MAX_CLIENT_NAME} bytes stores no assets"),
            ));
        }
        if data.len() > MAX_FILE {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!("{name} has more than {} KiB", MAX_FILE >> 10),
            ));
        }
        let root = self
            .dir
            .as_ref()
            .ok_or_else(|| rejected("chora has no asset directory"))?;
        let old = self.files.get(client).and_then(|f| f.get(name)).copied();
        let count = self.files.get(client).map_or(0, HashMap::len);
        if old.is_none() && count >= MAX_FILES {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!("a client has at most {MAX_FILES} assets"),
            ));
        }
        let total: usize = self.files.values().flat_map(HashMap::values).sum();
        if total - old.unwrap_or(0) + data.len() > MAX_TOTAL {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!("all assets together have at most {} MiB", MAX_TOTAL >> 20),
            ));
        }
        let dir = root.join(hex(client));
        let path = write(&dir, name, data).map_err(|e| rejected(format!("{name}: {e}")))?;
        self.files
            .entry(client.to_string())
            .or_default()
            .insert(name.to_string(), data.len());
        Ok(path)
    }

    /// Removes a file of a client.
    pub fn remove(&mut self, client: &str, name: &str) -> Result<(), String> {
        let files = self.files.get_mut(client);
        if files.and_then(|f| f.remove(name)).is_none() {
            return Err(format!("no asset {name:?}"));
        }
        if let Some(root) = &self.dir {
            fs::remove_file(root.join(hex(client)).join(name))
                .map_err(|e| format!("{name}: {e}"))?;
        }
        Ok(())
    }

    /// Whether a path names a file that a client stored.
    pub fn owns(&self, client: &str, path: &Path) -> bool {
        let (Some(root), Some(name)) = (&self.dir, path.file_name()) else {
            return false;
        };
        path.parent() == Some(&root.join(hex(client)))
            && name
                .to_str()
                .is_some_and(|n| self.files.get(client).is_some_and(|f| f.contains_key(n)))
    }

    /// Removes all files of a client.
    pub fn drop_client(&mut self, client: &str) {
        if self.files.remove(client).is_some()
            && let Some(root) = &self.dir
        {
            let _ = fs::remove_dir_all(root.join(hex(client)));
        }
    }
}

/// A file name of one part, without a hidden or a temporary file.
fn check_name(name: &str) -> Result<(), String> {
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-');
    if name.is_empty()
        || name.len() > MAX_NAME
        || name.starts_with('.')
        || !name.chars().all(allowed)
    {
        return Err(format!(
            "{name:?} is no asset name: letters, digits, '.', '_' and '-', not first a '.', \
             at most {MAX_NAME}"
        ));
    }
    Ok(())
}

/// The bytes of a name in hex, as a directory name.
fn hex(name: &str) -> String {
    name.bytes().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

/// Writes a file through a temporary file and a rename, so that a reader
/// sees the old file or the new one, never a part.
fn write(dir: &Path, name: &str, data: &[u8]) -> io::Result<PathBuf> {
    fs::DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(dir)?;
    let part = dir.join(format!(".{name}.part"));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&part)?;
    file.write_all(data)?;
    let path = dir.join(name);
    fs::rename(&part, &path)?;
    Ok(path)
}

impl State {
    /// `assets.put` and `assets.remove`.
    pub fn ipc_assets(
        &mut self,
        conn: u64,
        method: &str,
        params: Value,
    ) -> Result<Value, RpcError> {
        let Some(client) = self.ipc.name(conn).map(str::to_owned) else {
            return Err(rejected("send hello first"));
        };
        if method == "assets.remove" {
            let remove: AssetRemove = params_of(params)?;
            self.ipc
                .uploads
                .remove(&client, &remove.name)
                .map_err(rejected)?;
            return Ok(Value::Null);
        }
        let put: AssetPut = params_of(params)?;
        let data = STANDARD.decode(&put.data).map_err(|e| {
            RpcError::new(RpcError::INVALID_PARAMS, format!("data is no base64: {e}"))
        })?;
        let path = self.ipc.uploads.put(&client, &put.name, &data)?;
        let path = path
            .to_str()
            .ok_or_else(|| rejected("the asset directory is no UTF-8 path"))?;
        // A replace must reach the nodes that show the file: the docs
        // promise that they load it again.
        self.assets.images.reload(path);
        Ok(serde_json::json!(Asset { path: path.into() }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uploads(test: &str) -> (Uploads, PathBuf) {
        let dir = std::env::temp_dir().join(format!("chora-uploads-{test}-{}", std::process::id()));
        let mut uploads = Uploads::default();
        uploads.open(dir.clone()).unwrap();
        (uploads, dir)
    }

    #[test]
    fn a_put_replaces_and_a_remove_removes() {
        let (mut uploads, _dir) = uploads("replace");
        let path = uploads.put("agent", "glow.frag", b"one").unwrap();
        assert!(path.is_absolute() && path.ends_with("6167656e74/glow.frag"));
        assert_eq!(uploads.put("agent", "glow.frag", b"two").unwrap(), path);
        assert_eq!(fs::read(&path).unwrap(), b"two");
        uploads.remove("agent", "glow.frag").unwrap();
        assert!(!path.exists());
        assert!(uploads.remove("agent", "glow.frag").is_err());
    }

    #[test]
    fn names_stay_in_the_directory_of_the_client() {
        let (mut uploads, _dir) = uploads("names");
        for name in [
            "",
            "../x",
            "a/b",
            ".hidden",
            ".x.part",
            "ä",
            &"x".repeat(65),
        ] {
            assert!(uploads.put("agent", name, b"x").is_err(), "{name:?}");
        }
        assert!(uploads.put("agent", "Sky_2.png", b"x").is_ok());
        assert!(uploads.put(&"a".repeat(MAX_CLIENT_NAME), "x", b"x").is_ok());
        assert!(
            uploads
                .put(&"a".repeat(MAX_CLIENT_NAME + 1), "x", b"x")
                .is_err()
        );
    }

    #[test]
    fn an_upload_beyond_the_size_or_file_limit_fails() {
        let (mut uploads, _dir) = uploads("limits");
        let error = uploads
            .put("agent", "big", &vec![0; MAX_FILE + 1])
            .expect_err("a file beyond the size limit");
        assert_eq!(error.code, RpcError::LIMIT_REACHED);
        for i in 0..MAX_FILES {
            uploads.put("agent", &format!("{i}"), b"x").unwrap();
        }
        let error = uploads
            .put("agent", "one-more", b"x")
            .expect_err("a file beyond the count limit");
        assert_eq!(error.code, RpcError::LIMIT_REACHED);
        assert!(
            uploads.put("agent", "0", b"y").is_ok(),
            "a replace needs no room"
        );
    }

    #[test]
    fn the_files_of_a_client_go_with_it_and_all_go_at_the_end() {
        let (mut uploads, dir) = uploads("drop");
        let mine = uploads.put("agent", "a.png", b"x").unwrap();
        let other = uploads.put("panel", "a.png", b"x").unwrap();
        uploads.drop_client("agent");
        assert!(!mine.exists() && other.exists());
        drop(uploads);
        assert!(!dir.exists());
    }
}
