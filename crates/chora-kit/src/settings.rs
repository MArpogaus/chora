//! The settings of a plugin (D62): a serde type that the plugin checks,
//! kept in a YAML file. The plugin reads the file at its start and is its
//! only writer: `settings.set` checks a change, hands it to the plugin to
//! apply and then writes the file. A change that the plugin cannot apply
//! is not saved; a change that cannot be saved stays applied, and the
//! caller gets the error. Edits by hand apply with `settings.reload`, which
//! reads the file again, or at the next start.
//!
//! A key of a struct that the file leaves out takes its value of the
//! default file; a map in the file replaces the default map. A change of
//! `settings.set` is a JSON merge patch (RFC 7386): structs and maps merge,
//! lists and other values replace, also an enum in an object, and `null`
//! removes the key, so that a map loses the entry and a field of a struct
//! takes its default again.

use std::{
    io::Write as _,
    os::unix::fs::OpenOptionsExt as _,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use chora_scene::ipc::{DescribedMethod, ExportMethod, RpcError};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use crate::runtime;

/// The settings type of a plugin.
pub trait Settings: Serialize + DeserializeOwned + schemars::JsonSchema {
    /// Checks what serde cannot, for example a range.
    ///
    /// # Errors
    ///
    /// A value is wrong; the message names it.
    fn check(&self) -> Result<(), String>;
}

/// The settings of a plugin and their file.
pub struct File<S> {
    /// The settings as the plugin applies them.
    pub value: S,
    path: PathBuf,
    /// The default file, for the keys that a change or a file leaves out.
    default: &'static str,
    /// The settings before the last `set` or `reload`, for `undo`.
    previous: Option<S>,
}

/// The settings behind `settings.get` and `settings.set`, for the runtime.
pub trait Store {
    /// The settings as JSON.
    fn get(&self) -> Value;

    /// Merges `changes` as a JSON merge patch, checks the result and
    /// takes it, without a save; answers the new settings.
    ///
    /// # Errors
    ///
    /// A change is wrong; the settings stay.
    fn set(&mut self, changes: Value) -> Result<Value, RpcError>;

    /// Writes the settings to the file.
    ///
    /// # Errors
    ///
    /// The file cannot be written.
    fn save(&self) -> Result<(), RpcError>;

    /// Reads the file again and takes its settings; answers them.
    ///
    /// # Errors
    ///
    /// The file cannot be read, or its settings are wrong; the settings
    /// stay.
    fn reload(&mut self) -> Result<Value, RpcError>;

    /// Takes back the last `set` or `reload`, after the plugin failed to
    /// apply it.
    fn undo(&mut self);
}

impl<S: Settings> File<S> {
    /// Reads the settings at `path`, or `default` without a file there.
    /// A key that the file leaves out takes its value of `default`.
    ///
    /// # Errors
    ///
    /// The file cannot be read, or its settings are wrong.
    pub fn load(path: PathBuf, default: &'static str) -> Result<Self, String> {
        let text = match crate::read_text(&path, crate::MAX_TEXT) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => default.into(),
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        // One path for every file, so that a key it leaves out takes its
        // default also where the type has an `Option`. A wrong value then
        // has no line number; a wrong YAML still has one.
        let value = serde_saphyr::from_str_with_options(&text, options())
            .map_err(|e| e.to_string())
            .and_then(|v| complete(v, default))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Self {
            value,
            path,
            default,
            previous: None,
        })
    }

    /// The file that `--config` names, else `chora/NAME.yaml` in
    /// `$XDG_CONFIG_HOME`.
    ///
    /// # Errors
    ///
    /// Neither `$XDG_CONFIG_HOME` nor `$HOME` is an absolute path.
    pub fn path(arg: Option<PathBuf>, name: &str) -> Result<PathBuf, String> {
        arg.or_else(|| crate::xdg::config_home().map(|d| d.join(format!("chora/{name}.yaml"))))
            .ok_or_else(|| "no $XDG_CONFIG_HOME or $HOME".into())
    }

    /// The file of the settings.
    #[must_use]
    pub fn file(&self) -> &Path {
        &self.path
    }

    /// `settings.get` and `settings.set`, with the schema of `S`.
    #[must_use]
    pub fn methods() -> Vec<ExportMethod> {
        let mut schema = schema::<S>();
        if let Some(root) = schema.as_object_mut() {
            root.remove("$schema");
            root.remove("title");
            // `settings.set` takes the keys to change only, also of a
            // nested struct.
            root.remove("required");
            let defs = root.get_mut("$defs").and_then(Value::as_object_mut);
            for def in defs.into_iter().flat_map(|d| d.values_mut()) {
                def.as_object_mut().map(|d| d.remove("required"));
            }
        }
        vec![
            runtime::method("settings.get", "Answers the settings.", json!({}), &[]),
            runtime::privileged(runtime::method(
                "settings.reload",
                "Reads the settings file again, after an edit by hand, and applies it.",
                json!({}),
                &[],
            )),
            ExportMethod::Described(DescribedMethod {
                name: "settings.set".into(),
                description: Some(
                    "Changes the settings at once and saves them, as a JSON merge patch: \
                     structs and maps merge, other values replace, null resets a key to \
                     its default or removes a map entry."
                        .into(),
                ),
                input_schema: Some(schema),
                privileged: true,
            }),
        ]
    }

    /// Writes a new temporary file of mode 0600 next to the file and
    /// renames it, so that a crash leaves the old file or the new one,
    /// never a part. A symlink, for example into a dotfiles repository,
    /// stays: the file that it points to takes the settings.
    fn write(&self) -> Result<(), String> {
        let text = serde_saphyr::to_string(&self.value).map_err(|e| e.to_string())?;
        let write = || -> std::io::Result<()> {
            let dir = self.path.parent().unwrap_or(Path::new("."));
            std::fs::create_dir_all(dir)?;
            let target = std::fs::canonicalize(&self.path).unwrap_or_else(|_| {
                // A dangling symlink still names its target.
                std::fs::read_link(&self.path).map_or_else(|_| self.path.clone(), |l| dir.join(l))
            });
            let (tmp, mut file) = new_temp(&target)?;
            let done = file
                .write_all(text.as_bytes())
                .and_then(|()| file.sync_all())
                .and_then(|()| std::fs::rename(&tmp, &target));
            if done.is_err() {
                let _ = std::fs::remove_file(&tmp);
            }
            done
        };
        write().map_err(|e| format!("{}: {e}", self.path.display()))
    }
}

impl<S: Settings> Store for File<S> {
    fn get(&self) -> Value {
        serde_json::to_value(&self.value).expect("settings serialize to JSON")
    }

    fn set(&mut self, changes: Value) -> Result<Value, RpcError> {
        let invalid = |e: String| RpcError::new(RpcError::INVALID_PARAMS, e);
        let Value::Object(changes) = changes else {
            return Err(invalid("settings.set takes an object".into()));
        };
        let mut merged = self.get();
        let schema = schema::<S>();
        merge(&mut merged, Value::Object(changes), &schema, &schema);
        let value = complete(merged, self.default).map_err(invalid)?;
        self.previous = Some(std::mem::replace(&mut self.value, value));
        Ok(self.get())
    }

    fn save(&self) -> Result<(), RpcError> {
        self.write()
            .map_err(|e| RpcError::new(RpcError::REJECTED, e))
    }

    fn reload(&mut self) -> Result<Value, RpcError> {
        let value = Self::load(self.path.clone(), self.default)
            .map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, e))?
            .value;
        self.previous = Some(std::mem::replace(&mut self.value, value));
        Ok(self.get())
    }

    fn undo(&mut self) {
        if let Some(previous) = self.previous.take() {
            self.value = previous;
        }
    }
}

/// Creates a temporary file of mode 0600 with a new name next to `path`;
/// an existing file of the name is never opened.
fn new_temp(path: &Path) -> std::io::Result<(PathBuf, std::fs::File)> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    loop {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let tmp = path.with_file_name(format!(".{name}.{}.{n}.tmp", std::process::id()));
        let open = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&tmp);
        match open {
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            open => return open.map(|file| (tmp, file)),
        }
    }
}

/// Parses settings in YAML and checks them.
///
/// # Errors
///
/// The text does not fit `S`, or a check fails.
pub fn parse<S: Settings>(text: &str) -> Result<S, String> {
    let value: S =
        serde_saphyr::from_str_with_options(text, options()).map_err(|e| e.to_string())?;
    value.check()?;
    Ok(value)
}

fn options() -> serde_saphyr::Options {
    let mut options = serde_saphyr::Options::default();
    options.strict_booleans = true;
    options.reject_unsupported_tags = true;
    options
}

fn schema<S: Settings>() -> Value {
    schemars::generate::SchemaSettings::draft2020_12()
        .into_generator()
        .into_root_schema_for::<S>()
        .to_value()
}

/// Settings that may leave keys of structs out, which take their values
/// of the default file; then checked.
fn complete<S: Settings>(mut value: Value, default: &str) -> Result<S, String> {
    let default = serde_json::to_value(parse::<S>(default)?).map_err(|e| e.to_string())?;
    let schema = schema::<S>();
    fill(&mut value, &default, &schema, &schema);
    let value: S = serde_json::from_value(value).map_err(|e| e.to_string())?;
    value.check()?;
    Ok(value)
}

/// Adds the keys of `default` that a struct of `value` leaves out. The
/// schema tells a struct, which has `properties`, from a map, which keeps
/// its own entries.
fn fill(value: &mut Value, default: &Value, schema: &Value, root: &Value) {
    let schema = resolve(schema, root);
    let (Some(properties), Value::Object(value), Value::Object(default)) =
        (schema.get("properties"), value, default)
    else {
        return;
    };
    for (key, default) in default {
        match value.get_mut(key) {
            Some(value) => fill(value, default, &properties[key], root),
            None => {
                value.insert(key.clone(), default.clone());
            }
        }
    }
}

/// The schema behind a `$ref`, or the struct of an `Option`.
fn resolve<'a>(schema: &'a Value, root: &'a Value) -> &'a Value {
    if let Some(name) = schema["$ref"]
        .as_str()
        .and_then(|r| r.strip_prefix("#/$defs/"))
    {
        return &root["$defs"][name];
    }
    schema["anyOf"]
        .as_array()
        .and_then(|any| {
            any.iter()
                .map(|s| resolve(s, root))
                .find(|s| s.get("properties").is_some())
        })
        .unwrap_or(schema)
}

/// Applies a JSON merge patch (RFC 7386) where the schema has a struct or
/// a map; any other value, such as an enum, takes the patch as it is, so
/// that two variants never merge into one object.
fn merge(target: &mut Value, patch: Value, schema: &Value, root: &Value) {
    let schema = resolve(schema, root);
    let fields = schema.get("properties");
    let entries = schema.get("additionalProperties").filter(|s| s.is_object());
    let Value::Object(patch) = patch else {
        *target = patch;
        return;
    };
    if fields.is_none() && entries.is_none() {
        *target = Value::Object(patch);
        return;
    }
    if !target.is_object() {
        *target = Value::Object(Map::new());
    }
    let Value::Object(target) = target else {
        unreachable!("an object since the line above")
    };
    for (key, value) in patch {
        if value.is_null() {
            target.remove(&key);
        } else {
            let schema = fields.map_or(entries.unwrap_or(&Value::Null), |f| &f[&key]);
            merge(
                target.entry(key).or_insert(Value::Null),
                value,
                schema,
                root,
            );
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeMap;

    use serde::Deserialize;

    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
    #[serde(deny_unknown_fields, rename_all = "snake_case")]
    enum Action {
        Close,
        Run(String),
        Spawn(String),
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
    #[serde(deny_unknown_fields)]
    pub(crate) struct Demo {
        pub(crate) gap: u16,
        opacity: f64,
        bindings: BTreeMap<String, Action>,
        frame: Frame,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
    #[serde(deny_unknown_fields)]
    struct Frame {
        width: u16,
        title: Option<String>,
    }

    impl Settings for Demo {
        fn check(&self) -> Result<(), String> {
            if (0.0..=1.0).contains(&self.opacity) {
                return Ok(());
            }
            Err("opacity: from 0 to 1".into())
        }
    }

    pub(crate) const DEFAULT: &str = "gap: 4\nopacity: 1.0\nbindings:\n  MOD+q: close\n  MOD+r: {run: foot}\n\
                           frame: {width: 1, title: demo}\n";

    pub(crate) fn dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("chora-kit-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn set_checks_and_answers_the_new_settings() {
        let dir = dir("set");
        let path = dir.join("chora/demo.yaml");
        let mut file = File::<Demo>::load(path.clone(), DEFAULT).unwrap();
        assert_eq!(file.get()["gap"], 4);
        assert!(!path.exists(), "a load without a file writes none");
        let answer = file.set(json!({"gap": 8})).unwrap();
        assert_eq!(answer["gap"], 8);
        assert_eq!(answer["opacity"], 1.0, "the other keys stay");
        assert_eq!(file.value.gap, 8);
        assert!(!path.exists(), "set saves nothing");
        file.save().unwrap();
        let saved = File::<Demo>::load(path.clone(), DEFAULT).unwrap();
        assert_eq!(saved.value, file.value, "the file round-trips");
        let names: Vec<_> = std::fs::read_dir(dir.join("chora")).unwrap().collect();
        assert_eq!(names.len(), 1, "no temporary file stays: {names:?}");
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// The saved file has mode 0600, and a symlink to the settings, for
    /// example into a dotfiles repository, stays a symlink.
    #[test]
    fn a_save_writes_through_a_symlink_with_mode_0600() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = dir("symlink");
        std::fs::create_dir_all(dir.join("dotfiles")).unwrap();
        let real = dir.join("dotfiles/demo.yaml");
        std::fs::write(&real, "gap: 5\n").unwrap();
        let link = dir.join("demo.yaml");
        std::os::unix::fs::symlink("dotfiles/demo.yaml", &link).unwrap();
        let mut file = File::<Demo>::load(link.clone(), DEFAULT).unwrap();
        file.set(json!({"gap": 6})).unwrap();
        file.save().unwrap();
        assert!(link.symlink_metadata().unwrap().is_symlink());
        assert!(std::fs::read_to_string(&real).unwrap().contains("gap: 6"));
        let mode = real.metadata().unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
        // A dangling symlink keeps its target too.
        std::fs::remove_file(&real).unwrap();
        file.save().unwrap();
        assert!(link.symlink_metadata().unwrap().is_symlink());
        assert!(real.exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_wrong_change_fails_and_changes_nothing() {
        let path = dir("wrong").join("demo.yaml");
        let mut file = File::<Demo>::load(path.clone(), DEFAULT).unwrap();
        for wrong in [
            json!({"opacity": 2.0}),
            json!({"gap": -1}),
            json!({"border": 2}),
            json!({"frame": {"depth": 2}}),
            json!([1]),
        ] {
            let err = file.set(wrong.clone()).unwrap_err();
            assert_eq!(err.code, RpcError::INVALID_PARAMS, "{wrong}");
        }
        assert_eq!(file.value.opacity, 1.0);
        assert!(!path.exists(), "a wrong change writes no file");
    }

    #[test]
    fn the_schema_of_set_takes_single_keys() {
        let methods = File::<Demo>::methods();
        let names: Vec<&str> = methods.iter().map(ExportMethod::name).collect();
        assert_eq!(names, ["settings.get", "settings.reload", "settings.set"]);
        let ExportMethod::Described(set) = &methods[2] else {
            panic!("settings.set has a schema");
        };
        let schema = set.input_schema.as_ref().unwrap();
        assert!(schema.get("required").is_none());
        assert!(schema["$defs"]["Frame"].get("required").is_none());
        assert_eq!(schema["properties"]["gap"]["type"], "integer");
    }

    /// `settings.set` is a JSON merge patch (RFC 7386): structs and maps merge,
    /// `null` removes a map entry and resets a field to its default.
    #[test]
    fn set_merges_a_patch() {
        let path = dir("merge").join("demo.yaml");
        let mut file = File::<Demo>::load(path.clone(), DEFAULT).unwrap();
        file.set(json!({"frame": {"width": 3}, "gap": 8})).unwrap();
        assert_eq!(file.value.frame.width, 3);
        assert_eq!(
            file.value.frame.title.as_deref(),
            Some("demo"),
            "a nested key stays"
        );
        file.set(json!({"bindings": {"MOD+x": "close", "MOD+q": null}}))
            .unwrap();
        let keys: Vec<&str> = file.value.bindings.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            ["MOD+r", "MOD+x"],
            "a map merges, null removes an entry"
        );
        file.set(json!({"gap": null, "frame": {"title": null}}))
            .unwrap();
        assert_eq!(file.value.gap, 4, "null resets a field to its default");
        assert_eq!(file.value.frame.title.as_deref(), Some("demo"));
        assert_eq!(file.value.frame.width, 3);
        file.set(json!({"bindings": {"MOD+r": {"spawn": "kitty"}}}))
            .unwrap();
        assert_eq!(
            file.value.bindings["MOD+r"],
            Action::Spawn("kitty".into()),
            "an entry that is no struct takes the patch as it is"
        );
        let err = file.set(json!({"frame": {"width": "wide"}})).unwrap_err();
        assert_eq!(err.code, RpcError::INVALID_PARAMS);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    /// A key that a file leaves out takes its default, also in a nested
    /// struct; a map in the file replaces the default map.
    #[test]
    fn a_file_without_keys_takes_their_defaults() {
        let dir = dir("partial");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("demo.yaml");
        std::fs::write(
            &path,
            "gap: 7\nbindings: {MOD+x: close}\nframe: {width: 2}\n",
        )
        .unwrap();
        let file = File::<Demo>::load(path, DEFAULT).unwrap();
        assert_eq!(file.value.gap, 7);
        assert_eq!(file.value.opacity, 1.0);
        assert_eq!(file.value.bindings.len(), 1);
        assert_eq!(file.value.frame.width, 2);
        assert_eq!(file.value.frame.title.as_deref(), Some("demo"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// A file with every key that serde needs still takes the default of
    /// an `Option` that it leaves out.
    #[test]
    fn a_file_without_an_option_takes_its_default() {
        let dir = dir("option");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("demo.yaml");
        std::fs::write(
            &path,
            "gap: 7\nopacity: 1.0\nbindings: {}\nframe: {width: 2}\n",
        )
        .unwrap();
        let file = File::<Demo>::load(path, DEFAULT).unwrap();
        assert_eq!(file.value.frame.title.as_deref(), Some("demo"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_wrong_file_fails_with_its_path() {
        let dir = dir("load");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("demo.yaml");
        std::fs::write(&path, "gap: 4\nopacity: 3\nbindings: {}\n").unwrap();
        let err = File::<Demo>::load(path.clone(), DEFAULT).err().unwrap();
        assert!(
            err.contains("demo.yaml") && err.contains("opacity"),
            "{err}"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
