//! The settings of a plugin (D62): a serde type that the plugin checks,
//! kept in a YAML file. The plugin reads the file at its start and is its
//! only writer: `settings.set` checks a change, writes the file and hands
//! the change to the plugin. Edits by hand apply at the next start.

use std::{
    io::Write as _,
    path::{Path, PathBuf},
};

use chora_scene::ipc::{DescribedMethod, ExportMethod, RpcError};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

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
}

/// The settings behind `settings.get` and `settings.set`, for the runtime.
pub trait Store {
    /// The settings as JSON.
    fn get(&self) -> Value;

    /// Replaces the top-level keys of `changes`, checks the result and
    /// writes the file; answers the new settings.
    ///
    /// # Errors
    ///
    /// A change is wrong, or the file cannot be written.
    fn set(&mut self, changes: Value) -> Result<Value, RpcError>;
}

impl<S: Settings> File<S> {
    /// Reads the settings at `path`, or `default` without a file there.
    ///
    /// # Errors
    ///
    /// The file cannot be read, or its settings are wrong.
    pub fn load(path: PathBuf, default: &str) -> Result<Self, String> {
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => default.into(),
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        let value = parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Self { value, path })
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
        let mut schema = schemars::generate::SchemaSettings::draft2020_12()
            .into_generator()
            .into_root_schema_for::<S>()
            .to_value();
        if let Some(root) = schema.as_object_mut() {
            root.remove("$schema");
            root.remove("title");
            // `settings.set` takes the keys to change only.
            root.remove("required");
        }
        vec![
            runtime::method("settings.get", "Answers the settings.", json!({}), &[]),
            ExportMethod::Described(DescribedMethod {
                name: "settings.set".into(),
                description: Some(
                    "Changes the settings at once and saves them: each key replaces its value."
                        .into(),
                ),
                input_schema: Some(schema),
            }),
        ]
    }

    /// Writes a temporary file and renames it, so that a crash leaves the
    /// old file or the new one, never a part.
    fn save(&self, value: &S) -> Result<(), String> {
        let text = serde_saphyr::to_string(value).map_err(|e| e.to_string())?;
        let mut tmp = self.path.clone().into_os_string();
        tmp.push(".tmp");
        let write = || -> std::io::Result<()> {
            if let Some(dir) = self.path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let mut file = std::fs::File::create(&tmp)?;
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
            std::fs::rename(&tmp, &self.path)
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
        for (key, value) in changes {
            merged[key] = value;
        }
        let value: S = serde_json::from_value(merged).map_err(|e| invalid(e.to_string()))?;
        value.check().map_err(invalid)?;
        self.save(&value)
            .map_err(|e| RpcError::new(RpcError::REJECTED, e))?;
        self.value = value;
        Ok(self.get())
    }
}

/// Parses settings in YAML and checks them.
///
/// # Errors
///
/// The text does not fit `S`, or a check fails.
pub fn parse<S: Settings>(text: &str) -> Result<S, String> {
    let mut options = serde_saphyr::Options::default();
    options.strict_booleans = true;
    options.reject_unsupported_tags = true;
    let value: S = serde_saphyr::from_str_with_options(text, options).map_err(|e| e.to_string())?;
    value.check()?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde::Deserialize;

    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
    #[serde(deny_unknown_fields, rename_all = "snake_case")]
    enum Action {
        Close,
        Run(String),
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
    #[serde(deny_unknown_fields)]
    struct Demo {
        gap: u16,
        opacity: f64,
        bindings: BTreeMap<String, Action>,
    }

    impl Settings for Demo {
        fn check(&self) -> Result<(), String> {
            if (0.0..=1.0).contains(&self.opacity) {
                return Ok(());
            }
            Err("opacity: from 0 to 1".into())
        }
    }

    const DEFAULT: &str = "gap: 4\nopacity: 1.0\nbindings:\n  MOD+q: close\n  MOD+r: {run: foot}\n";

    fn dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("chora-kit-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn set_checks_saves_and_answers_the_new_settings() {
        let dir = dir("set");
        let path = dir.join("chora/demo.yaml");
        let mut file = File::<Demo>::load(path.clone(), DEFAULT).unwrap();
        assert_eq!(file.get()["gap"], 4);
        assert!(!path.exists(), "a load without a file writes none");
        let answer = file.set(json!({"gap": 8})).unwrap();
        assert_eq!(answer["gap"], 8);
        assert_eq!(answer["opacity"], 1.0, "the other keys stay");
        assert_eq!(file.value.gap, 8);
        let saved = File::<Demo>::load(path.clone(), "").unwrap();
        assert_eq!(saved.value, file.value, "the file round-trips");
        assert!(!dir.join("chora/demo.yaml.tmp").exists());
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
        assert_eq!(names, ["settings.get", "settings.set"]);
        let ExportMethod::Described(set) = &methods[1] else {
            panic!("settings.set has a schema");
        };
        let schema = set.input_schema.as_ref().unwrap();
        assert!(schema.get("required").is_none());
        assert_eq!(schema["properties"]["gap"]["type"], "integer");
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
