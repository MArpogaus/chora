//! Views (D35): YAML files with plugins and a part of the scene.
//!
//! `choractl view load NAME` starts the plugins of the view that do not run
//! yet, passes each its configuration through the method that its manifest
//! names, and applies the nodes of the view as one transaction. The client
//! `view-NAME` owns the nodes, so a second load replaces them,
//! `view unload` removes them, and `view save` writes them back.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use chora_scene::{
    Command, Node, NodeKind, Props, PropsUpdate, ROOT,
    ipc::{Apply, Client, Hello, MethodCall, Query},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

/// How long a load waits for a plugin to connect and export its methods.
const START_TIMEOUT: Duration = Duration::from_secs(10);

/// A view: plugins and a part of the scene.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View {
    /// The name, which also names the client `view-NAME`.
    pub name: String,
    /// The plugins that drive the view.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<PluginUse>,
    /// The nodes, parents before their children, bottom first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scene: Vec<ViewNode>,
}

/// A plugin of a view and its configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginUse {
    /// The client name, which also names the manifest.
    pub name: String,
    /// The parameters of the `configure` method of the manifest.
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub config: Value,
}

/// A node of a view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewNode {
    /// The id.
    pub id: String,
    /// `group`, `rect`, `image`, `shader` or `text`.
    pub kind: NodeKind,
    /// The parent; the root if missing.
    #[serde(default = "root", skip_serializing_if = "is_root")]
    pub parent: String,
    /// The properties.
    #[serde(default, skip_serializing_if = "is_default")]
    pub props: PropsUpdate,
}

/// A plugin manifest: how to start a plugin and how to configure it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// The client name of the plugin.
    pub name: String,
    /// The shell command that starts the plugin.
    pub exec: String,
    /// The exported method that takes the `config` of a view.
    #[serde(default)]
    pub configure: Option<String>,
    /// What the plugin does.
    #[serde(default)]
    pub description: Option<String>,
}

fn root() -> String {
    ROOT.into()
}

fn is_root(parent: &str) -> bool {
    parent == ROOT
}

fn is_default(props: &PropsUpdate) -> bool {
    *props == PropsUpdate::default()
}

/// The directories of a kind of file, for example `views`: the one in
/// `$XDG_CONFIG_HOME/chora/` first, then the one of the package.
fn dirs(kind: &str) -> Vec<PathBuf> {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")));
    let mut dirs: Vec<PathBuf> = config
        .map(|c| c.join("chora").join(kind))
        .into_iter()
        .collect();
    dirs.push(Path::new("/usr/share/chora").join(kind));
    dirs
}

/// The file of a name: a path as it is, or `NAME.yaml` in the first
/// directory of its kind that has it.
fn find(kind: &str, name: &str) -> Result<PathBuf, String> {
    if is_path(name) {
        return Ok(PathBuf::from(name));
    }
    check_name(name)?;
    dirs(kind)
        .into_iter()
        .map(|d| d.join(format!("{name}.yaml")))
        .find(|p| p.is_file())
        .ok_or_else(|| format!("no {kind} file {name}.yaml in {:?}", dirs(kind)))
}

/// Whether a name of the command line is the path of a file.
fn is_path(name: &str) -> bool {
    let yaml = Path::new(name)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("yaml"));
    name.contains('/') || yaml
}

/// Checks the name of a view or a plugin by the rule of the core. The
/// check carries the `view-` prefix of a view, so a plugin name stays a
/// few bytes below the limit of `hello`. A name that ends in `.yaml`
/// would load a file by path, so it is no name.
pub fn check_name(name: &str) -> Result<(), String> {
    let client = format!("view-{name}");
    if chora_scene::ipc::valid_name(name) && chora_scene::ipc::valid_name(&client) && !is_path(name)
    {
        Ok(())
    } else {
        Err(format!(
            "invalid name {name:?}: lowercase letters, digits, _, - and ., from a letter, at most {} bytes, not ending in .yaml",
            chora_scene::ipc::MAX_NAME
        ))
    }
}

/// Reads a YAML file strictly: unknown keys, duplicate keys, unknown tags
/// and the YAML 1.1 booleans such as `no` are errors.
pub fn read_yaml<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_yaml(&text).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn parse_yaml<T: DeserializeOwned>(text: &str) -> Result<T, String> {
    let mut options = serde_saphyr::Options::default();
    options.strict_booleans = true;
    options.reject_unsupported_tags = true;
    serde_saphyr::from_str_with_options(text, options).map_err(|e| e.to_string())
}

/// The client name of a view.
fn owner(view: &str) -> String {
    format!("view-{view}")
}

fn connect(view: &str) -> Result<Client, String> {
    let hello = Hello {
        name: owner(view),
        keep_nodes: true,
        ..Hello::default()
    };
    Client::from_env(&hello)
        .map_err(|e| format!("the view {view:?} connects as {}: {e}", owner(view)))
}

/// Loads a view and returns what it did.
/// A path loads the file; a name loads `NAME.yaml`, which must name the
/// same view.
pub fn load(name: &str) -> Result<Value, String> {
    let view: View = read_yaml(&find("views", name)?)?;
    if !is_path(name) && view.name != name {
        return Err(format!("{name}.yaml names the view {:?}", view.name));
    }
    check(&view)?;
    let mut client = connect(&view.name)?;
    let started = start_plugins(&mut client, &view.plugins)?;
    let mut commands = removals(&mut client, &view.name)?;
    commands.extend(view.scene.iter().map(|n| Command::Create {
        id: n.id.clone(),
        kind: n.kind,
        parent: n.parent.clone(),
        props: n.props.clone(),
    }));
    client
        .call::<_, Value>("scene.apply", Apply { commands })
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "view": view.name,
        "started": started,
        "nodes": view.scene.len(),
    }))
}

/// Removes the nodes of a view.
pub fn unload(name: &str) -> Result<Value, String> {
    check_name(name)?;
    let mut client = connect(name)?;
    let commands = removals(&mut client, name)?;
    let removed = commands.len();
    if !commands.is_empty() {
        client
            .call::<_, Value>("scene.apply", Apply { commands })
            .map_err(|e| e.to_string())?;
    }
    Ok(serde_json::json!({"view": name, "removed": removed}))
}

/// Writes the current nodes of a view to its file, with the plugins of
/// the file that was there. Returns the path.
pub fn save(name: &str, file: Option<&str>) -> Result<Value, String> {
    check_name(name)?;
    let path = match file {
        Some(f) => PathBuf::from(f),
        None => dirs("views")[0].join(format!("{name}.yaml")),
    };
    let plugins = find("views", name)
        .ok()
        .and_then(|p| read_yaml::<View>(&p).ok())
        .map(|v| v.plugins)
        .unwrap_or_default();
    let mut client = connect(name)?;
    let nodes = owned(&mut client, name)?;
    let view = View {
        name: name.into(),
        plugins,
        scene: in_order(&nodes)
            .into_iter()
            .map(view_node)
            .collect::<Result<_, _>>()?,
    };
    let text = to_yaml(&view)?;
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(serde_json::json!({"view": name, "file": path, "nodes": view.scene.len()}))
}

/// Checks what the core cannot: unique ids, and no surfaces.
fn check(view: &View) -> Result<(), String> {
    check_name(&view.name)?;
    for plugin in &view.plugins {
        check_name(&plugin.name)?;
    }
    let mut ids = HashSet::new();
    for node in &view.scene {
        if !ids.insert(&node.id) {
            return Err(format!("the node {} appears twice", node.id));
        }
        if node.kind == NodeKind::Surface {
            return Err(format!(
                "{} is a surface; views do not create surfaces",
                node.id
            ));
        }
    }
    Ok(())
}

/// Starts the plugins that do not run, and configures all of them. Returns
/// the names of the started plugins.
fn start_plugins(client: &mut Client, plugins: &[PluginUse]) -> Result<Vec<String>, String> {
    let manifests: Vec<Manifest> = plugins
        .iter()
        .map(|p| {
            let manifest: Manifest = read_yaml(&find("plugins", &p.name)?)?;
            if manifest.name == p.name {
                Ok(manifest)
            } else {
                Err(format!(
                    "the manifest of {} names {}",
                    p.name, manifest.name
                ))
            }
        })
        .collect::<Result<_, String>>()?;
    let running = clients(client)?;
    let mut started = Vec::new();
    for manifest in &manifests {
        if !running.contains_key(&manifest.name) {
            let argv = ["sh".into(), "-c".into(), manifest.exec.clone()];
            let answer = crate::run_program(client, &argv, Some(&manifest.name))
                .map_err(|e| format!("{}: {e}", manifest.name))?;
            // A unit that runs already answers no token: nothing started.
            if !answer["token"].is_null() {
                started.push(manifest.name.clone());
            }
        }
    }
    for (plugin, manifest) in plugins.iter().zip(&manifests) {
        let method = manifest.configure.as_deref();
        wait_for(client, &manifest.name, method)?;
        if plugin.config.is_null() {
            continue;
        }
        let method = method.ok_or_else(|| format!("{} takes no config", plugin.name))?;
        let call = MethodCall {
            client: plugin.name.clone(),
            method: method.into(),
            params: plugin.config.clone(),
        };
        client
            .call::<_, Value>("methods.call", call)
            .map_err(|e| format!("{}.{method}: {e}", plugin.name))?;
    }
    Ok(started)
}

/// The connected clients and their methods.
fn clients(client: &mut Client) -> Result<HashMap<String, Vec<String>>, String> {
    #[derive(Deserialize)]
    struct Entry {
        name: String,
        methods: Vec<String>,
    }
    let list: Vec<Entry> = client
        .call("clients.list", Value::Null)
        .map_err(|e| e.to_string())?;
    Ok(list.into_iter().map(|e| (e.name, e.methods)).collect())
}

/// Waits until a plugin connected, and exported a method if it has one.
fn wait_for(client: &mut Client, name: &str, method: Option<&str>) -> Result<(), String> {
    let deadline = Instant::now() + START_TIMEOUT;
    loop {
        let ready = clients(client)?
            .get(name)
            .is_some_and(|methods| method.is_none_or(|m| methods.iter().any(|x| x == m)));
        if ready {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!("{name} did not connect within {START_TIMEOUT:?}"));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// The nodes that a view owns.
fn owned(client: &mut Client, view: &str) -> Result<Vec<Node>, String> {
    let query = Query::new(format!("owner:{}", owner(view)));
    client.call("scene.query", query).map_err(|e| e.to_string())
}

/// The removals of the nodes of a view: the nodes whose parent the view
/// does not own; their descendants go with them.
fn removals(client: &mut Client, view: &str) -> Result<Vec<Command>, String> {
    let all: Vec<Node> = client
        .call("scene.query", Query::new("*"))
        .map_err(|e| e.to_string())?;
    Ok(top_owned(&all, &owner(view))
        .into_iter()
        .map(|id| Command::Remove { id, to: None })
        .collect())
}

/// The nodes of an owner without an ancestor of the same owner, also
/// through nodes of others between them.
fn top_owned(all: &[Node], owner: &str) -> Vec<String> {
    let by_id: HashMap<&str, &Node> = all.iter().map(|n| (n.id.as_str(), n)).collect();
    let mine = |n: &Node| n.owner.as_deref() == Some(owner);
    all.iter()
        .filter(|n| mine(n))
        .filter(|n| {
            let mut parent = n.parent.as_deref().and_then(|p| by_id.get(p));
            while let Some(p) = parent {
                if mine(p) {
                    return false;
                }
                parent = p.parent.as_deref().and_then(|q| by_id.get(q));
            }
            true
        })
        .map(|n| n.id.clone())
        .collect()
}

/// The nodes with each parent before its children, and siblings bottom
/// first, so that creating them in this order keeps their stacking.
fn in_order(nodes: &[Node]) -> Vec<&Node> {
    let by_id: HashMap<&str, &Node> = nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    let mut queue: VecDeque<&Node> = nodes
        .iter()
        .filter(|n| n.parent.as_deref().is_none_or(|p| !by_id.contains_key(p)))
        .collect();
    let mut out = Vec::new();
    while let Some(node) = queue.pop_front() {
        out.push(node);
        queue.extend(
            node.children
                .iter()
                .filter_map(|c| by_id.get(c.as_str()).copied()),
        );
    }
    out
}

/// A node as a view stores it: the properties that differ from the
/// defaults.
fn view_node(node: &Node) -> Result<ViewNode, String> {
    let props = serde_json::to_value(&node.props).map_err(|e| e.to_string())?;
    let defaults = serde_json::to_value(Props::default()).map_err(|e| e.to_string())?;
    let changed: serde_json::Map<String, Value> = props
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(key, value)| defaults.get(key.as_str()) != Some(value))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let props = serde_json::from_value(Value::Object(changed)).map_err(|e| e.to_string())?;
    Ok(ViewNode {
        id: node.id.clone(),
        kind: node.kind,
        parent: node.parent.clone().unwrap_or_else(root),
        props,
    })
}

/// The YAML text of a view, short where it can be: a transform that only
/// moves shows as `translate`, and lists of numbers fit on one line.
fn to_yaml(view: &View) -> Result<String, String> {
    let mut value = serde_json::to_value(view).map_err(|e| e.to_string())?;
    for node in value["scene"].as_array_mut().into_iter().flatten() {
        if let Some(transform) = node["props"].get_mut("transform") {
            shorten(transform);
        }
    }
    serde_saphyr::to_string(&Yaml::from(value)).map_err(|e| e.to_string())
}

/// Turns a matrix that only moves into `{translate: [x, y, z]}`.
fn shorten(transform: &mut Value) {
    let Some(m) = transform["matrix"].as_array() else {
        return;
    };
    let identity = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    #[allow(clippy::float_cmp)] // exact values of an identity
    let moves = m.len() == 16
        && m[..12]
            .iter()
            .zip(identity)
            .all(|(v, i)| v.as_f64() == Some(i))
        && m[15].as_f64() == Some(1.0);
    if moves {
        let translate = if m[14].as_f64() == Some(0.0) {
            vec![m[12].clone(), m[13].clone()]
        } else {
            vec![m[12].clone(), m[13].clone(), m[14].clone()]
        };
        *transform = serde_json::json!({ "translate": translate });
    }
}

/// A JSON value for YAML, with lists of plain values in flow style.
#[derive(Serialize)]
#[serde(untagged)]
enum Yaml {
    Plain(Value),
    Flow(serde_saphyr::FlowSeq<Vec<Value>>),
    List(Vec<Yaml>),
    Nested(std::collections::BTreeMap<String, Yaml>),
}

impl From<Value> for Yaml {
    fn from(value: Value) -> Self {
        match value {
            Value::Array(items) if items.iter().all(|v| !v.is_array() && !v.is_object()) => {
                Self::Flow(serde_saphyr::FlowSeq(items))
            }
            Value::Array(items) => Self::List(items.into_iter().map(Self::from).collect()),
            Value::Object(map) => {
                Self::Nested(map.into_iter().map(|(k, v)| (k, Self::from(v))).collect())
            }
            plain => Self::Plain(plain),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_view_reads_strictly() {
        let view: View = parse_yaml(
            "name: work\nplugins:\n- name: tiling\n  config: {layout: rows}\nscene:\n- id: bar\n  kind: rect\n  props: {size: [10, 2], transform: {translate: [0, 5]}}\n",
        )
        .unwrap();
        assert_eq!(view.plugins[0].config["layout"], "rows");
        assert_eq!(view.scene[0].parent, ROOT);
        assert_eq!(view.scene[0].props.size, Some([10.0, 2.0]));
        assert!(
            parse_yaml::<View>("name: work\ncolor: red\n").is_err(),
            "an unknown key"
        );
        assert!(
            parse_yaml::<View>("name: a\nname: b\n").is_err(),
            "a duplicate key"
        );
    }

    #[test]
    fn no_stays_a_string() {
        let view: View =
            parse_yaml("name: no\nplugins:\n- name: tiling\n  config: {country: no}\n").unwrap();
        assert_eq!(view.name, "no");
        assert_eq!(view.plugins[0].config["country"], "no");
    }

    #[test]
    fn a_saved_view_reads_back() {
        let view = View {
            name: "work".into(),
            plugins: vec![PluginUse {
                name: "tiling".into(),
                config: serde_json::json!({"layout": "columns", "gap": 8}),
            }],
            scene: vec![ViewNode {
                id: "bar".into(),
                kind: NodeKind::Rect,
                parent: ROOT.into(),
                props: PropsUpdate {
                    size: Some([10.0, 2.0]),
                    color: Some([1.0, 0.0, 0.0, 1.0]),
                    ..PropsUpdate::default()
                },
            }],
        };
        let text = to_yaml(&view).unwrap();
        assert_eq!(parse_yaml::<View>(&text).unwrap(), view, "{text}");
        assert!(text.contains("size: [10.0, 2.0]"), "{text}");
    }

    #[test]
    fn names_stay_in_their_directory() {
        assert!(check_name("work-2.night").is_ok());
        for bad in [
            "",
            "../x",
            ".hidden",
            "a/b",
            "/etc/x",
            "a b",
            "Work",
            "9lives",
            "evil.yaml",
        ] {
            assert!(check_name(bad).is_err(), "{bad:?}");
        }
        assert!(check_name(&"v".repeat(59)).is_ok());
        assert!(
            check_name(&"v".repeat(60)).is_err(),
            "the prefix of the view counts into the 64"
        );
        assert!(
            find("views", "../../etc/passwd").is_ok(),
            "a path stays a path"
        );
        assert!(find("views", "..").is_err());
    }

    #[test]
    fn removals_skip_nodes_under_an_owned_ancestor() {
        let node = |id: &str, owner: &str, parent: &str| Node {
            id: id.into(),
            kind: NodeKind::Group,
            owner: Some(owner.into()),
            parent: Some(parent.into()),
            children: Vec::new(),
            props: Props::default(),
            surface: None,
            to: None,
        };
        let all = vec![
            node("a", "view-x", "root"),
            node("b", "other", "a"),
            node("c", "view-x", "b"),
            node("d", "view-x", "root"),
        ];
        assert_eq!(top_owned(&all, "view-x"), vec!["a", "d"]);
    }

    #[test]
    fn a_plain_move_saves_as_translate() {
        let mut moved =
            serde_json::to_value(chora_scene::Transform::translate(5.0, 6.0, 0.0)).unwrap();
        shorten(&mut moved);
        assert_eq!(moved, serde_json::json!({"translate": [5.0, 6.0]}));
        let mut turned = serde_json::to_value(chora_scene::Transform::rotate_z(30.0)).unwrap();
        shorten(&mut turned);
        assert!(turned.get("matrix").is_some());
    }

    #[test]
    fn a_manifest_names_its_command() {
        let manifest: Manifest =
            parse_yaml("name: tiling\nexec: chora-tiling --config x.yaml\nconfigure: set_layout\n")
                .unwrap();
        assert_eq!(manifest.configure.as_deref(), Some("set_layout"));
        assert!(parse_yaml::<Manifest>("name: tiling\n").is_err(), "no exec");
    }
}
