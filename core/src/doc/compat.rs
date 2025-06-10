use core::str;
use std::cell::RefCell;
use std::collections::HashMap;

use chrono::Utc;
use serde_yaml_ng::{Mapping, Value};

use crate::graph::node::{Node, NodeMetadata};
use crate::graph::Graph;
use crate::time::timeline::Timeline;

use super::{errors::ErrorType, Doc, DocResult, VERSION};

/// Parse (possibly) old version documents
pub fn compat_parse(input: &[u8]) -> DocResult<Doc> {
    // String form
    if let Ok(input) = str::from_utf8(input) {
        return match serde_yaml_ng::from_str::<Value>(input) {
            Ok(docs) => parse_yaml(docs),
            Err(err) => Err(ErrorType::YAMLError(err)),
        };
    }
    Err(super::errors::ErrorType::ParseError(
        "Unimplemented".to_string(),
    ))
}

/// Manually parse yaml instead of using serde_derive
pub fn parse_yaml(doc: Value) -> DocResult<Doc> {
    let mut doc_use = doc;
    // Version mismatch
    let doc_ver = doc_use["version"].as_u64();
    if doc_ver.is_none() {
        return Err(ErrorType::ParseError(
            "Version field not found!".to_string(),
        ));
    } else if let Some(version) = doc_ver {
        if version != VERSION as u64 {
            doc_use = match parse_old_yaml(&doc_use) {
                Ok(result) => result,
                Err(err) => {
                    return Err(ErrorType::ParseError(format!(
                        "Compatibility parsers failed parsing old version: {err}"
                    )))
                }
            };
        }
    }

    let graph_doc = &doc_use["graph"];

    // For use in missing DateTimes
    let now = Utc::now();

    // Roots, archived, and dates
    let roots = graph_doc["roots"]
        .as_sequence()
        .unwrap_or(&vec![])
        .iter()
        .map(|i| i.as_i64().expect("Root index must be an integer") as usize)
        .collect::<Vec<_>>();
    let archived = graph_doc["archived"]
        .as_sequence()
        .unwrap_or(&vec![])
        .iter()
        .map(|i| i.as_i64().expect("Archived index must be an integer") as usize)
        .collect::<Vec<_>>();
    let mut aliases = graph_doc["aliases"]
        .as_mapping()
        .unwrap_or(&Mapping::new())
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().expect("Alias key must be a string").to_string(),
                v.as_i64().expect("Alias node index must be an integer") as usize,
            )
        })
        .collect::<HashMap<_, _>>();

    // Parse nodes
    // Provide default values if any are missing
    // ~ Maps are funky, functional programming go brrrr
    let mut nodes = vec![];
    for node_doc in graph_doc["nodes"].as_sequence().unwrap_or(&vec![]) {
        if node_doc.is_null() {
            nodes.push(None);
            continue;
        }

        let metadata = node_doc.get("metadata").ok_or(ErrorType::ParseError(
            "No metadata found in node".to_string(),
        ))?;

        // Index
        let index = metadata["index"].as_i64().ok_or(ErrorType::ParseError(
            "Node index must be an integer".to_string(),
        ))? as usize;

        // Update parent and children
        let mut parents = vec![];
        for parent_doc in metadata["parents"].as_sequence().unwrap_or(&vec![]) {
            parents.push(parent_doc.as_i64().ok_or(ErrorType::ParseError(
                "Parent index must be an integer".to_string(),
            ))? as usize);
        }
        let mut children = vec![];
        for child_doc in metadata["children"].as_sequence().unwrap_or(&vec![]) {
            children.push(child_doc.as_i64().ok_or(ErrorType::ParseError(
                "Child index must be an integer".to_string(),
            ))? as usize);
        }

        // Add local node alias to root doc aliases if not already added
        let alias = metadata["alias"].as_str();
        if let Some(ref alias) = alias {
            aliases.insert(alias.to_string(), index);
        }

        nodes.push(Some(RefCell::new(Node {
            title: node_doc["title"].as_str().unwrap_or("No Title").to_string(),
            data: serde_yaml_ng::from_value(node_doc["data"].clone())?,
            metadata: NodeMetadata {
                archived: metadata["archived"].as_bool().unwrap_or(false),
                index,
                alias: alias.map(|s| s.to_string()),
                parents,
                children,
                created_at: serde_yaml_ng::from_value(metadata["created_at"].to_owned())
                    .unwrap_or(now),
                events: vec![], // PRECOMMIT: Properly implement events parsing
            },
        })));
    }

    // Remove aliases pointing to invalid nodes
    aliases.retain(|_, v| nodes[*v].is_some());

    // Fix any node aliases that may be desynchronized with the root doc's aliases
    for (k, v) in aliases.iter() {
        nodes[*v].as_ref().unwrap().borrow_mut().metadata.alias = Some(k.clone());
    }

    // Unify everything
    let result_doc = Doc {
        version: doc_use["version"]
            .as_i64()
            .expect("Version should be integer") as u32,
        graph: Graph {
            nodes,
            roots,
            archived,
            aliases,
        },
        // TODO: Properly implement this
        timeline: Timeline::new(),
    };
    Ok(result_doc)
}

fn parse_old_yaml(doc: &Value) -> DocResult<Value> {
    let mut doc_modified: Value = doc.clone();
    loop {
        let current_ver = doc_modified["version"]
            .as_u64()
            .ok_or(ErrorType::ParseError(
                "Failed to parse version field from save file".into(),
            ))?;
        if current_ver < VERSION as u64 {
            match current_ver {
                4 => doc_modified = old_yaml::v4_to_v5(doc)?,
                5 => doc_modified = old_yaml::v5_to_v6(doc)?,
                6 => doc_modified = old_yaml::v6_to_v7(doc)?,
                _ => {
                    return Err(ErrorType::ParseError(format!(
                        "Oops, no available parsers to parse this document version: {current_ver}"
                    )))
                }
            }
        } else {
            break;
        }
    }
    Ok(doc_modified)
}

/// Docs-parser for older version of the save file. This is done incrementally (e.g v4 -> v5, v5 ->
/// v6, etc up until the current version).
mod old_yaml {
    #[derive(PartialEq, Eq, Deserialize, Debug)]
    struct Empty;

    #[derive(PartialEq, Eq, Deserialize, Debug)]
    enum V5nodeType {
        Date(Empty),
        Task(TaskData),
        Pseudo,
    }

    use chrono::{NaiveDate, NaiveTime, TimeDelta};
    use serde::Deserialize;
    use serde_yaml_ng::value::{Tag, TaggedValue};
    use serde_yaml_ng::Mapping;

    use crate::graph::node::task::TaskData;

    use super::*;

    /// The v4 to v5 update introduced some major breaking structure changes:
    /// - `message` is renamed to `title`.
    /// - `type` & `state` are merged into the `data` field which is now tagged based on the node type.
    /// - `archived`, `index`, `alias`, `parents`, and `children` fields are now moved under `metadata`.
    pub fn v4_to_v5(doc: &Value) -> DocResult<Value> {
        let mut cloned_doc = doc.clone();
        let version = &mut cloned_doc["version"];
        *version = Value::Number(5.into());

        let graph_doc = &mut cloned_doc["graph"];
        let nodes = graph_doc["nodes"].as_sequence_mut().unwrap();

        for node_doc in nodes {
            if node_doc.is_null() {
                continue;
            }

            if let Value::Mapping(node) = node_doc {
                node.insert("title".into(), node["message"].clone());
                node.remove("message");

                let mut metadata = Mapping::new();
                let archived = node["archived"].clone();
                let index = node["index"].clone();
                let alias = node["alias"].clone();
                let children = node["children"].clone();
                let parents = node["parents"].clone();
                metadata.insert("archived".into(), archived);
                metadata.insert("index".into(), index);
                metadata.insert("alias".into(), alias);
                metadata.insert("children".into(), children);
                metadata.insert("parents".into(), parents);
                node.insert("metadata".into(), Value::Mapping(metadata));

                node.remove("archived");
                node.remove("index");
                node.remove("alias");
                node.remove("children");
                node.remove("parents");

                let data = match node["type"].as_str() {
                    Some("Normal") => {
                        let mut map = Mapping::new();
                        map.insert("state".into(), node["state"].clone());
                        TaggedValue {
                            value: Value::Mapping(map),
                            tag: Tag::new("Task"),
                        }
                    }
                    Some("Date") => TaggedValue {
                        value: Value::Mapping(Mapping::new()),
                        tag: Tag::new("Date"),
                    },
                    Some("Pseudo") => TaggedValue {
                        value: Value::Null,
                        tag: Tag::new("Pseudo"),
                    },
                    _ => {
                        return Err(ErrorType::ParseError(
                            "Failed to determine node's type".to_string(),
                        ))
                    }
                };

                node.insert("data".into(), Value::Tagged(data.into()));
                node.remove("type");
                node.remove("state");
            }
        }

        Ok(cloned_doc)
    }

    /// The change from v5 to v6 only introduced a slight change: Date nodes have its actual date
    /// now stored inside the data field and allows for a different message to be used.
    pub fn v5_to_v6(doc: &Value) -> DocResult<Value> {
        let mut cloned_doc = doc.clone();
        let version = &mut cloned_doc["version"];
        *version = Value::Number(6.into());

        let graph_doc = &mut cloned_doc["graph"];
        let nodes = graph_doc["nodes"].as_sequence_mut().unwrap();

        for node_doc in nodes {
            if node_doc.is_null() {
                continue;
            }
            if let Value::Mapping(node) = node_doc {
                // TODO: if there's a way to parse tagged value without giving it a struct, use that.
                if let Value::Tagged(val) = &node["data"].clone() {
                    if val.tag == "!Date" {
                        let mut new_map = Mapping::new();
                        new_map.insert(
                            "date".into(),
                            Value::String(format!(
                                "{}",
                                NaiveDate::parse_from_str(
                                    node["title"].as_str().unwrap(),
                                    "%Y-%m-%d"
                                )?
                            )),
                        );
                        node.remove("data");
                        node.insert(
                            "data".into(),
                            Value::Tagged(Box::new(TaggedValue {
                                tag: Tag::new("Date"),
                                value: Value::Mapping(new_map),
                            })),
                        );
                    }
                }
            }
        }
        Ok(cloned_doc)
    }

    /// In v7 date nodes cease to exist, being replaced instead with the
    /// [Timeline](crate::time::timeline::Timeline)'s [Events][crate::time::event::Event] stored in [Doc]
    pub fn v6_to_v7(doc: &Value) -> DocResult<Value> {
        let mut cloned_doc = doc.clone();
        let version = &mut cloned_doc["version"];
        *version = Value::Number(6.into());

        let graph_doc = &mut cloned_doc["graph"];
        let nodes = graph_doc["nodes"].as_sequence_mut().unwrap();

        let mut timeline = Timeline::new();
        let mut event_map = vec![];

        for node_doc in nodes.iter() {
            if node_doc.is_null() {
                continue;
            }
            if let Value::Mapping(node) = node_doc {
                if let Value::Tagged(val) = &node["data"].clone() {
                    if val.tag != "Date" {
                        continue;
                    }
                    let date = NaiveDate::parse_from_str(
                        val.value.as_mapping().unwrap()["date"].as_str().unwrap(),
                        "%Y-%m-%d",
                    )
                    .unwrap();

                    if let Value::Mapping(metadata) = &node["metadata"] {
                        if let Value::Sequence(seq) = &metadata["children"] {
                            for child_id in seq.iter().map(|e| e.as_u64().unwrap() as usize) {
                                let start = date.and_time(NaiveTime::MIN).and_utc();
                                let end = start + TimeDelta::days(1);
                                let event_id = timeline.create_event(
                                    start..end,
                                    format!(
                                        "{} event",
                                        nodes[child_id]["title"].as_str().unwrap_or("")
                                    ),
                                    metadata["index"].as_u64().unwrap() as usize,
                                );
                                event_map.push((child_id, event_id));
                            }
                        }
                    }
                }
            }
        }
        use serde_yaml_ng::mapping::Entry;
        for (child_id, event_id) in event_map {
            match nodes[child_id]
                .as_mapping_mut()
                .unwrap()
                .entry("events".into())
            {
                Entry::Occupied(mut occupied_entry) => {
                    occupied_entry
                        .get_mut()
                        .as_sequence_mut()
                        .unwrap()
                        .push(event_id.into());
                }
                Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(vec![event_id].into());
                }
            }
        }
        cloned_doc.as_mapping_mut().unwrap().insert(
            "timeline".into(),
            serde_yaml_ng::to_string(&timeline).unwrap().into(),
        );

        Ok(cloned_doc)
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml_ng::Value;

    use super::old_yaml;

    #[test]
    fn test_v4_v5() {
        let old = serde_yaml_ng::from_str::<Value>(
            "
version: 4
graph:
  nodes:
  - message: root
    type: Normal
    state: None
    archived: false
    index: 0
    alias: null
    parents: []
    children: []
  - message: 2025-01-01
    type: Date
    state: None
    archived: false
    index: 1
    alias: null
    parents: []
    children: []
  - message: pseudo
    type: Pseudo
    state: None
    archived: false
    index: 2
    alias: null
    parents: []
    children: []
  roots:
  - 0
  - 2
  archived: []
  dates:
    2025-01-01: 1
  aliases: {}",
        );

        let new_should_be = serde_yaml_ng::from_str::<Value>(
            "
version: 5
graph:
  nodes:
  - title: root
    data: !Task
      state: None
    metadata:
      archived: false
      index: 0
      alias: null
      children: []
      parents: []
  - title: 2025-01-01
    data: !Date {}
    metadata:
      archived: false
      index: 1
      alias: null
      children: []
      parents: []
  - title: pseudo
    data: !Pseudo
    metadata:
      archived: false
      index: 2
      alias: null
      children: []
      parents: []
  roots:
  - 0
  - 2
  archived: []
  dates:
    2025-01-01: 1
  aliases: {}
",
        );
        let new = old_yaml::v4_to_v5(&old.unwrap()).unwrap();
        assert_eq!(
            serde_yaml_ng::to_string(&new).unwrap(),
            serde_yaml_ng::to_string(&new_should_be.unwrap()).unwrap()
        );
    }

    #[test]
    fn test_v5_v6() {
        let old = serde_yaml_ng::from_str::<Value>(
            "
version: 5
graph:
  nodes:
  - title: root
    data: !Task
      state: None
    metadata:
      archived: false
      index: 0
      alias: null
      children: []
      parents: []
  - title: 2025-01-01
    data: !Date {}
    metadata:
      archived: false
      index: 1
      alias: null
      children: []
      parents: []
  - title: pseudo
    data: !Pseudo
    metadata:
      archived: false
      index: 2
      alias: null
      children: []
      parents: []
  roots:
  - 0
  - 2
  archived: []
  dates:
    2025-01-01: 1
  aliases: {}
",
        );

        let new = old_yaml::v5_to_v6(&old.unwrap()).unwrap();
        let new_should_be = serde_yaml_ng::from_str::<Value>(
            "
version: 6
graph:
  nodes:
  - title: root
    data: !Task
      state: None
    metadata:
      archived: false
      index: 0
      alias: null
      children: []
      parents: []
  - title: 2025-01-01
    metadata:
      archived: false
      index: 1
      alias: null
      children: []
      parents: []
    data: !Date
      date: 2025-01-01
  - title: pseudo
    data: !Pseudo
    metadata:
      archived: false
      index: 2
      alias: null
      children: []
      parents: []
  roots:
  - 0
  - 2
  archived: []
  dates:
    2025-01-01: 1
  aliases: {}
",
        )
        .unwrap();

        assert_eq!(
            serde_yaml_ng::to_string(&new).unwrap(),
            serde_yaml_ng::to_string(&new_should_be).unwrap()
        );
    }
}
