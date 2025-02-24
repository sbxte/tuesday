use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about = "TUI for Tuesday")]
pub struct Args {
    #[arg(short, long, default_value = "")]
    pub(crate) local: Option<String>,

    #[arg(short, long)]
    pub(crate) global: bool,
}
