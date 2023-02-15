mod app;
mod db_connector;
mod event;
mod handler;
mod ui;
mod user_config;

use app::App;
use db_connector::get_all_item_data;
use handler::*;

use anyhow::Result;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tokio;
use tui::{backend::CrosstermBackend, Terminal};

use crate::db_connector::{get_attachments_for_docs, get_collections, get_creators_for_docs};
use crate::event::Key;
use crate::ui::draw_main_layout;
use crate::user_config::UserConfig;

async fn start_ui(user_config: UserConfig) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // create app and run it
    let events = event::Events::new(user_config.behavior.tick_rate_milliseconds);
    let mut app = App::default();

    let mut is_first_render = true;
    loop {
        terminal.draw(|f| draw_main_layout(f, &mut app))?;
        if is_first_render {
            app.init_sqlite().await?;

            app.documents.items = Vec::from_iter(get_all_item_data(&mut app).await?);
            get_creators_for_docs(&mut app).await?;
            get_attachments_for_docs(&mut app).await?;
            get_collections(&mut app).await?;
            is_first_render = false;
        }
        match events.next()? {
            event::Event::Input(key) => {
                if key == Key::Ctrl('c') {
                    break;
                }
                match key {
                    Key::Down => app.documents.next(),
                    Key::Up => app.documents.previous(),
                    Key::Backspace => {
                        app.search_input.pop();
                        app.documents.state.select(Some(0));
                    }
                    Key::Char(c) => {
                        app.search_input.push(c);
                        app.documents.state.select(Some(0));
                    }
                    Key::Enter => {
                        handle_enter(&app).await?;
                    }
                    _ => {}
                }
            }

            event::Event::Tick => {
                app.update_on_tick();
            }
        }
    }
    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // setup terminal
    let mut user_config = UserConfig::new();

    start_ui(user_config).await?;
    Ok(())
}
