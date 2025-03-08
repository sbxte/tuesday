mod display;
mod errors;
use std::path::PathBuf;

use clap::{arg, value_parser, Arg, ArgMatches, Command};

use display::{aliases_title, display_alias, print_link, print_link_dates, print_link_root, print_removal, CLIDisplay};
use errors::AppError;
use rand::rng;
use rand::seq::IndexedRandom as _;
use tuecore::doc::{self, Doc};
use tuecore::graph::node::task::TaskState;
use tuecore::graph::{Graph, GraphGetters};
use parse_datetime::parse_datetime;

type AppResult<T> = Result<T, AppError>;

fn handle_command(matches: &ArgMatches, graph: &mut Graph) -> AppResult<()> {
    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let root = sub_matches.get_flag("root");
            let date = sub_matches.get_one::<String>("date");
            let pseudo = sub_matches.get_flag("pseudo");
            if date.is_some() && root {
                return Err(AppError::ConflictingArgs(
                    "Node cannot be both date node and root node!".to_string(),
                ));
            }
            if root {
                let message = sub_matches
                    .get_one::<String>("message")
                    .ok_or(AppError::MissingArgument("adding root node requires message to be given".to_string()))?;
                let idx = graph.insert_root(message.to_string(), pseudo);
                print_link_root(idx, true);
                return Ok(());
            } else if let Some(when) = date {
                if !Graph::is_date(when) {
                    return Err(AppError::MalformedDate(when.to_string()));
                }
                let date = parse_datetime(when)?;

                // TODO: make this default configurabe
                let default_date = format!("{}", date.format("%Y-%m-%d"));
                let message = sub_matches
                    .get_one::<String>("message")
                    .unwrap_or(&default_date);

                let idx = graph.insert_date(message.clone(), date.date_naive());
                print_link_dates(idx, true);
                return Ok(());
            } 

            let message = sub_matches
                .get_one::<String>("message")
                .ok_or(AppError::MissingArgument("adding root node requires message to be given".to_string()))?;
            let idx = if let Some(i) = sub_matches.get_one::<String>("parent") {
                i
            } else {
                return Err(AppError::InvalidArg("Parent ID required!".to_string()));
            };
            let parent = graph.get_index(idx)?;
            let to = graph.insert_child(message.to_string(), parent, pseudo)?;
            print_link(to, parent, true);
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
                print_removal(id, recursive);
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
            print_link(parent, child, true);
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
            print_link(parent, child, false);
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
                .get_one::<TaskState>("state")
                .expect("node state required");
            graph.set_task_state(id, *state, true)?;
            Ok(())
        }
        Some(("check", sub_matches)) => {
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = graph.get_index(id)?;
                graph.set_task_state(id, TaskState::Done, true)?;
            }
            Ok(())
        }
        Some(("uncheck", sub_matches)) => {
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = graph.get_index(id)?;
                graph.set_task_state(id, TaskState::None, true)?;
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
            let aliases = graph.get_aliases();
            println!("{}", aliases_title());
            for (alias, idx) in aliases {
                println!(" * {}", display_alias(*idx, alias));
            }
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
            let id = sub_matches
                .get_one::<String>("ID")
                .ok_or(AppError::InvalidArg("ID required".to_string()))?;
            let unchecked = sub_matches.get_one::<bool>("unchecked").unwrap_or(&false);
            let checked = sub_matches.get_one::<bool>("checked").unwrap_or(&false);
            if *unchecked && *checked {
                return Err(AppError::InvalidArg(
                    "--unchecked and --checked cannot be used together!".to_string(),
                ));
            }

            let mut nodes = graph.get_node_children(graph.get_index(id)?).clone();
            let item;
            if *unchecked {
                nodes.retain(|x| {
                    graph
                        .get_node(*x)
                        .data
                        .as_task()
                        .map(|task| task.state != TaskState::Done)
                        .unwrap_or(true)
                });
                item = nodes.choose(&mut rng());
            } else if *checked {
                nodes.retain(|x| {
                    graph
                        .get_node(*x)
                        .data
                        .as_task()
                        .map(|task| task.state == TaskState::Done)
                        .unwrap_or(true)
                });
                item = nodes.choose(&mut rng());
            } else {
                item = nodes.choose(&mut rng());
            }
            match item {
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
        _ => Err(AppError::InvalidSubcommand),
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
            .arg(Arg::new("message").help("This node's message").required_unless_present_any(vec!["date", "root"]))
            .arg(Arg::new("parent").help("Parent to assign the added node to").required_unless_present_any(vec!["date", "root"]))
            .arg(arg!(-u --pseudo "Makes this a pseudo node (does not count towards parent completion)")
                .required(false))
            .arg(arg!(-r --root "Makes this a root node")
                .conflicts_with_all(["parent", "date"]))
            .arg(arg!(-d --date <date> "Makes this a date node")
                .value_parser(value_parser!(String))
                .conflicts_with_all(["parent", "root"]))
        )
        .subcommand(Command::new("rm")
            .about("Removes nodes from the graph")
            .arg(arg!(<ID>... "Which nodes to remove"))
            .arg(arg!(-r --recursive "Whether to remove child nodes recursively").required(false))
        )
        .subcommand(Command::new("link")
            .about("Creates a parent-child edge connection between 2 nodes")
            .arg(arg!(parent: <ID1> "Which node should be the parent in this connection"))
            .arg(arg!(child: <ID2> "Which node should be the child in this connection"))
        )
        .subcommand(Command::new("unlink")
            .about("Removes a parent-child edge connection between 2 nodes")
            .arg(arg!(parent: <ID1> "Which node should be the parent in this connection"))
            .arg(arg!(child: <ID2> "Which node should be the child in this connection"))
        )
        .subcommand(Command::new("mv")
            .about("Unlink nodes from all current parents, then link to a new parent")
            .arg(arg!(node: <ID1>... "Which nodes to unlink"))
            .arg(arg!(parent: <ID2> "New parent for node"))
        )
        .subcommand(Command::new("set")
            .about("Sets a node's state")
            .arg(arg!(<ID> "Which node to modify"))
            .arg(arg!(<state> "What state to set the node").value_parser(value_parser!(TaskState)))
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
            .arg(arg!(-u --unchecked "Only pick among unchecked tasks"))
            .arg(arg!(-c --checked "Only pick among checked tasks"))
        )
        .subcommand(Command::new("stats")
            .about("Displays statistics of a node")
            .arg(arg!([ID] "Which node to display stats"))
        )
        .subcommand(Command::new("clean")
            .about("Compresses and cleans up the graph")
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

    let (mut graph, local) = match (
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
