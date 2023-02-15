use core::fmt;

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::{app::App, db_connector::Document};

impl fmt::Display for UIBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UIBlock::Title(_) => write!(f, "Title"),
            UIBlock::Creator(_) => write!(f, "Creator"),
            UIBlock::Year(_) => write!(f, "Year"),
            UIBlock::Collections(_) => write!(f, "Collections"),
        }
    }
}

pub enum UIBlock {
    Title(usize),
    Creator(usize),
    Year(usize),
    Collections(usize),
}

fn draw_items_block<'a, B: Backend>(f: &mut Frame<B>, rect: Rect, app: &mut App, block: &UIBlock) {
    let entries: Vec<ListItem> = match block {
        UIBlock::Title(_) => {
            app.filtered_documents
                .items
                .iter()
                .map(|doc| ListItem::new(Span::raw(doc.item_data.title.to_owned())))
                .collect()
        }
        UIBlock::Creator(_) => {
            app.filtered_documents
                .items
                .iter()
                .map(|doc| {
                    ListItem::new(Span::raw(match &doc.creators {
                        Some(creators) => creators.get(0).unwrap().firstName.as_ref().unwrap().to_owned(),
                        None => "Unknown author(s)".to_string(),
                    }))
                })
                .collect()
        }
        UIBlock::Year(_) => {
            app.filtered_documents
                .items
                .iter()
                .map(|doc| ListItem::new(Span::raw(&doc.item_data.pubdate[..4]).to_owned()))
                .collect()
        }
        _ => {
            unreachable!()
        }
    };
    let list = List::new(entries)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(block.to_string()),
        )
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(list, rect, &mut app.filtered_documents.state);
}

pub fn draw_main_layout<B: Backend>(
    f: &mut Frame<B>,
    app: &mut App,
    blocks_to_draw: &Vec<UIBlock>,
) {
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

    let collection_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(main_layout[1]);
    let collections: Vec<ListItem> = app
        .collections
        .items
        .iter()
        .map(|col| ListItem::new(Span::raw(&col.collectionName)))
        .collect();
    let collections_list = List::new(collections)
        .block(Block::default().borders(Borders::ALL).title("Collections"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(
        collections_list,
        collection_split[0],
        &mut app.documents.state,
    );

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(70),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(collection_split[1]);

    let input = Paragraph::new(app.search_input.as_ref())
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, main_layout[0]);

    let mut i = 0;
    for block in blocks_to_draw {
        draw_items_block(f, chunks[i], app, block);
        i += 1;
    }
}
