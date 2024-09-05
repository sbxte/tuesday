#![allow(unreachable_code)]

use anyhow::{bail, Result};
use clap::{arg, value_parser, ArgMatches, Command};
use graph::{ErrorType, NodeState};

mod graph;
mod save;

fn handle_command(matches: ArgMatches, graph: &mut graph::Graph) -> Result<()> {
    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let message = sub_matches.get_one::<String>("message").expect("required");
            let root = sub_matches.get_flag("root");
            let date = sub_matches.get_flag("date");
            let pseudo = sub_matches.get_flag("pseudo");
            if date && root {
                bail!("Node cannot be both date node and root node!");
            }
            if date {
                if !graph::Graph::is_date(message) {
                    return Err(ErrorType::MalformedDate(message.to_string()))?;
                }
                graph.insert_date(message.to_string());
            } else if root {
                graph.insert_root(message.to_string(), pseudo);
            } else {
                let parent = sub_matches.get_one::<String>("parent").expect("required");
                graph.insert_child(message.to_string(), parent.to_string(), pseudo)?;
            }
            Ok(())
        }
        Some(("rm", sub_matches)) => {
            let id = sub_matches.get_one::<String>("ID").expect("required");
            let recursive = sub_matches.get_flag("recursive");
            if recursive {
                graph.remove_children_recursive(id.to_string())?;
            } else {
                graph.remove(id.to_string())?;
            }
            Ok(())
        }
        Some(("link", sub_matches)) => {
            let parent = sub_matches.get_one::<String>("parent").expect("required");
            let child = sub_matches.get_one::<String>("child").expect("required");
            graph.link(parent.to_string(), child.to_string())?;
            Ok(())
        }
        Some(("unlink", sub_matches)) => {
            let parent = sub_matches.get_one::<String>("parent").expect("required");
            let child = sub_matches.get_one::<String>("child").expect("required");
            graph.unlink(parent.to_string(), child.to_string())?;
            Ok(())
        }
        Some(("mv", sub_matches)) => {
            let node = sub_matches.get_one::<String>("node").expect("required");
            let parent = sub_matches.get_one::<String>("parent").expect("required");

            graph.clean_parents(node.to_string())?;
            graph.link(parent.to_string(), node.to_string())?;

            Ok(())
        }
        Some(("setnoprop", sub_matches)) => {
            let id = sub_matches.get_one::<String>("ID").expect("required");
            let state = sub_matches.get_one::<NodeState>("state").expect("required");
            graph.set_state(id.to_string(), *state, false)?;
            Ok(())
        }
        Some(("check", sub_matches)) => {
            let id = sub_matches.get_one::<String>("ID").expect("required");
            graph.set_state(id.to_string(), NodeState::Complete, true)?;
            Ok(())
        }
        Some(("uncheck", sub_matches)) => {
            let id = sub_matches.get_one::<String>("ID").expect("required");
            graph.set_state(id.to_string(), NodeState::None, true)?;
            Ok(())
        }
        Some(("alias", sub_matches)) => {
            let id = sub_matches.get_one::<String>("ID").expect("required");
            let alias = sub_matches.get_one::<String>("alias").expect("required");
            graph.set_alias(id.to_string(), alias.to_string())?;
            Ok(())
        }
        Some(("unalias", sub_matches)) => {
            let id = sub_matches.get_one::<String>("ID").expect("required");
            graph.unset_alias(id.to_string())?;
            Ok(())
        }
        Some(("aliases", _)) => {
            graph.list_aliases()?;
            Ok(())
        }
        Some(("rename", sub_matches)) => {
            let id = sub_matches.get_one::<String>("ID").expect("required");
            let message = sub_matches.get_one::<String>("message").expect("required");
            graph.rename_node(id.to_string(), message.to_string())?;
            Ok(())
        }
        Some(("ls", sub_matches)) => {
            let depth = *sub_matches
                .get_one::<u32>("depth")
                .expect("depth should exist");
            match sub_matches.get_one::<String>("ID") {
                None => graph.list_roots(depth)?,
                Some(id) => graph.list_children(id.to_string(), depth)?,
            }
            Ok(())
        }
        Some(("lsd", _)) => {
            graph.list_dates()?;
            Ok(())
        }
        Some(("stats", sub_matches)) => {
            let id = sub_matches.get_one::<String>("ID").expect("required");
            graph.display_stats(id.to_string())?;
            Ok(())
        }
        Some(("clean", _)) => {
            *graph = graph.clean()?;
            Ok(())
        }
        Some(("export", _)) => {
            println!("{}", save::export_json(graph)?);
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
            println!("Welcome to Tuesday");
            Ok(())
        }
    }
}

fn cli() -> Command {
    Command::new("tue")
        .about("CLI Todo graph")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .arg(arg!(--local).required(false))
        .arg(arg!(--global).required(false))
        .subcommand(Command::new("add")
            .about("Adds a node to the graph")
            .arg(arg!(-r --root "Makes this a root node")
                .required(false))
            .arg(arg!(-d --date "Makes this a date node")
                .required(false))
            .arg(arg!(-u --pseudo "Makes this a pseudo node (does not count towards parent completion)")
                .required(false))
            .arg(arg!(-p --parent <ID> "Assigns this node to a parent")
                .required_unless_present("root")
                .required_unless_present("date")
            )
            .arg(arg!(<message> "This node's message"))
        )
        .subcommand(Command::new("rm")
            .about("Removes a node from the graph")
            .arg(arg!(<ID> "Which node to remove"))
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
            .about("Unlink node from all current parents, then link to a new parent")
            .arg(arg!(node: <ID> "Which node to unlink "))
            .arg(arg!(parent: <ID> "New parent for node"))
        )
        .subcommand(Command::new("setnoprop")
            .about("Sets a node's state without propogating state updates")
            .arg(arg!(<ID>).value_parser(value_parser!(NodeState)))
        )
        .subcommand(Command::new("check")
            .about("Marks a node as completed")
            .arg(arg!(<ID> "Which node to mark as completed"))
        )
        .subcommand(Command::new("uncheck")
            .about("Marks a node as incomplete")
            .arg(arg!(<ID> "Which node to mark as incomplete"))
        )
        .subcommand(Command::new("alias")
            .about("Adds an alias for a node")
            .arg(arg!(<ID> "Which node to alias"))
            .arg(arg!(<alias> "What alias to give this node"))
        )
        .subcommand(Command::new("unalias")
            .about("Removes an alias")
            .long_about("Removes all aliases of a node")
            .arg(arg!(<ID> ""))
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
            .arg(arg!(-d --depth <depth> "What depth to recursively display children")
                .default_value("1")
                .value_parser(value_parser!(u32))
            )
        )
        .subcommand(Command::new("lsd")
            .about("Lists all date nodes")
        )
        .subcommand(Command::new("stats")
            .about("Displays statistics of a node")
            .arg(arg!(<ID> "Which node to display stats"))
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
}

fn main() -> Result<()> {
    let matches = cli().get_matches();

    let (mut graph, local) = if let Some(("import", _)) = matches.subcommand() {
        let local = match (matches.get_flag("local"), matches.get_flag("global")) {
            (true, true) => bail!("Config cannot be both local and global!"),
            (false, false) => save::local_exists(std::env::current_dir()?),
            (l, _) => l,
        };
        (save::import_json_stdin()?.graph, local)
    } else {
        match (matches.get_flag("local"), matches.get_flag("global")) {
            (true, true) => bail!("Config cannot be both local and global!"),
            (true, false) => (save::load_local(std::env::current_dir()?)?, true),
            (false, true) => (save::load_global()?, false),
            (false, false) => {
                // Try to load local config otherwise load global
                match save::try_load_local(std::env::current_dir()?)? {
                    None => (save::load_global()?, false),
                    Some(g) => (g, true),
                }
            }
        }
    };

    let result: Result<()> = handle_command(matches, &mut graph);
    if let Err(e) = result {
        println!("{}\n{}", e, e.backtrace());
    }

    if local {
        save::save_local(std::env::current_dir()?, &save::Config::new(&graph))?;
    } else {
        save::save_global(&save::Config::new(&graph))?;
    }

    Ok(())
}
