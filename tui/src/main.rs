use anyhow::Result;
use app::App;
use clap::Parser;
use components::{new_layout, AppLayout};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::process_key;
use ratatui::crossterm::execute;
use ratatui::prelude::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use tuecore::doc;

pub mod app;
pub mod cli;
pub mod components;
pub mod events;

fn app_init(stderr: &mut io::Stderr) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

fn app_loop<B>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()>
where
    B: Backend,
{
    while !app.should_exit() {
        terminal.draw(|f| {
            f.render_widget(&mut app.components, f.area());

            // Set cursor to correct position when typing on Cmdline
            if app.is_capturing_keys() {
                let cmdline_rect = AppLayout::split(new_layout(), f.area()).cmdline;
                f.set_cursor_position(app.components.cmdline.get_cursor_pos(cmdline_rect))
            }
        })?;
        if let event::Event::Key(key_event) = event::read()? {
            let mut event = process_key(&app, key_event);
            loop {
                if let Some(e) = event {
                    event = app.process_event(e);
                } else {
                    break;
                }
            }
        }
    }
    io::Result::Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();

    let (graph, local) = match (args.local.is_some(), args.global) {
        // --global overrides --local argument
        (_, true) => (doc::load_global()?, false),
        (true, _) => (
            doc::load_local(PathBuf::from(
                args.local.expect("--local should provide a path"),
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

    let mut stderr = io::stderr();
    app_init(&mut stderr)?;
    let mut app = App::new();
    app.load_graph(graph);

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    app_loop(&mut terminal, &mut app)?;

    // TODO: extract these below into a new function
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;

    Ok(())
}
