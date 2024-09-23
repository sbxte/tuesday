use core::str;
use std::cell::RefCell;
use std::collections::HashMap;

use anyhow::{bail, Result};
use clap::ValueEnum;
use thiserror::Error;
use yaml_rust2::{Yaml, YamlLoader};

use crate::graph::Graph;

use super::{Doc, VERSION};

#[derive(Clone, Debug, Error)]
enum ParseError {
    #[error("No parseable document version is implemented for this document version!")]
    Unimplemented,
}

/// Parse (possibly) old version documents
pub fn compat_parse(input: &[u8]) -> Result<Doc> {
    // String form
    if let Ok(input) = str::from_utf8(input) {
        if let Ok(docs) = YamlLoader::load_from_str(input) {
            return parse_yaml(&docs[0]);
        }
    }
    Err(ParseError::Unimplemented.into())
}

/// Manually parse yaml instead of using serde_derive
pub fn parse_yaml(doc: &Yaml) -> Result<Doc> {
    // Version mismatch
    let doc_ver = doc["version"].as_i64();
    if doc_ver.is_none() {
        bail!("Yaml parse error: Version field does not exist");
    } else if let Some(version) = doc_ver
        && version != VERSION as i64
    {
        bail!("Yaml parse error: Version mismatch");
    }

    let graph_doc = &doc["graph"];

    // Roots, archived, and dates
    let roots = graph_doc["roots"]
        .as_vec()
        .unwrap_or(&vec![])
        .iter()
        .map(|i| i.as_i64().expect("Root index must be an integer") as usize)
        .collect::<Vec<_>>();
    let archived = graph_doc["archived"]
        .as_vec()
        .unwrap_or(&vec![])
        .iter()
        .map(|i| i.as_i64().expect("Archived index must be an integer") as usize)
        .collect::<Vec<_>>();
    let dates = graph_doc["dates"]
        .as_hash()
        .unwrap_or(&yaml_rust2::yaml::Hash::new())
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().expect("Date key must be a string").to_string(),
                v.as_i64().expect("Date node index must be an integer") as usize,
            )
        })
        .collect::<HashMap<_, _>>();
    let mut aliases = graph_doc["aliases"]
        .as_hash()
        .unwrap_or(&yaml_rust2::yaml::Hash::new())
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
    for node_doc in graph_doc["nodes"].as_vec().unwrap_or(&vec![]) {
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
        for parent_doc in node_doc["parents"].as_vec().unwrap_or(&vec![]) {
            parents.push(
                parent_doc
                    .as_i64()
                    .expect("Parent index must be an integer") as usize,
            );
        }
        let mut children = vec![];
        for child_doc in node_doc["children"].as_vec().unwrap_or(&vec![]) {
            children.push(child_doc.as_i64().expect("Parent index must be an integer") as usize);
        }

        // Add local node alias to root doc aliases if not already added
        let alias = node_doc["alias"].as_str();
        if let Some(ref alias) = alias {
            aliases.insert(alias.to_string(), index);
        }

        nodes.push(Some(RefCell::new(crate::graph::Node {
            message: node_doc["message"].as_str().unwrap_or("").to_string(),
            r#type: node_doc["type"]
                .as_str()
                .map_or(crate::graph::NodeType::default(), |n| {
                    crate::graph::NodeType::from_str(n, true)
                        .unwrap_or(crate::graph::NodeType::default())
                }),
            state: node_doc["state"]
                .as_str()
                .map_or(crate::graph::NodeState::default(), |n| {
                    crate::graph::NodeState::from_str(n, true)
                        .unwrap_or(crate::graph::NodeState::default())
                }),
            archived: node_doc["archived"].as_bool().unwrap_or(false),
            index,
            alias: alias.map(|s| s.to_string()),
            parents,
            children,
        })));
    }

    // Remove aliases pointing to invalid nodes
    aliases.retain(|_, v| nodes[*v].is_some());

    // Fix any node aliases that may be desynchronized with the root doc's aliases
    for (k, v) in aliases.iter() {
        nodes[*v].as_ref().unwrap().borrow_mut().alias = Some(k.clone());
    }

    // Unify everything
    let config = Doc {
        version: doc["version"].as_i64().expect("Version should be integer") as u32,
        graph: Graph {
            nodes,
            roots,
            archived,
            dates,
            aliases,
        },
    };
    Ok(config)
}
