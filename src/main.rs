mod app;
mod data_structures;
mod db_connector;
mod event;
mod handler;
mod ui;
mod user_config;

use app::App;
use data_structures::Collection;
use db_connector::get_all_item_data;
use handler::*;

use anyhow::Result;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::{cell::RefCell, rc::Rc};
use tokio;
use tui::{backend::CrosstermBackend, Terminal};
use ui::{UIBlock, UIBlockType};

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
    app.ui_blocks.extend(vec![
        Rc::new(RefCell::new(UIBlock {
            ratio: 20,
            ty: UIBlockType::Collections,
            activated: false,
        })),
        Rc::new(RefCell::new(UIBlock {
            ratio: 50,
            ty: UIBlockType::Title,
            activated: false,
        })),
        Rc::new(RefCell::new(UIBlock {
            ratio: 20,
            ty: UIBlockType::Creator,
            activated: false,
        })),
        Rc::new(RefCell::new(UIBlock {
            ratio: 10,
            ty: UIBlockType::Year,
            activated: false,
        })),
    ]);
    loop {
        terminal.draw(|f| draw_main_layout(f, &mut app))?;
        if is_first_render {
            app.init_sqlite(&user_config.behavior.zotero_db_path)
                .await?;

            app.documents = Vec::from_iter(get_all_item_data(&mut app).await?);
            get_creators_for_docs(&mut app).await?;
            get_attachments_for_docs(&mut app).await?;
            get_collections(&mut app).await?;

            app.collections.items.push(Collection {
                collectionId: 0,
                collectionName: "My Library".to_owned(),
                parentCollectionId: None,
            });
            app.update_filtered_doc();
            is_first_render = false;
        }
        match events.next()? {
            event::Event::Input(key) => {
                if key == Key::Ctrl('c') {
                    break;
                }
                match key {
                    Key::Down => match app
                        .ui_blocks
                        .get(app.active_block_idx.get())
                        .unwrap()
                        .borrow()
                        .ty
                    {
                        UIBlockType::Collections => {
                            app.collections.next();
                        }
                        _ => {
                            app.filtered_documents.next();
                        }
                    },
                    Key::Right => app.select_next_block(),
                    Key::Left => app.select_prev_block(),
                    Key::Up => match app
                        .ui_blocks
                        .get(app.active_block_idx.get())
                        .unwrap()
                        .borrow()
                        .ty
                    {
                        // TODO: make this more flexible
                        UIBlockType::Collections => {
                            app.collections.previous();
                        }
                        _ => {
                            app.filtered_documents.previous();
                        }
                    },
                    Key::Backspace => {
                        app.search_input.pop();
                        app.update_filtered_doc();
                    }
                    Key::Ctrl(c) => match c {
                        's' => {
                            app.toggle_sorted();
                            if app.sorted.get() {
                                app.sort_documents()
                            } else {
                                app.unsort_documents()
                            }
                        }
                        _ => {}
                    },
                    Key::Char(c) => {
                        app.search_input.push(c);
                        app.update_filtered_doc();
                    }
                    Key::Enter => {
                        handle_enter(
                            &app,
                            &user_config.behavior.zotero_storage_dir,
                            &user_config.behavior.pdf_viewer,
                        )
                        .await?;
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
    user_config.load_config().unwrap();
    start_ui(user_config).await?;
    Ok(())
}
