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
            UIBlock::Title => write!(f, "Title"),
            UIBlock::Creator => write!(f, "Creator"),
            UIBlock::Year => write!(f, "Year"),
        }
    }
}

enum UIBlock {
    Title,
    Creator,
    Year,
}

fn draw_block<'a, B: Backend, I>(
    f: &mut Frame<B>,
    rect: Rect,
    filtered_docs: I,
    state: &mut ListState,
    block: UIBlock,
) where
    I: Iterator<Item = &'a Document>,
{
    let entries: Vec<ListItem> = filtered_docs
        .map(|doc| {
            let header = match block {
                UIBlock::Title => Span::raw(doc.item_data.title.to_owned()),
                UIBlock::Year => Span::raw(doc.item_data.pubdate[..4].to_owned().to_string()),
                UIBlock::Creator => Span::raw(match &doc.creators {
                    Some(creators) => creators
                        .get(0)
                        .unwrap()
                        .firstName
                        .as_ref()
                        .unwrap()
                        .to_owned(),
                    None => "Unknown author(s)".to_owned(),
                }),
            };
            ListItem::new(header)
        })
        .collect();
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

    f.render_stateful_widget(list, rect, state);
}

pub fn draw_main_layout<B: Backend>(f: &mut Frame<B>, app: &mut App) {
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
    let blocks_to_draw = vec![UIBlock::Title, UIBlock::Creator, UIBlock::Year];

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

    let matcher = SkimMatcherV2::default();

    let filtered_docs = app.documents.items.iter().filter(|doc| {
        // Match fuzzy find
        matcher
            .fuzzy_match(&doc.item_data.title, app.search_input.as_str())
            .is_some()
    });

    let mut i = 0;
    for block in blocks_to_draw {
        draw_block(
            f,
            chunks[i],
            filtered_docs.clone(),
            &mut app.documents.state,
            block,
        );
        i += 1;
    }
}
