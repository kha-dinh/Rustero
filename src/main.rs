mod db_connector;
mod event;
mod user_config;
use anyhow::Result;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use db_connector::Document;
use event::Key;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use sqlx::SqlitePool;
/// A simple example demonstrating how to handle user input. This is
/// a bit out of the scope of the library as it does not provide any
/// input handling out of the box. However, it may helps some to get
/// started.
///
/// This is a very simple example:
///   * A input box always focused. Every character you type is registered
///   here
///   * Pressing Backspace erases a character
///   * Pressing Enter pushes the current input in the history of previous
///   messages
use std::{env, error::Error, io};
use tokio;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;
use user_config::UserConfig;

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    search_input: String,
    sqlite_pool: Option<SqlitePool>,
    /// History of recorded messages
    documents: StatefulList<Document>,
    // documents_state: StatefulList<Document>,
}

impl Default for App {
    fn default() -> App {
        App {
            search_input: String::new(),
            sqlite_pool: None,
            documents: StatefulList {
                state: ListState::default(),
                items: Vec::new(),
            },
        }
    }
}
impl App {
    fn update_on_tick(&self) {}
}

async fn start_ui(user_config: UserConfig) -> Result<()> {
    dotenv::dotenv().ok();
    let url = env::var("DATABASE_URL");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // create app and run it
    let events = event::Events::new(user_config.behavior.tick_rate_milliseconds);
    let mut app = App::default();
    app.sqlite_pool = Option::from(SqlitePool::connect(&url.unwrap()).await?);
    let mut item_data = Vec::new();
    db_connector::get_all_item_data(app.sqlite_pool.as_mut().unwrap(), &mut item_data).await?;
    app.documents.items = Vec::from_iter(item_data);

    let mut is_first_render = true;
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        if is_first_render {
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

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(f.size());

    f.set_cursor(
        // Put cursor past the end of the input text
        main_layout[0].x + app.search_input.width() as u16 + 1,
        // Move one line down, from the border to the input line
        main_layout[0].y + 1,
    );
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(50),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(main_layout[1]);

    // let test = Layout::default()
    //     .margin(2)
    //     .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
    //     .split(chunks[1]);
    //

    let input = Paragraph::new(app.search_input.as_ref())
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, main_layout[0]);

    let matcher = SkimMatcherV2::default();

    let mut filtered_doc = app.documents.items.iter().filter(|doc| {
        // Match fuzzy find
        matcher
            .fuzzy_match(&doc.item_data.title, app.search_input.as_str())
            .is_some()
    });

    let titles: Vec<ListItem> = filtered_doc
        .clone()
        .map(|doc| {
            let header = Span::raw(&doc.item_data.title);
            ListItem::new(header)
        })
        .collect();

    let titles = List::new(titles)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    let pubdate: Vec<ListItem> = filtered_doc
        .clone()
        .map(|doc| {
            let header = Span::raw(&doc.item_data.pubdate[..4]);
            ListItem::new(header)
        })
        .collect();

    let pubdate = List::new(pubdate)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );
    let author: Vec<ListItem> = filtered_doc
        .clone()
        .map(|doc| {
            let header = Span::raw(&doc.creators[0].firstName);
            ListItem::new(header)
        })
        .collect();

    let author = List::new(author)
        .block(Block::default().borders(Borders::ALL).title("creator"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(titles, chunks[1], &mut app.documents.state);
    f.render_stateful_widget(pubdate, chunks[3], &mut app.documents.state);
    f.render_stateful_widget(author, chunks[2], &mut app.documents.state);

    // let documents = List::new(app.documents);
    // let chunks = Layout::default()
    //     .direction(Direction::Vertical)
    //     .margin(2)
    //     .constraints(
    //         [
    //             Constraint::Length(1),
    //             Constraint::Length(3),
    //             Constraint::Min(1),
    //         ]
    //         .as_ref(),
    //     )
    //     .split(f.size());
    // .split(f.size());

    // let (msg, style) = match app.input_mode {
    //     InputMode::Normal => (
    //         vec![
    //             Span::raw("Press "),
    //             Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
    //             Span::raw(" to exit, "),
    //             Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
    //             Span::raw(" to start editing."),
    //         ],
    //         Style::default().add_modifier(Modifier::RAPID_BLINK),
    //     ),
    //     InputMode::Editing => (
    //         vec![
    //             Span::raw("Press "),
    //             Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
    //             Span::raw(" to stop editing, "),
    //             Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
    //             Span::raw(" to record the message"),
    //         ],
    //         Style::default(),
    //     ),
    // };
    // let mut text = Text::from(Spans::from(msg));
    // text.patch_style(style);
    // let help_message = Paragraph::new(text);
    // f.render_widget(help_message, chunks[0]);

    // match app.input_mode {
    //     InputMode::Normal =>
    //         // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
    //         {}

    //     InputMode::Editing => {
    //         // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
    //         f.set_cursor(
    //             // Put cursor past the end of the input text
    //             chunks[1].x + app.input.width() as u16 + 1,
    //             // Move one line down, from the border to the input line
    //             chunks[1].y + 1,
    //         )
    //     }
    // }

    // let messages: Vec<ListItem> = app
    //     .messages
    //     .iter()
    //     .enumerate()
    //     .map(|(i, m)| {
    //         let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
    //         ListItem::new(content)
    //     })
    //     .collect();
    // let messages =
    //     List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    // f.render_widget(messages, chunks[2]);
}
