//! Selectors find nodes by id, attribute or meta data.

use std::str::FromStr;

use crate::{Node, NodeKind};

/// A list of conditions, separated by spaces. A node matches when it meets
/// all of them. `*` matches every node.
///
/// | Condition | Matches |
/// |-----------|---------|
/// | `id:work` | the node with this id |
/// | `kind:surface` | nodes of this kind |
/// | `owner:tiling` | nodes of this owner |
/// | `app_id:foot` | surfaces with this application id |
/// | `title:Inbox` | surfaces with exactly this title |
/// | `title~:Inbox` | surfaces whose title contains the text |
/// | `meta:role=editor` | nodes with this meta value |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selector(Vec<Condition>);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Condition {
    Id(String),
    Kind(NodeKind),
    Owner(String),
    AppId(String),
    Title(String),
    TitleContains(String),
    Meta(String, String),
}

/// A selector with a syntax error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid selector condition {0:?}")]
pub struct SelectorError(String);

impl FromStr for Selector {
    type Err = SelectorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_whitespace()
            .filter(|c| *c != "*")
            .map(parse_condition)
            .collect::<Result<_, _>>()
            .map(Self)
    }
}

fn parse_condition(c: &str) -> Result<Condition, SelectorError> {
    let err = || SelectorError(c.into());
    let (key, value) = c.split_once(':').ok_or_else(err)?;
    let value = value.to_owned();
    Ok(match key {
        "id" => Condition::Id(value),
        "kind" => Condition::Kind(serde_json::from_value(value.into()).map_err(|_| err())?),
        "owner" => Condition::Owner(value),
        "app_id" => Condition::AppId(value),
        "title" => Condition::Title(value),
        "title~" => Condition::TitleContains(value),
        "meta" => {
            let (k, v) = value.split_once('=').ok_or_else(err)?;
            Condition::Meta(k.into(), v.into())
        }
        _ => return Err(err()),
    })
}

impl Selector {
    /// Whether the node meets all conditions.
    #[must_use]
    pub fn matches(&self, node: &Node) -> bool {
        self.0.iter().all(|c| c.matches(node))
    }
}

impl Condition {
    fn matches(&self, node: &Node) -> bool {
        let info = node.surface.as_ref();
        let title = info.and_then(|i| i.title.as_deref());
        match self {
            Self::Id(id) => node.id == *id,
            Self::Kind(kind) => node.kind == *kind,
            Self::Owner(owner) => node.owner.as_deref() == Some(owner),
            Self::AppId(app_id) => info.and_then(|i| i.app_id.as_deref()) == Some(app_id),
            Self::Title(t) => title == Some(t),
            Self::TitleContains(t) => title.is_some_and(|title| title.contains(t.as_str())),
            Self::Meta(k, v) => node.props.meta.get(k) == Some(v),
        }
    }
}
