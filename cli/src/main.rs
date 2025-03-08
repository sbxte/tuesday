mod display;
mod errors;
mod dates;

use std::path::PathBuf;

use clap::{arg, value_parser, Arg, ArgMatches, Command};

use display::{aliases_title, display_alias, print_calendar, print_link, print_link_dates, print_link_root, print_removal, CLIDisplay};
use errors::AppError;
use rand::rng;
use rand::seq::IndexedRandom as _;
use tuecore::doc::{self, Doc};
use tuecore::graph::node::task::TaskState;
use tuecore::graph::{Graph, GraphGetters};
use dates::parse_datetime_extended;

type AppResult<T> = Result<T, AppError>;

/// Wrapper for the `get_index` method under `Graph` that also takes care of retrieving indices of date nodes.
fn get_index(graph: &Graph, id: &str, assume_date: bool) -> AppResult<usize> {
    // When user forces the ID to be interpreted as a date, just search through the dates hashmap.
    if assume_date {
        let date = parse_datetime_extended(id)?.date_naive();
        return Ok(graph.get_date_index(&date)?);
    }

    // Normally, any number below the amount of dates from the current month can also be
    // interpreted as a date. However, when the user is just writing arbitrary number, it's most
    // likely that they're working with node indices. With this assumption, we parse any valid
    // usize as a node index.
    // edit: I may be wrong about this; maybe the implementation of parse_datetime is different
    // than how GNU's date does it. uhh, we'll just leave it as is.
    if let Ok(_) = id.parse::<u64>() {
        return Ok(graph.get_index(id)?)
    }

    // The second priority to our ID matching are aliases.
    if let Ok(idx) = graph.get_index(id) {
        return Ok(idx)
    }

    // If none of those worked, then interpret the ID as a date.
    if let Ok(date) = parse_datetime_extended(id)  {
        let idx = graph.get_date_index(&date.date_naive())?;
        return Ok(idx);
    }

    // If that didn't work as well then the ID is invalid.
    Err(AppError::IndexRetrievalError("Failed to match index with node".to_string()))
}

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
                let date = parse_datetime_extended(when)?;

                let empty = String::new();
                let message = sub_matches
                    .get_one::<String>("message")
                    .unwrap_or(&empty);

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
            let parent = get_index(graph, idx, false)?;
            let to = graph.insert_child(message.to_string(), parent, pseudo)?;
            print_link(to, parent, true);
            Ok(())
        }
        Some(("rm", sub_matches)) => {
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            let assume_date = sub_matches.get_one::<bool>("assumedate").unwrap_or(&false);
            for id in ids {
                let recursive = sub_matches.get_flag("recursive");
                let node_id = get_index(graph, id, *assume_date)?;
                if recursive {
                    graph.remove_children_recursive(node_id)?;
                } else {
                    graph.remove(node_id)?;
                }
                print_removal(node_id, recursive);
            }
            Ok(())
        }
        Some(("link", sub_matches)) => {
            let assume_date = sub_matches.get_one::<bool>("assumedate").unwrap_or(&false);
            let parent = get_index(
                graph,
                sub_matches
                    .get_one::<String>("parent")
                    .expect("parent ID required"),
                *assume_date
            )?;
            let child = get_index(
                graph,
                sub_matches
                    .get_one::<String>("child")
                    .expect("child ID required"),
                *assume_date
            )?;
            graph.link(parent, child)?;
            print_link(parent, child, true);
            Ok(())
        }
        Some(("unlink", sub_matches)) => {
            let assume_date = sub_matches.get_one::<bool>("assumedate").unwrap_or(&false);
            let parent = get_index(
                graph,
                sub_matches
                    .get_one::<String>("parent")
                    .expect("parent ID required"),
                *assume_date
            )?;
            let child = get_index(
                graph,
                sub_matches
                    .get_one::<String>("child")
                    .expect("child ID required"),
                *assume_date
            )?;
            graph.unlink(parent, child)?;
            print_link(parent, child, false);
            Ok(())
        }
        Some(("mv", sub_matches)) => {
            let assume_date_1 = sub_matches.get_one::<bool>("assumedate1").unwrap_or(&false);
            let assume_date_2 = sub_matches.get_one::<bool>("assumedate2").unwrap_or(&false);
            let nodes = sub_matches
                .get_many::<String>("node")
                .expect("node ID required");
            let parent = get_index(
                graph,
                sub_matches
                    .get_one::<String>("parent")
                    .expect("parent ID required"),
                *assume_date_2
            )?;

            for node in nodes {
                let node = get_index(graph, node, *assume_date_1)?;
                graph.clean_parents(node)?;
                graph.link(parent, node)?;
            }
            Ok(())
        }
        Some(("set", sub_matches)) => {
            let assume_date = sub_matches.get_one::<bool>("assumedate").unwrap_or(&false);
            let id = get_index(graph, sub_matches.get_one::<String>("ID").expect("ID required"), *assume_date)?;
            let state = sub_matches
                .get_one::<TaskState>("state")
                .expect("node state required");
            graph.set_task_state(id, *state, true)?;
            Ok(())
        }
        Some(("check", sub_matches)) => {
            let assume_date = sub_matches.get_one::<bool>("assumedate").unwrap_or(&false);
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = get_index(graph, id, *assume_date)?;
                graph.set_task_state(id, TaskState::Done, true)?;
            }
            Ok(())
        }
        Some(("uncheck", sub_matches)) => {
            let assume_date = sub_matches.get_one::<bool>("assumedate").unwrap_or(&false);
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = get_index(graph, id, *assume_date)?;
                graph.set_task_state(id, TaskState::None, true)?;
            }
            Ok(())
        }
        Some(("arc", sub_matches)) => {
            let assume_date = sub_matches.get_one::<bool>("assumedate").unwrap_or(&false);
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = get_index(graph, id, *assume_date)?;
                graph.set_archived(id, true)?;
            }
            Ok(())
        }
        Some(("unarc", sub_matches)) => {
            let assume_date = sub_matches.get_one::<bool>("assumedate").unwrap_or(&false);
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = get_index(graph, id, *assume_date)?;
                graph.set_archived(id, false)?;
            }
            Ok(())
        }
        Some(("alias", sub_matches)) => {
            let assume_date = sub_matches.get_one::<bool>("assumedate").unwrap_or(&false);
            let id = get_index(graph, sub_matches.get_one::<String>("ID").expect("ID required"), *assume_date)?;
            let alias = sub_matches
                .get_one::<String>("alias")
                .expect("alias required");
            graph.set_alias(id, alias.clone())?;
            Ok(())
        }
        Some(("unalias", sub_matches)) => {
            let assume_date = sub_matches.get_one::<bool>("assumedate").unwrap_or(&false);
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = get_index(graph, id, *assume_date)?;
                graph.unset_alias(id)?;
            }
            Ok(())
        }
        Some(("aliases", _)) => {
            let aliases = graph.get_aliases();
            println!("{}", aliases_title());
            if aliases.len() > 0 {
                for (alias, idx) in aliases {
                    println!(" * {}", display_alias(*idx, alias));
                }
            } else {
                println!("No added alias.");
            }
            Ok(())
        }
        Some(("rename", sub_matches)) => {
            let assume_date = sub_matches.get_one::<bool>("assumedate").unwrap_or(&false);
            let id = get_index(graph, sub_matches.get_one::<String>("ID").expect("ID required"), *assume_date)?;
            let message = sub_matches
                .get_one::<String>("message")
                .expect("ID required");
            graph.rename_node(id, message.to_string())?;
            Ok(())
        }
        Some(("ls", sub_matches)) => {
            let assume_date = sub_matches.get_one::<bool>("assumedate").unwrap_or(&false);
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
                Some(id) => graph.list_children(get_index(graph, &id, *assume_date)?, depth, show_archived)?,
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
            let assume_date = sub_matches.get_one::<bool>("assumedate").unwrap_or(&false);
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

            let mut nodes = graph.get_node_children(get_index(graph, id, *assume_date)?).clone();
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
                Some(child) => {
                    // TODO: Don't use stat
                    graph.print_stats(Some(*child))?;
                }
            };
            Ok(())
        }
        Some(("stats", sub_matches)) => {
            let assume_date = sub_matches.get_one::<bool>("assumedate").unwrap_or(&false);
            if let Some(id) = sub_matches.get_one::<String>("ID") {
                graph.print_stats(Some(get_index(graph, id, *assume_date)?))
            } else {
                graph.print_stats(None)
            }
        }
        Some(("clean", _)) => {
            graph.clean();
            Ok(())
        }
        Some(("cal", sub_matches)) => {
            let date_str = sub_matches.get_one::<String>("date");
            if let Some(date) = date_str {
                let date = parse_datetime_extended(date)?;
                return print_calendar(graph, &date.date_naive());
            } else {
                let today = parse_datetime_extended("today")?;
                return print_calendar(graph, &today.date_naive());
            }
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
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
        )
        .subcommand(Command::new("link")
            .about("Creates a parent-child edge connection between 2 nodes")
            .arg(arg!(parent: <ID1> "Which node should be the parent in this connection"))
            .arg(arg!(child: <ID2> "Which node should be the child in this connection"))
            .arg(arg!(-D1 --assumedate1 "Force ID1 to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
            .arg(arg!(-D2 --assumedate2 "Force ID2 to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
        )
        .subcommand(Command::new("unlink")
            .about("Removes a parent-child edge connection between 2 nodes")
            .arg(arg!(parent: <ID1> "Which node should be the parent in this connection"))
            .arg(arg!(child: <ID2> "Which node should be the child in this connection"))
            .arg(arg!(-D1 --assumedate1 "Force ID1 to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
            .arg(arg!(-D2 --assumedate2 "Force ID2 to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
        )
        .subcommand(Command::new("mv")
            .about("Unlink nodes from all current parents, then link to a new parent")
            .arg(arg!(node: <ID1>... "Which nodes to unlink"))
            .arg(arg!(parent: <ID2> "New parent for node"))
            .arg(arg!(-D1 --assumedate1 "Force ID1 (all when provided more than one) to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
            .arg(arg!(-D2 --assumedate2 "Force ID2 to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
        )
        .subcommand(Command::new("set")
            .about("Sets a node's state")
            .arg(arg!(<ID> "Which node to modify"))
            .arg(arg!(<state> "What state to set the node").value_parser(value_parser!(TaskState)))
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
        )
        .subcommand(Command::new("check")
            .about("Marks nodes as completed")
            .arg(arg!(<ID>... "Which node(s) to mark as completed"))
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
        )
        .subcommand(Command::new("uncheck")
            .about("Marks nodes as incomplete")
            .arg(arg!(<ID>... "Which node(s) to mark as incomplete"))
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
        )
        .subcommand(Command::new("arc")
            .about("Archives (hides) nodes from view")
            .arg(arg!(<ID>... "Which node(s) to archive"))
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
        )
        .subcommand(Command::new("unarc")
            .about("Unarchives (unhides) nodes from view")
            .arg(arg!(<ID>... "Which node(s) to archive"))
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
        )
        .subcommand(Command::new("alias")
            .about("Adds an alias for a node")
            .arg(arg!(<ID> "Which node to alias"))
            .arg(arg!(<alias> "What alias to give this node"))
            .arg(arg!(-D --assumedate "Force the ID to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
        )
        .subcommand(Command::new("unalias")
            .about("Removes nodes' alias")
            .long_about("Removes all aliases of nodes")
            .arg(arg!(<ID>... "Which node(s) to remove aliases"))
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
        )
        .subcommand(Command::new("aliases")
            .about("Lists all aliases")
        )
        .subcommand(Command::new("rename")
            .about("Edit a node's message")
            .arg(arg!(<ID> "Which node to edit"))
            .arg(arg!(<message> "What new message to give it"))
            .arg(arg!(-D --assumedate "Force the ID to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
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
            .arg(arg!(-D --assumedate "Force the IDs to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
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
            .arg(arg!(-D --assumedate "Force the ID to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
        )
        .subcommand(Command::new("stats")
            .about("Displays statistics of a node")
            .arg(arg!([ID] "Which node to display stats"))
            .arg(arg!(-D --assumedate "Force the ID to be interpreted as a date").action(clap::ArgAction::SetTrue).default_value("false"))
        )
        .subcommand(Command::new("clean")
            .about("Compresses and cleans up the graph")
        )
        .subcommand(Command::new("cal")
            .about("Print calendar to display date nodes")
            .arg(arg!(date: [date] "Date to use (only the month will be considered)").value_parser(value_parser!(String)).default_value("today"))
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
