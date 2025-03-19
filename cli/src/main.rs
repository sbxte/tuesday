mod blueprints;
mod config;
mod dates;
mod display;
mod errors;
mod graph;
mod paths;

use std::ffi::{OsStr, OsString};
use std::fs::{create_dir, remove_file, File};
use std::path::PathBuf;

use blueprints::{try_get_blueprint_from_save_dir, BlueprintDoc, BlueprintError, get_blueprints_listing};
use chrono::Local;
use clap::{arg, value_parser, Arg, ArgMatches, Command};

use config::{get_config, CliConfig};
use display::Displayer;
use errors::AppError;
use graph::{graph_from_blueprint, new_graph_indices_map, CLIGraphOps};
use rand::rng;
use rand::seq::IndexedRandom;
use tuecore::doc::{self, get_doc_ver, Doc};
use tuecore::graph::node::task::TaskState;
use tuecore::graph::{Graph, GraphGetters};
use dates::parse_datetime_extended;

type AppResult<T> = Result<T, AppError>;

fn get_bp_path(save_dir: PathBuf, name: &str) -> PathBuf {
    let mut path = save_dir.to_path_buf();
    path.push(format!("{}.yaml", name));
    path
}

fn handle_blueprints_command<'a>(subcommand: Option<(&str, &ArgMatches)>, graph: &mut Graph, config: &CliConfig, displayer: &'a Displayer) -> AppResult<()> {
    match subcommand {
        Some(("edit", sub_matches)) => {
            if let Some(args) = sub_matches.get_raw("args") {
                let mut args: Vec<&OsStr> = args.collect();
                let edit_cmd = OsString::from("edit");
                args.insert(0, &edit_cmd);

                let name = sub_matches.get_one::<String>("name").ok_or(AppError::InvalidSubcommand)?;

                // prioritize editing the file from current directory, if the file exists
                let path = if PathBuf::from(name).exists() {
                    PathBuf::from(name)
                } else {
                    get_bp_path(config.blueprints.store_path.clone(), name)
                };

                // TODO: dont let infinite recursion possible lol (tuecli bp edit path bp edit
                // path bp edit path ...)
                // while it won't actually work, the command would still get validated by clap (not
                // validated by us tho!)
                let matches = cli()?.get_matches_from(args);

                let bp = blueprints::get_doc(&mut File::open(&path)?).map_err(|_| BlueprintError::FailedToAccess("Failed to match to any existing blueprint!".to_string()))?;

                let mut graph = graph_from_blueprint(&bp)?;
                handle_graph_command(matches.subcommand(), &mut graph, config, displayer, true)?;

                let new_bp = BlueprintDoc::from_idx(&graph, get_doc_ver(), 0, bp.author);
                let mut new_file = File::create(&path)?;
                new_bp.save_to_file(&mut new_file)?;
            } else {
                return Err(AppError::InvalidSubcommand)
            }
        }
        Some(("show", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").ok_or(AppError::InvalidSubcommand)?;
            
            let bp = if let Ok(bp) = try_get_blueprint_from_save_dir(&config.blueprints.store_path, name) {
                bp
            } else if let Ok(bp) = blueprints::get_doc(&mut File::open(name)?) {
                    bp
            } else {
                return Err(BlueprintError::FailedToAccess("failed to match to any existing blueprint!".to_string()).into());
            };

            let graph = graph_from_blueprint(&bp)?;

            println!("{}", displayer.display_bp_title(bp.author.as_deref(), &name));
            displayer.list_roots(&graph, 0, false)?;
        }
        Some(("ls", _)) => {
            let bps = get_blueprints_listing(&config.blueprints.store_path)?;
            displayer.list_blueprints(&bps);
        }
        Some(("rm", sub_matches)) => {
            let path = get_bp_path(config.blueprints.store_path.clone(), sub_matches.get_one::<String>("name").ok_or(AppError::InvalidSubcommand)?);
            remove_file(&path)?;
            println!("{}", displayer.display_bp_deleted(&path.to_string_lossy()));
        }
        Some(("export", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").ok_or(AppError::InvalidSubcommand)?;
            let bp = try_get_blueprint_from_save_dir(&config.blueprints.store_path, name)?;
            println!("{}", bp.to_string());

        }
        Some(("ins", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").ok_or(AppError::InvalidSubcommand)?;
            let title = sub_matches.get_one::<String>("title");
            let id = sub_matches.get_one::<String>("ID");
            let root = sub_matches.get_flag("root");
            let assumedate = sub_matches.get_flag("assumedate");


            let path = if PathBuf::from(name).exists() {
                PathBuf::from(name)
            } else {
                get_bp_path(config.blueprints.store_path.clone(), name)
            };

            let bp = blueprints::get_doc(&mut File::open(&path)?).map_err(|_| BlueprintError::FailedToAccess("Failed to match to any existing blueprint!".to_string()))?;

            let map = new_graph_indices_map(&bp, &graph, graph.get_nodes().len());

            // TODO: send help
            let new_parent = &bp.graph.nodes[bp.parent];
            let parent_id = if root {
                graph.insert_root(title.unwrap_or(&new_parent.title).to_string(), new_parent.data.is_pseudo())
            } else {
                // id shouldn't be None here since !root implies id being Some(..)
                let id = graph.get_index_cli(id.unwrap(), assumedate)?;
                graph.insert_child(title.unwrap_or(&new_parent.title).to_string(), id, new_parent.data.is_pseudo())?

            };


            for child in &new_parent.metadata.children {
                graph.insert_blueprint_recurse(&map, &bp, *child, parent_id)?;
            }

            graph.update_node_metadata_on_blueprint(bp.parent, &map, &bp);


            if config.display.show_connections {
                displayer.display_bp_inserted(name, parent_id);
            }

        }
        Some(("save", sub_matches)) => {
            let id = sub_matches.get_one::<String>("ID").ok_or(AppError::InvalidSubcommand)?;
            let mut author = sub_matches.get_one::<String>("author");
            let name = sub_matches.get_one::<String>("name").ok_or(AppError::InvalidSubcommand)?;
            let to_file = sub_matches.get_flag("to_file");
            let preserve = sub_matches.get_flag("preserve");
            let overwrite = sub_matches.get_flag("overwrite");
            let assumedate = sub_matches.get_flag("assumedate");

            let node_id = graph.get_index_cli(id, assumedate)?;
            let bp = BlueprintDoc::from_idx(graph, get_doc_ver(), node_id, author.take().cloned());

            let path = if to_file {
                format!("{}.yaml", name).into()
            } else {
                if !&config.blueprints.store_path.exists() {
                    create_dir(&config.blueprints.store_path).map_err(|e| BlueprintError::SaveDirError(e.to_string()))?;
                }
                let mut path = config.blueprints.store_path.clone();
                path.push(format!("{}.yaml", name));
                path
            };

            if path.exists() && !overwrite {
                return Err(BlueprintError::FileExists(path.to_string_lossy().into()).into());
            }

            let mut file = File::create(&path)?;
            bp.save_to_file(&mut file)?;

            println!("{}", displayer.display_bp_written(&path.to_string_lossy()));

            if !preserve {
                graph.remove_children_recursive(node_id)?;
                if config.graph.auto_clean {
                    graph.clean();
                }
            }
        }
        _ => return Err(AppError::InvalidSubcommand)
    };
    Ok(())
}

fn handle_graph_command<'a>(subcommand: Option<(&str, &ArgMatches)>, graph: &mut Graph, config: &CliConfig, displayer: &'a Displayer, is_bp_graph: bool) -> AppResult<()> {
    match subcommand {
        Some(("add", sub_matches)) => {
            let root = sub_matches.get_flag("root");
            let date = sub_matches.get_one::<String>("date");
            let pseudo = sub_matches.get_flag("pseudo");

            if (root || date.is_some()) && is_bp_graph {
                return Err(AppError::InvalidArg(
                    "Cannot add a root or date node to a blueprint".to_string(),
                ));
            }
            if date.is_some() && root {
                return Err(AppError::ConflictingArgs(
                    "Node cannot be both date node and root node!".to_string(),
                ));
            };

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
            }  else {
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

        }
        Some(("rm", sub_matches)) => {
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            let assume_date = sub_matches.get_flag("assumedate");
            for id in ids {
                let recursive = sub_matches.get_flag("recursive");
                let node_id = graph.get_index_cli(id, assume_date)?;

                // do not remove the root of a blueprint
                if node_id == 0 && is_bp_graph {
                    return Err(AppError::InvalidArg(
                        "Cannot remove parent from blueprint!".to_string(),
                    ));
                }

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
            if is_bp_graph {
                return Err(AppError::InvalidArg(
                    "Cannot check a blueprint node!".to_string(),
                ));
            }
            let assume_date = sub_matches.get_flag("assumedate");
            let ids = sub_matches.get_many::<String>("ID").expect("ID required");
            for id in ids {
                let id = graph.get_index_cli(id, assume_date)?;
                graph.set_task_state(id, TaskState::Done, true)?;
            }
        }
        Some(("uncheck", sub_matches)) => {
            if is_bp_graph {
                return Err(AppError::InvalidArg(
                    "Cannot uncheck a blueprint node!".to_string(),
                ));
            }
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
            if is_bp_graph {
                return Err(AppError::InvalidArg(
                    "Aliases are not supported in blueprints!".to_string(),
                ));
            }
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
                None => displayer.list_roots(graph, depth, !show_archived)?,
                Some(id) => displayer.list_children(graph, graph.get_index_cli(&id, assume_date)?, depth, !show_archived)?,
            }
        }
        Some(("lsd", sub_matches)) => {
            let show_archived = sub_matches.get_flag("archived");
            displayer.list_dates(graph, !show_archived)?;
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

                // we make special treatment for date -> date copying, when the target date used
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
    }

    // TODO: maybe dont run this every time?
    if config.graph.auto_clean {
        graph.clean();
    }

    Ok(())
}

fn handle_command<'a>(matches: &ArgMatches, graph: &mut Graph, config: &CliConfig, displayer: &'a Displayer) -> AppResult<()> {
    match matches.subcommand() {
        Some(("bp", sub_matches)) => handle_blueprints_command(sub_matches.subcommand(), graph, config, displayer),
        Some((_, _)) => handle_graph_command(matches.subcommand(), graph, config, displayer, false),
        _ => Err(AppError::InvalidSubcommand),
    }
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
        .arg(arg!(config: -c --config <path>)
            .value_parser(value_parser!(PathBuf))
            .required(false))
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
            .arg(arg!(--assumedate1 "Force ID1 to be interpreted as a date"))
            .arg(arg!(--assumedate2 "Force ID2 to be interpreted as a date"))
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
        .subcommand(Command::new("bp")
        .subcommand_required(true)
            .about("Blueprints-related operations")
            .subcommand(Command::new("ls")
                .about("List blueprints from the save directory")
            )
            .subcommand(Command::new("save")
                .about("Save node to blueprint")
                .arg(arg!(<ID> "Which node to turn to a blueprint"))
                .arg(arg!(<name> "Blueprint name")
                    .value_parser(value_parser!(String)))
                .arg(arg!(author: -a --author <name> "Author name of the blueprint")
                    .value_parser(value_parser!(String)))
                .arg(arg!(to_file: -f --file "Write to file with name <name> instead of to your save directory"))
                .arg(arg!(preserve: -p --preserve "Preserve node after the conversion"))
                .arg(arg!(overwrite: -o --overwrite "Overwrite existing blueprint"))
                .arg(arg!(-D --assumedate "Force the ID to be interpreted as a date"))
            )
            .subcommand(Command::new("rm")
                .about("Remove a blueprint")
                .arg(arg!(<name> "Blueprint name to remove"))
            )
            .subcommand(Command::new("ins")
                .about("Insert the blueprint to graph")
                .arg(arg!(<name> "Name or path of blueprint file"))
                .arg(Arg::new("ID").help("Parent of blueprint tree").required_unless_present("root"))
                .arg(arg!([message] "Title of the new blueprint node"))
                .arg(arg!(root: -r --root "Insert the blueprint to root"))
                .arg(arg!(-D --assumedate "Force the ID to be interpreted as a date"))
            )
            .subcommand(Command::new("show")
                .about("Show a blueprint file in tree/graph form")
                .arg(arg!(<name> "Blueprint name or file"))
            )
            .subcommand(Command::new("export")
                .about("Dump existing blueprint")
                .arg(arg!(<name> "Blueprint name to export"))
            )
            .subcommand(Command::new("edit")
                .about("Edits a blueprint file in-place")
                .arg(arg!(<name> "Name or path of blueprint")
                    .value_parser(value_parser!(String))
                )
                .arg(arg!(args: <args>... "Edit arguments"))
            )
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
    
    let config = get_config(matches.get_one::<PathBuf>("config"))?;

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
