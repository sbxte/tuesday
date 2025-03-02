use core::str;
use std::cell::RefCell;
use std::collections::HashMap;

use serde_yaml_ng::{Mapping, Value};

use crate::graph::node::{Node, NodeMetadata};
use crate::graph::Graph;

use super::{errors::ErrorType, Doc, DocResult, VERSION};

/// Parse (possibly) old version documents
pub fn compat_parse(input: &[u8]) -> DocResult<Doc> {
    // String form
    if let Ok(input) = str::from_utf8(input) {
        return match serde_yaml_ng::from_str::<Value>(input) {
            Ok(docs) => parse_yaml(&docs),
            Err(err) => Err(ErrorType::YAMLError(err)),
        };
    }
    Err(super::errors::ErrorType::ParseError(
        "Unimplemented".to_string(),
    ))
}

/// Manually parse yaml instead of using serde_derive
pub fn parse_yaml(doc: &Value) -> DocResult<Doc> {
    // Version mismatch
    let doc_ver = doc["version"].as_i64();
    if doc_ver.is_none() {
        return Err(ErrorType::ParseError(
            "Version field not found!".to_string(),
        ));
    } else if let Some(version) = doc_ver {
        if version != VERSION as i64 {
            return match parse_old_yaml(doc, version) {
                Ok(result) => Ok(result),
                Err(err) => Err(ErrorType::ParseError(format!(
                    "Compatibility parsers failed parsing old version: {}",
                    err
                ))),
            };
        }
    }

    let graph_doc = &doc["graph"];

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
    let dates = graph_doc["dates"]
        .as_mapping()
        .unwrap_or(&Mapping::new())
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().expect("Date key must be a string").to_string(),
                v.as_i64().expect("Date node index must be an integer") as usize,
            )
        })
        .collect::<HashMap<_, _>>();
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
        version: doc["version"].as_i64().expect("Version should be integer") as u32,
        graph: Graph {
            nodes,
            roots,
            archived,
            dates,
            aliases,
        },
    };
    Ok(result_doc)
}

fn parse_old_yaml(doc: &Value, doc_ver: i64) -> DocResult<Doc> {
    match doc_ver {
        4 => old_yaml::v4(doc),
        v => Err(ErrorType::ParseError(format!(
            "No available parsers to parse this document version: {}",
            v
        ))),
    }
}

mod old_yaml {
    use crate::graph::node::task::{TaskData, TaskState};
    use crate::graph::node::NodeType;

    use super::*;

    pub fn v4(doc: &Value) -> DocResult<Doc> {
        let graph_doc = &doc["graph"];

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
        let dates = graph_doc["dates"]
            .as_mapping()
            .unwrap_or(&Mapping::new())
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().expect("Date key must be a string").to_string(),
                    v.as_i64().expect("Date node index must be an integer") as usize,
                )
            })
            .collect::<HashMap<_, _>>();
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

            // Index
            let index = node_doc["index"]
                .as_i64()
                .expect("Node index must be an integer") as usize;

            // Update parent and children
            let mut parents = vec![];
            for parent_doc in node_doc["parents"].as_sequence().unwrap_or(&vec![]) {
                parents.push(
                    parent_doc
                        .as_i64()
                        .expect("Parent index must be an integer") as usize,
                );
            }
            let mut children = vec![];
            for child_doc in node_doc["children"].as_sequence().unwrap_or(&vec![]) {
                children
                    .push(child_doc.as_i64().expect("Parent index must be an integer") as usize);
            }

            // Add local node alias to root doc aliases if not already added
            let alias = node_doc["alias"].as_str();
            if let Some(ref alias) = alias {
                aliases.insert(alias.to_string(), index);
            }

            // Node data
            let node_type = node_doc["type"].as_str().ok_or(ErrorType::ParseError(
                "Node type must be string".to_string(),
            ))?;
            let node_state = node_doc["state"].as_str().ok_or(ErrorType::ParseError(
                "Node state must be string".to_string(),
            ))?;
            let data = match node_type {
                "Normal" => NodeType::Task(TaskData {
                    state: match node_state {
                        "Partial" => TaskState::Partial,
                        "Done" => TaskState::Done,
                        _ => TaskState::None,
                    },
                }),
                "Date" => NodeType::Date(Default::default()),
                "Pseudo" => NodeType::Pseudo,
                _ => Default::default(),
            };

            nodes.push(Some(RefCell::new(Node {
                title: node_doc["message"].as_str().unwrap_or("").to_string(),
                data,
                metadata: NodeMetadata {
                    archived: node_doc["archived"].as_bool().unwrap_or(false),
                    index,
                    alias: alias.map(|s| s.to_string()),
                    parents,
                    children,
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
            version: doc["version"].as_i64().expect("Version should be integer") as u32,
            graph: Graph {
                nodes,
                roots,
                archived,
                dates,
                aliases,
            },
        };
        Ok(result_doc)
    }
}
