mod display;
mod errors;
use std::path::PathBuf;

use clap::{arg, value_parser, ArgMatches, Command};

use display::CLIDisplay;
use errors::AppError;
use rand::seq::SliceRandom;
use rand::thread_rng;
use tuecore::doc::{self, Doc};
use tuecore::graph::node::NodeState;
use tuecore::graph::{Graph, GraphGetters};

type AppResult<T> = Result<T, AppError>;

fn handle_command(matches: &ArgMatches, graph: &mut Graph) -> AppResult<()> {
    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let message = sub_matches
                .get_one::<String>("message")
                .expect("message required");
            let root = sub_matches.get_flag("root");
            let date = sub_matches.get_flag("date");
            let pseudo = sub_matches.get_flag("pseudo");
            if date && root {
                return Err(AppError::ConflictingArgs("Node cannot be both date node and root node!".to_string()));
            }
            if date {
                if !Graph::is_date(message) {
                    return Err(AppError::MalformedDate(message.to_string()));
                }
                graph.insert_date(message.to_string());
            } else if root {
                graph.insert_root(message.to_string(), pseudo);
            } else {
                let idx = if let Some(i) = sub_matches.get_one::<String>("parent")
                {
                    i
                } else {
                    return Err(AppError::InvalidArg("Parent ID required!".to_string()));
                };
                let parent = graph.get_index(idx)?;
                graph.insert_child(message.to_string(), parent, pseudo)?;
            }
            Ok(())
        }
        Some(("rm", sub_matches)) => {
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let recursive = sub_matches.get_flag("recursive");
                let id = graph.get_index(id)?;
                if recursive {
                    graph.remove_children_recursive(id)?;
                } else {
                    graph.remove(id)?;
                }
            }
            Ok(())
        }
        Some(("link", sub_matches)) => {
            let parent = graph.get_index(
                sub_matches
                    .get_one::<String>("parent")
                    .expect("parent ID required"),
            )?;
            let child = graph.get_index(
                sub_matches
                    .get_one::<String>("child")
                    .expect("child ID required"),
            )?;
            graph.link(parent, child)?;
            Graph::print_link(parent, child, true);
            Ok(())
        }
        Some(("unlink", sub_matches)) => {
            let parent = graph.get_index(
                sub_matches
                    .get_one::<String>("parent")
                    .expect("parent ID required"),
            )?;
            let child = graph.get_index(
                sub_matches
                    .get_one::<String>("child")
                    .expect("child ID required"),
            )?;
            graph.unlink(parent, child)?;
            Graph::print_link(parent, child, false);
            Ok(())
        }
        Some(("mv", sub_matches)) => {
            let nodes = sub_matches
                .get_many::<String>("node")
                .expect("node ID required");
            let parent = graph.get_index(
                sub_matches
                    .get_one::<String>("parent")
                    .expect("parent ID required"),
            )?;

            for node in nodes {
                let node = graph.get_index(node)?;
                graph.clean_parents(node)?;
                graph.link(parent, node)?;
            }
            Ok(())
        }
        Some(("set", sub_matches)) => {
            let id = graph.get_index(sub_matches.get_one::<String>("ID").expect("ID required"))?;
            let state = sub_matches
                .get_one::<NodeState>("state")
                .expect("node state required");
            graph.set_state(id, *state, true)?;
            Ok(())
        }
        Some(("check", sub_matches)) => {
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = graph.get_index(id)?;
                graph.set_state(id, NodeState::Done, true)?;
            }
            Ok(())
        }
        Some(("uncheck", sub_matches)) => {
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = graph.get_index(id)?;
                graph.set_state(id, NodeState::None, true)?;
            }
            Ok(())
        }
        Some(("arc", sub_matches)) => {
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = graph.get_index(id)?;
                graph.set_archived(id, true)?;
            }
            Ok(())
        }
        Some(("unarc", sub_matches)) => {
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = graph.get_index(id)?;
                graph.set_archived(id, false)?;
            }
            Ok(())
        }
        Some(("alias", sub_matches)) => {
            let id = graph.get_index(sub_matches.get_one::<String>("ID").expect("ID required"))?;
            let alias = sub_matches
                .get_one::<String>("alias")
                .expect("alias required");
            graph.set_alias(id, alias.clone())?;
            Ok(())
        }
        Some(("unalias", sub_matches)) => {
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = graph.get_index(id)?;
                graph.unset_alias(id)?;
            }
            Ok(())
        }
        Some(("aliases", _)) => {
            graph.list_aliases()?;
            Ok(())
        }
        Some(("rename", sub_matches)) => {
            let id = graph.get_index(sub_matches.get_one::<String>("ID").expect("ID required"))?;
            let message = sub_matches
                .get_one::<String>("message")
                .expect("ID required");
            graph.rename_node(id, message.to_string())?;
            Ok(())
        }
        Some(("ls", sub_matches)) => {
            let depth = if sub_matches.get_flag("recurse") {
                // Override with infinite depth
                0
            } else {
                *sub_matches
                    .get_one::<u32>("depth")
                    .expect("depth should exist")
            };

            let show_archived = *sub_matches.get_one::<bool>("archived").unwrap();
            match sub_matches.get_one::<String>("ID") {
                None => graph.list_roots(depth, show_archived)?,
                Some(id) => graph.list_children(id.to_string(), depth, show_archived)?,
            }
            Ok(())
        }
        Some(("lsd", _)) => {
            graph.list_dates()?;
            Ok(())
        }
        Some(("lsa", _)) => {
            graph.list_archived()?;
            Ok(())
        }
        Some(("rand", sub_matches)) => {
            let id = sub_matches.get_one::<String>("ID").expect("ID required");
            match graph
                .get_node_children(graph.get_index(id)?)
                .choose(&mut thread_rng())
            {
                None => return Err(AppError::NodeNoChildren),
                Some(children) => {
                    // TODO: Don't use stat
                    graph.print_stats(Some(children.to_string().clone()))?;
                }
            };
            Ok(())
        }
        Some(("stats", sub_matches)) => {
            let id = sub_matches.get_one::<String>("ID");
            graph.print_stats(id.map(|i| i.to_string()))?;
            Ok(())
        }
        Some(("clean", _)) => {
            graph.clean();
            Ok(())
        }
        Some(("export", _)) => {
            println!("{}", doc::export_json(graph)?);
            Ok(())
        }
        Some(("import", _)) => {
            // Import would have finished by this stage so just log received data
            println!(
                "Successfully imported json! {} nodes; {} root nodes; {} aliases",
                graph.node_count(),
                graph.root_count(),
                graph.alias_count()
            );
            Ok(())
        }
        _ => {
            Err(AppError::NoSubcommand)
        }
    }
}

fn cli() -> AppResult<Command> {
    Ok(Command::new("tue")
        .about("Tuesday CLI, todo graph")
        .subcommand_required(false)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .arg(arg!(-V --version "Displays build and version information"))
        .arg(arg!(-l --local <path>)
            .value_parser(value_parser!(String))
            .required(false))
        .arg(arg!(-g --global).required(false))
        .subcommand(Command::new("add")
            .about("Adds a node to the graph")
            .arg(arg!(<message> "This node's message"))
            .arg(arg!(parent: [ID] "Assigns this node to a parent").required(false))
            .arg(arg!(-u --pseudo "Makes this a pseudo node (does not count towards parent completion)")
                .required(false))
            .arg(arg!(-r --root "Makes this a root node")
                .conflicts_with_all(["parent", "date"]))
            .arg(arg!(-d --date "Makes this a date node")
                .conflicts_with_all(["parent", "root"]))
        )
        .subcommand(Command::new("rm")
            .about("Removes nodes from the graph")
            .arg(arg!(<ID>... "Which nodes to remove"))
            .arg(arg!(-r --recursive "Whether to remove child nodes recursively").required(false))
        )
        .subcommand(Command::new("link")
            .about("Creates a parent-child edge connection between 2 nodes")
            .arg(arg!(parent: <ID> "Which node should be the parent in this connection"))
            .arg(arg!(child: <ID> "Which node should be the child in this connection"))
        )
        .subcommand(Command::new("unlink")
            .about("Removes a parent-child edge connection between 2 nodes")
            .arg(arg!(parent: <ID> "Which node should be the parent in this connection"))
            .arg(arg!(child: <ID> "Which node should be the child in this connection"))
        )
        .subcommand(Command::new("mv")
            .about("Unlink nodes from all current parents, then link to a new parent")
            .arg(arg!(node: <ID>... "Which nodes to unlink"))
            .arg(arg!(parent: <ID> "New parent for node"))
        )
        .subcommand(Command::new("set")
            .about("Sets a node's state")
            .arg(arg!(<ID> "Which node to modify"))
            .arg(arg!(<state> "What state to set the node").value_parser(value_parser!(NodeState)))
        )
        .subcommand(Command::new("check")
            .about("Marks nodes as completed")
            .arg(arg!(<ID>... "Which nodes to mark as completed"))
        )
        .subcommand(Command::new("uncheck")
            .about("Marks nodes as incomplete")
            .arg(arg!(<ID>... "Which nodes to mark as incomplete"))
        )
        .subcommand(Command::new("arc")
            .about("Archives (hides) nodes from view")
            .arg(arg!(<ID>... "Which nodes to archive"))
        )
        .subcommand(Command::new("unarc")
            .about("Unarchives (unhides) nodes from view")
            .arg(arg!(<ID>... "Which nodes to archive"))
        )
        .subcommand(Command::new("alias")
            .about("Adds an alias for a node")
            .arg(arg!(<ID> "Which node to alias"))
            .arg(arg!(<alias> "What alias to give this node"))
        )
        .subcommand(Command::new("unalias")
            .about("Removes nodes' alias")
            .long_about("Removes all aliases of nodes")
            .arg(arg!(<ID>... "Which nodes to remove aliases"))
        )
        .subcommand(Command::new("aliases")
            .about("Lists all aliases")
        )
        .subcommand(Command::new("rename")
            .about("Edit a node's message")
            .arg(arg!(<ID> "Which node to edit"))
            .arg(arg!(<message> "What new message to give it"))
        )
        .subcommand(Command::new("ls")
            .about("Lists root nodes or children nodes")
            .arg(arg!([ID] "Which node's children to display"))
            .arg(arg!(-a --archived "Display archived nodes"))
            .arg(arg!(-d --depth <depth> "What depth to recursively display children")
                .default_value("1")
                .value_parser(value_parser!(u32))
            )
            .arg(arg!(-r --recurse "Whether to recursively display at infinite depth"))
        )
        .subcommand(Command::new("lsd")
            .about("Lists all date nodes")
        )
        .subcommand(Command::new("lsa")
            .about("Lists all archived nodes")
        )
        .subcommand(Command::new("rand")
            .about("Picks a random child node")
            .arg(arg!(<ID> "Which parent node to randomly pick a child from"))
        )
        .subcommand(Command::new("stats")
            .about("Displays statistics of a node")
            .arg(arg!([ID] "Which node to display stats"))
        )
        .subcommand(Command::new("clean")
            .about("Compresses and cleans up the graph")
        )
        .subcommand(Command::new("export")
            .about("Exports JSON to stdout")
        )
        .subcommand(Command::new("import")
            .about("Imports JSON from stdin")
        )
    )
}

fn main() -> AppResult<()> {
    let matches = cli()?.get_matches();

    if matches.get_flag("version") {
        println!("Tuesday CLI");
        println!("Version {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let (mut graph, local) = if let Some(("import", _)) = matches.subcommand() {
        let local = match (
            matches.get_one::<String>("local").is_some(),
            matches.get_flag("global"),
        ) {
            // -- global --overrdes --local argument
            (_, true) => false,
            (false, false) => doc::local_exists(PathBuf::from(
                matches
                    .get_one::<String>("local")
                    .expect("--local should provide a path"),
            )),
            (l, _) => l,
        };
        (doc::import_json_stdin()?.graph, local)
    } else {
        match (
            matches.get_one::<String>("local").is_some(),
            matches.get_flag("global"),
        ) {
            // --global overrides --local argument
            (_, true) => (doc::load_global()?, false),
            (true, _) => (
                doc::load_local(PathBuf::from(
                    matches
                        .get_one::<String>("local")
                        .expect("--local should provide a path"),
                ))?,
                true,
            ),
            (false, false) => {
                // Try to load local config otherwise load global
                match doc::try_load_local(std::env::current_dir()?)? {
                    None => (doc::load_global()?, false),
                    Some(g) => (g, true),
                }
            }
        }
    };

    handle_command(&matches, &mut graph)?;

    if local {
        doc::save_local(
            // Default to current directory if --local is not specified
            PathBuf::from(
                matches
                    .get_one::<String>("local")
                    .unwrap_or(&".".to_string()),
            ),
            &Doc::new(&graph),
        )?;
    } else {
        doc::save_global(&Doc::new(&graph))?;
    }

    Ok(())
}
