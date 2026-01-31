use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::collections::HashSet;
use std::io;
use std::path::PathBuf;

pub mod state;
pub mod view;

use crate::core::config::ContextConfig;
use crate::core::file::FileNode;
use crate::ui::state::App;

pub fn run_tui(
    files: &[FileNode],
    root_path: &std::path::Path,
    initial_config: ContextConfig,
) -> Result<Option<(HashSet<PathBuf>, ContextConfig)>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(files, root_path, initial_config);
    let res = run_app_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        return Err(err);
    }

    if app.confirmed {
        Ok(Some((app.get_selected_paths(), app.config)))
    } else {
        Ok(None)
    }
}

fn run_app_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    while !app.should_quit {
        terminal.draw(|f| view::render_app(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                        KeyCode::Enter => app.confirm(),
                        KeyCode::Char('c') => app.toggle_clipboard(),
                        KeyCode::Char('m') => app.toggle_minify(),
                        KeyCode::Char('f') => app.cycle_format(),
                        KeyCode::Char('o') => app.toggle_output_destination(),
                        KeyCode::Up => app.move_up(),
                        KeyCode::Down => app.move_down(),
                        KeyCode::Char(' ') => app.toggle_selection(),
                        KeyCode::Right | KeyCode::Left => app.toggle_expand(),

                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}