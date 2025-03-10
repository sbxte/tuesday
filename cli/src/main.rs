mod display;
mod errors;
mod dates;
mod graph;
mod config;

use std::path::PathBuf;

use chrono::Local;
use clap::{arg, value_parser, Arg, ArgMatches, Command};

use config::{get_config, CliConfig};
use display::Displayer;
use errors::AppError;
use graph::CLIGraphOps;
use rand::rng;
use rand::seq::IndexedRandom as _;
use tuecore::doc::{self, Doc};
use tuecore::graph::node::task::TaskState;
use tuecore::graph::{Graph, GraphGetters};
use dates::parse_datetime_extended;

type AppResult<T> = Result<T, AppError>;

fn handle_command<'a>(matches: &ArgMatches, graph: &mut Graph, config: &CliConfig, displayer: &'a Displayer) -> AppResult<()> {
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

                if config.display.show_connections {
                    displayer.print_link_root(idx, true);
                }
            } else if let Some(when) = date {
                let date = parse_datetime_extended(when)?;

                let empty = String::new();
                let message = sub_matches
                    .get_one::<String>("message")
                    .unwrap_or(&empty);

                let idx = graph.insert_date(message.clone(), date.date_naive());
                if config.display.show_connections {
                    displayer.print_link_dates(idx, true);
                }
            } 

            let message = sub_matches
                .get_one::<String>("message")
                .ok_or(AppError::MissingArgument("adding root node requires message to be given".to_string()))?;
            let idx = if let Some(i) = sub_matches.get_one::<String>("parent") {
                i
            } else {
                return Err(AppError::InvalidArg("Parent ID required!".to_string()));
            };
            let parent = graph.get_index_cli(idx, false)?;
            let to = graph.insert_child(message.to_string(), parent, pseudo)?;

            if config.display.show_connections {
                displayer.print_link(to, parent, true);
            }
        }
        Some(("rm", sub_matches)) => {
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            let assume_date = sub_matches.get_flag("assumedate");
            for id in ids {
                let recursive = sub_matches.get_flag("recursive");
                let node_id = graph.get_index_cli(id, assume_date)?;
                if recursive {
                    graph.remove_children_recursive(node_id)?;
                } else {
                    graph.remove(node_id)?;
                }
                if config.display.show_connections {
                    displayer.print_removal(node_id, recursive);
                }
            }
        }
        Some(("link", sub_matches)) => {
            let assume_date_1 = sub_matches.get_flag("assumedate1");
            let assume_date_2 = sub_matches.get_flag("assumedate2");
            let parent = graph.get_index_cli(
                sub_matches
                    .get_one::<String>("parent")
                    .expect("parent ID required"),
                assume_date_1
            )?;
            let child = graph.get_index_cli(
                sub_matches
                    .get_one::<String>("child")
                    .expect("child ID required"),
                assume_date_2
            )?;
            graph.link(parent, child)?;

            if config.display.show_connections {
                displayer.print_link(child, parent, true);
            }
        }
        Some(("unlink", sub_matches)) => {
            let assume_date_1 = sub_matches.get_flag("assumedate1");
            let assume_date_2 = sub_matches.get_flag("assumedate2");
            let parent = graph.get_index_cli(
                sub_matches
                    .get_one::<String>("parent")
                    .expect("parent ID required"),
                assume_date_1
            )?;
            let child = graph.get_index_cli(
                sub_matches
                    .get_one::<String>("child")
                    .expect("child ID required"),
                assume_date_2
            )?;
            graph.unlink(parent, child)?;

            if config.display.show_connections {
                displayer.print_link(parent, child, false);
            }
        }
        Some(("mv", sub_matches)) => {
            let assume_date_1 = sub_matches.get_flag("assumedate1");
            let assume_date_2 = sub_matches.get_flag("assumedate2");
            let nodes = sub_matches
                .get_many::<String>("node")
                .expect("node ID required");
            let parent = graph.get_index_cli(
            sub_matches
                    .get_one::<String>("parent")
                    .expect("parent ID required"),
                assume_date_2
            )?;

            for node in nodes {
                let node = graph.get_index_cli(node, assume_date_1)?;
                graph.mv(node, parent)?;
                if config.display.show_connections {
                    displayer.print_link(node, parent, true);
                }
            }
        }
        Some(("set", sub_matches)) => {
            let assume_date = sub_matches.get_flag("assumedate");
            let id = graph.get_index_cli(sub_matches.get_one::<String>("ID").expect("ID required"), assume_date)?;
            let state = sub_matches
                .get_one::<TaskState>("state")
                .expect("node state required");
            graph.set_task_state(id, *state, true)?;
        }
        Some(("check", sub_matches)) => {
            let assume_date = sub_matches.get_flag("assumedate");
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = graph.get_index_cli(id, assume_date)?;
                graph.set_task_state(id, TaskState::Done, true)?;
            }
        }
        Some(("uncheck", sub_matches)) => {
            let assume_date = sub_matches.get_flag("assumedate");
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = graph.get_index_cli(id, assume_date)?;
                graph.set_task_state(id, TaskState::None, true)?;
            }
        }
        Some(("arc", sub_matches)) => {
            let assume_date = sub_matches.get_flag("assumedate");
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = graph.get_index_cli(id, assume_date)?;
                graph.set_archived(id, true)?;
            }
        }
        Some(("unarc", sub_matches)) => {
            let assume_date = sub_matches.get_flag("assumedate");
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = graph.get_index_cli(id, assume_date)?;
                graph.set_archived(id, false)?;
            }
        }
        Some(("alias", sub_matches)) => {
            let assume_date = sub_matches.get_flag("assumedate");
            let id = graph.get_index_cli(sub_matches.get_one::<String>("ID").expect("ID required"), assume_date)?;
            let alias = sub_matches
                .get_one::<String>("alias")
                .expect("alias required");
            graph.set_alias(id, alias.clone())?;
        }
        Some(("unalias", sub_matches)) => {
            let assume_date = sub_matches.get_flag("assumedate");
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = graph.get_index_cli(id, assume_date)?;
                graph.unset_alias(id)?;
            }
        }
        Some(("aliases", _)) => {
            let aliases = graph.get_aliases();
            println!("{}", displayer.aliases_title());
            if aliases.len() > 0 {
                for (alias, idx) in aliases {
                    println!(" * {}", displayer.display_id(*idx, Some(alias)));
                }
            } else {
                println!("No added alias.");
            }
        }
        Some(("rename", sub_matches)) => {
            let assume_date = sub_matches.get_flag("assumedate");
            let id = graph.get_index_cli(sub_matches.get_one::<String>("ID").expect("ID required"), assume_date)?;
            let message = sub_matches
                .get_one::<String>("message")
                .expect("ID required");
            graph.rename_node(id, message.to_string())?;
        }
        Some(("ls", sub_matches)) => {
            let assume_date = sub_matches.get_flag("assumedate");
            let depth = if sub_matches.get_flag("recurse") {
                // Override with infinite depth
                0
            } else {
                *sub_matches
                    .get_one::<u32>("depth")
                    .expect("depth should exist")
            };

            let show_archived = sub_matches.get_flag("archived");
            match sub_matches.get_one::<String>("ID") {
                None => displayer.list_roots(graph, depth, show_archived)?,
                Some(id) => displayer.list_children(graph, graph.get_index_cli(&id, assume_date)?, depth, show_archived)?,
            }
        }
        Some(("lsd", sub_matches)) => {
            let show_archived = sub_matches.get_flag("archived");
            displayer.list_dates(graph, show_archived)?;
        }
        Some(("lsa", _)) => {
            displayer.list_archived(graph)?;
        }
        Some(("rand", sub_matches)) => {
            let assume_date = sub_matches.get_flag("assumedate");
            let id = sub_matches
                .get_one::<String>("ID")
                .ok_or(AppError::InvalidArg("ID required".to_string()))?;
            let unchecked = sub_matches.get_flag("unchecked");
            let checked = sub_matches.get_flag("checked");
            if unchecked && checked {
                return Err(AppError::InvalidArg(
                    "--unchecked and --checked cannot be used together!".to_string(),
                ));
            }

            let mut nodes = graph.get_node_children(graph.get_index_cli(id, assume_date)?).clone();
            let item;
            if unchecked {
                nodes.retain(|x| {
                    graph
                        .get_node(*x)
                        .data
                        .as_task()
                        .map(|task| task.state != TaskState::Done)
                        .unwrap_or(true)
                });
                item = nodes.choose(&mut rng());
            } else if checked {
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
                Some(child) => {
                    // TODO: Don't use stat
                    displayer.print_stats(graph, Some(*child))?;
                }
            };
        }
        Some(("stats", sub_matches)) => {
            let assume_date = sub_matches.get_flag("assumedate");
            if let Some(id) = sub_matches.get_one::<String>("ID") {
                displayer.print_stats(graph, Some(graph.get_index_cli( id, assume_date)?))?;
            } else {
                displayer.print_stats(graph, None)?;
            };
        }
        Some(("clean", _)) => {
            graph.clean();
        }
        Some(("cal", sub_matches)) => {
            let date_str = sub_matches.get_one::<String>("date");
            if let Some(date) = date_str {
                let date = parse_datetime_extended(date)?;
                return displayer.print_calendar(graph, &date.date_naive());
            } else {
                let today = Local::now();
                return displayer.print_calendar(graph, &today.date_naive());
            }
        }
        Some(("cp", sub_matches)) => {
            // FIXME: weird logic idk?
            let assume_date_1 = sub_matches.get_flag("assumedate1");
            let assume_date_2 = sub_matches.get_flag("assumedate2");
            let recursive = sub_matches.get_flag("recursive");

            let parent_id = sub_matches
                .get_one::<String>("parent")
                .ok_or(AppError::InvalidArg("parent node ID is required!".to_string()))?;

            let target_exists;

            // if user gives a nonexistent date as a target, make a new date node.
            let parent_idx = if let Ok(idx) = graph.get_index_cli(parent_id, assume_date_2) {
                target_exists = true;
                idx
            } else if let Ok(date) = parse_datetime_extended(parent_id) {
                if !recursive {
                    return Err(AppError::InvalidArg("Copying a date node to a nonexistent date requires --recursive".to_string()))
                }
                target_exists = false;
                graph.insert_date("".to_string(), date.date_naive())
            } else {
                return Err(AppError::IndexRetrievalError("Target node not found!".to_string()));
            };

            let from_ids = sub_matches.get_many::<String>("source")
                .ok_or(AppError::InvalidArg("source node ID(s) is required!".to_string()))?;

            for id in from_ids {
                let from = graph.get_index_cli(id, assume_date_1)?;

                // we make special treatment for date -> date copying, where the target date used
                // to not exist. because the graph.copy method doesn't really care about the type
                // of the node it's copying (everything will turn into normal nodes), we make the
                // target manually then copy the children from the date node.
                // also, recursion is guaranteed because of the logic above.
                if !target_exists {
                    let node = graph.get_node(from);
                    for idx in node.metadata.children {
                        graph.copy_recurse(idx, parent_idx)?;
                    }
                } else {
                    if recursive {
                        graph.copy_recurse(from, parent_idx)?;
                    } else {
                        graph.copy(from, parent_idx)?;
                    }
                }
            }

        }
        Some(("ord", sub_matches)) => {
            let assume_date_1 = sub_matches.get_flag("assumedate1");
            let assume_date_2 = sub_matches.get_flag("assumedate2");

            let direction = sub_matches
                .get_one::<OrderingDirection>("order")
                .ok_or(AppError::InvalidArg("Reordering direction required!".to_string()))?;

            let count = sub_matches
                .get_one::<u32>("count")
                .unwrap_or(&1);

            let node = sub_matches.get_one::<String>("node")
                .ok_or(AppError::InvalidArg("Node ID required!".to_string()))?;

            let parent = sub_matches.get_one::<String>("parent");
                

            let node_idx = graph.get_index_cli(node, assume_date_1)?;
            let parents = graph.get_node(node_idx).metadata.parents;

            let parent_idx;
            if let Some(id) = parent {
                parent_idx = graph.get_index_cli(id, assume_date_2)?;
                if !parents.contains(&parent_idx) {
                    return Err(AppError::InvalidArg(format!("Index {} is not parent of {}!", parent_idx, node_idx)));
                }

            } else {
                if parents.len() > 1 {
                    println!("{}", displayer.parents_title());

                    for id in &parents {
                        let node = graph.get_node(*id);
                        if let Some(alias) = node.metadata.alias {
                            println!("* {} ({})", displayer.display_id(*id, Some(&alias)), node.title);
                        } else {
                            println!("* {} ({})", displayer.display_id(*id, None), node.title);

                        }
                    }
                }
                parent_idx = parents[0];
            }

            match *direction {
                OrderingDirection::Up => graph.reorder_node_delta(node_idx, parent_idx, -(*count as i32))?,
                OrderingDirection::Down => graph.reorder_node_delta(node_idx, parent_idx, *count as i32)?,
            };

        }
        Some(("new-cfg", _)) => {
            println!("{}", displayer.template_cfg());

        }
        _ => return Err(AppError::InvalidSubcommand),
    };

    // TODO: maybe dont run this every time?
    if config.graph.auto_clean {
        graph.clean();
    }

    Ok(())
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, clap::ValueEnum)]
enum OrderingDirection {
    #[default]
    Down,
    Up
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
            .arg(arg!(-r --recursive "Whether to remove child nodes recursively"))
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date"))
        )
        .subcommand(Command::new("link")
            .about("Creates a parent-child edge connection between 2 nodes")
            .arg(arg!(parent: <ID1> "Which node should be the parent in this connection"))
            .arg(arg!(child: <ID2> "Which node should be the child in this connection"))
            .arg(arg!(--assumedate1 "Force ID1 to be interpreted as a date"))
            .arg(arg!(--assumedate2 "Force ID2 to be interpreted as a date"))
        )
        .subcommand(Command::new("unlink")
            .about("Removes a parent-child edge connection between 2 nodes")
            .arg(arg!(parent: <ID1> "Which node should be the parent in this connection"))
            .arg(arg!(child: <ID2> "Which node should be the child in this connection"))
            .arg(arg!(--assumedate_1 "Force ID1 to be interpreted as a date"))
            .arg(arg!(--assumedate_2 "Force ID2 to be interpreted as a date"))
        )
        .subcommand(Command::new("mv")
            .about("Unlink nodes from all current parents, then link to a new parent")
            .arg(arg!(node: <ID1>... "Which nodes to unlink"))
            .arg(arg!(parent: <ID2> "New parent for node"))
            .arg(arg!(--assumedate1 "Force ID1 (all when provided more than one) to be interpreted as a date"))
            .arg(arg!(--assumedate2 "Force ID2 to be interpreted as a date"))
        )
        .subcommand(Command::new("cp")
            .about("Copy a node to a parent")
            .arg(arg!(source: <ID1>... "Which node to copy from"))
            .arg(arg!(parent: <ID2> "Which node to copy to"))
            .arg(arg!(-r --recursive "Whether to copy nodes recursively"))
            .arg(arg!(--assumedate1 "Force IDs 1 to be interpreted as dates"))
            .arg(arg!(--assumedate2 "Force ID 2 to be interpreted as a date"))
        )
        .subcommand(Command::new("ord")
            .about("Reorder a node")
            .arg(arg!(node: <ID1> "Node to reorder"))
            .arg(arg!(<order> "Which direction to reorder node").value_parser(value_parser!(OrderingDirection)))
            .arg(arg!([count] "How many times to move up/down").value_parser(value_parser!(u32)).default_value("1"))
            .arg(arg!(parent: -p --parent <ID2> "Parent of node (can be omitted when there's only one parent)").required(false))
            .arg(arg!(--assumedate1 "Force ID1 to be interpreted as a date"))
            .arg(arg!(--assumedate2 "Force ID2 to be interpreted as a date"))
        )
        .subcommand(Command::new("set")
            .about("Sets a node's state")
            .arg(arg!(<ID> "Which node to modify"))
            .arg(arg!(<state> "What state to set the node").value_parser(value_parser!(TaskState)))
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date"))
        )
        .subcommand(Command::new("check")
            .about("Marks nodes as completed")
            .arg(arg!(<ID>... "Which node(s) to mark as completed"))
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date"))
        )
        .subcommand(Command::new("uncheck")
            .about("Marks nodes as incomplete")
            .arg(arg!(<ID>... "Which node(s) to mark as incomplete"))
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date"))
        )
        .subcommand(Command::new("arc")
            .about("Archives (hides) nodes from view")
            .arg(arg!(<ID>... "Which node(s) to archive"))
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date"))
        )
        .subcommand(Command::new("unarc")
            .about("Unarchives (unhides) nodes from view")
            .arg(arg!(<ID>... "Which node(s) to archive"))
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date"))
        )
        .subcommand(Command::new("alias")
            .about("Adds an alias for a node")
            .arg(arg!(<ID> "Which node to alias"))
            .arg(arg!(<alias> "What alias to give this node"))
            .arg(arg!(-D --assumedate "Force the ID to be interpreted as a date"))
        )
        .subcommand(Command::new("unalias")
            .about("Removes nodes' alias")
            .long_about("Removes all aliases of nodes")
            .arg(arg!(<ID>... "Which node(s) to remove aliases"))
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date"))
        )
        .subcommand(Command::new("aliases")
            .about("Lists all aliases")
        )
        .subcommand(Command::new("rename")
            .about("Edit a node's message")
            .arg(arg!(<ID> "Which node to edit"))
            .arg(arg!(<message> "What new message to give it"))
            .arg(arg!(-D --assumedate "Force the ID to be interpreted as a date"))
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
            .arg(arg!(-D --assumedate "Force the ID to be interpreted as a date"))
        )
        .subcommand(Command::new("lsd")
            .about("Lists all date nodes")
            .arg(arg!(-a --archived "Display archived nodes"))
        )
        .subcommand(Command::new("lsa")
            .about("Lists all archived nodes")
        )
        .subcommand(Command::new("rand")
            .about("Picks a random child node")
            .arg(arg!(<ID> "Which parent node to randomly pick a child from"))
            .arg(arg!(-u --unchecked "Only pick among unchecked tasks"))
            .arg(arg!(-c --checked "Only pick among checked tasks"))
            .arg(arg!(-D --assumedate "Force the ID to be interpreted as a date"))
        )
        .subcommand(Command::new("stats")
            .about("Displays statistics of a node")
            .arg(arg!([ID] "Which node to display stats"))
            .arg(arg!(-D --assumedate "Force the ID to be interpreted as a date"))
        )
        .subcommand(Command::new("clean")
            .about("Compresses and cleans up the graph")
        )
        .subcommand(Command::new("cal")
            .about("Print calendar to display date nodes")
            .arg(arg!(date: [date] "Date to use (only the month will be considered)")
                .value_parser(value_parser!(String))
                .default_value("today"))
        )
        .subcommand(Command::new("new-cfg")
            .about("Dump a default configuration file. Recommended: run then redirect and save to ~/.tueconf.toml")
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
    
    let config = get_config()?;

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

    let displayer = Displayer::new(&config);

    handle_command(&matches, &mut graph, &config, &displayer)?;

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
