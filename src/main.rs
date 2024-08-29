use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use graph::{ErrorType, NodeState};

mod graph;
mod save;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    local: bool,

    #[arg(long)]
    global: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds a node to the graph
    Add {
        /// Adds a root node (mutually exclusive to date)
        #[arg(short, long)]
        root: bool,

        /// Adds a date node (mutually exclusive to root)
        #[arg(short, long)]
        date: bool,

        /// Parent of node
        #[arg(short, long)]
        parent: Option<String>,

        #[arg(long)]
        pseudo: bool,

        /// What message should the node have
        #[arg(short, long)]
        message: String,
    },

    /// Removes a node from the graph
    Rm {
        /// Node index to remove
        #[arg(short, long)]
        target: String,

        /// Whether to remove child nodes recursively
        #[arg(short, long)]
        recursive: bool,
    },

    /// Links a node to a target node
    Link {
        /// Node to link from
        #[arg(short, long)]
        from: String,

        /// Node to link to
        #[arg(short, long)]
        to: String,
    },

    /// Unlinks a node
    Unlink {
        #[arg(short, long)]
        from: String,

        #[arg(short, long)]
        to: String,
    },

    /// Sets node status
    SetNoprop {
        #[arg(value_enum)]
        state: graph::NodeState,

        #[arg(short, long)]
        target: String,
    },

    /// Marks a node as completed
    Check {
        #[arg(short, long)]
        target: String,
    },

    /// Marks a node as incomplete
    Uncheck {
        #[arg(short, long)]
        target: String,
    },

    /// Adds an alias for a node
    Alias {
        #[arg(short, long)]
        target: String,

        #[arg(short, long)]
        alias: String,
    },

    Unalias {
        #[arg(short, long)]
        target: String,
    },

    /// Lists aliases
    Aliases,

    /// Renames a node
    Rename {
        #[arg(short, long)]
        target: String,

        #[arg(short, long)]
        message: String,
    },

    /// Lists root nodes or children nodes
    Ls {
        #[arg(short, long)]
        target: Option<String>,

        /// No value or 0 = infinite depth
        #[arg(short, long)]
        depth: Option<u32>,
    },

    /// Lists date nodes
    Lsd,

    /// Displays statistics of a node
    Stats {
        #[arg(short, long)]
        target: String,
    },

    /// Compresses and cleans up the graph
    Clean,

    /// Exports JSON to stdout
    Export,

    /// Imports JSON from stdin
    Import,
}

fn handle_command(commands: Commands, graph: &mut graph::Graph) -> Result<()> {
    match commands {
        Commands::Add {
            root,
            date,
            parent,
            pseudo,
            message,
        } => {
            if date && root {
                bail!("Node cannot be both date node and root node!");
            }
            if date {
                if !graph::Graph::is_date(message.as_str()) {
                    return Err(ErrorType::MalformedDate(message))?;
                    // how does ? do that lol
                }
                graph.insert_date(message);
            } else if root {
                graph.insert_root(message, pseudo);
            } else if let Some(target) = parent {
                graph.insert_child(message, target, pseudo)?;
            } else {
                bail!("Did not specify whether to add as root or as a child node!");
            }
            Ok(())
        }
        Commands::Rm { target, recursive } => {
            if recursive {
                graph.remove_children_recursive(target)?;
            } else {
                graph.remove(target)?;
            }
            Ok(())
        }
        Commands::Link { from, to } => {
            graph.link(from, to)?;
            Ok(())
        }
        Commands::Unlink { from, to } => {
            graph.unlink(from, to)?;
            Ok(())
        }
        Commands::SetNoprop { state, target } => {
            graph.set_state(target, state, false)?;
            Ok(())
        }
        Commands::Check { target } => {
            graph.set_state(target, NodeState::Complete, true)?;
            Ok(())
        }
        Commands::Uncheck { target } => {
            graph.set_state(target, NodeState::None, true)?;
            Ok(())
        }
        Commands::Alias { target, alias } => {
            graph.set_alias(target, alias)?;
            Ok(())
        }
        Commands::Unalias { target } => {
            graph.unset_alias(target)?;
            Ok(())
        }
        Commands::Aliases => {
            graph.list_aliases()?;
            Ok(())
        }
        Commands::Rename { target, message } => {
            graph.rename_node(target, message)?;
            Ok(())
        }
        Commands::Ls { target, depth } => {
            match target {
                None => graph.list_roots(depth.unwrap_or(1))?,
                Some(t) => graph.list_children(t, depth.unwrap_or(0))?,
            }
            Ok(())
        }
        Commands::Lsd => {
            graph.list_dates()?;
            Ok(())
        }
        Commands::Stats { target } => {
            graph.display_stats(target)?;
            Ok(())
        }
        Commands::Clean => {
            *graph = graph.clean()?;
            Ok(())
        }
        Commands::Export => {
            println!("{}", save::export_json(graph)?);
            Ok(())
        }
        Commands::Import => {
            // Import would have finished by this stage so just log received data
            println!(
                "Successfully imported json! {} nodes; {} root nodes; {} aliases",
                graph.node_count(),
                graph.root_count(),
                graph.alias_count()
            );
            Ok(())
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.command.is_none() {
        println!("Welcome to Tuesday");
        return Ok(());
    }

    let (mut graph, local) = if let Some(&Commands::Import) = cli.command.as_ref() {
        let local = match (cli.local, cli.global) {
            (true, true) => bail!("Config cannot be both local and global!"),
            (false, false) => save::local_exists(std::env::current_dir()?),
            (l, _) => l,
        };
        (save::import_json_stdin()?.graph, local)
    } else {
        match (cli.local, cli.global) {
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

    let result: Result<()> = handle_command(cli.command.unwrap(), &mut graph);
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
