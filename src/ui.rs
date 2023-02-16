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

impl fmt::Display for UIBlockType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UIBlockType::Title => write!(f, "Title"),
            UIBlockType::Creator => write!(f, "Creator"),
            UIBlockType::Year => write!(f, "Year"),
            UIBlockType::Collections => write!(f, "Collections"),
            UIBlockType::Input => write!(f, "Input"),
        }
    }
}

pub struct UIBlock {
    pub ratio: usize,
    pub ty: UIBlockType,
    pub activated: bool,
}
pub enum UIBlockType {
    Input,
    Title,
    Creator,
    Year,
    Collections,
}

fn draw_ui_block<'a, B: Backend>(f: &mut Frame<B>, rect: Rect, app: &mut App, idx: usize) {
    let block = app.ui_blocks.get(idx).unwrap();
    let entries: Vec<ListItem> = match block.ty {
        UIBlockType::Collections => app
            .collections
            .items
            .iter()
            .map(|col| ListItem::new(Span::raw(&col.collectionName)))
            .collect(),
        UIBlockType::Title => app
            .filtered_documents
            .items
            .iter()
            .map(|doc| ListItem::new(Span::raw(doc.item_data.title.to_owned())))
            .collect(),
        UIBlockType::Creator => app
            .filtered_documents
            .items
            .iter()
            .map(
                |doc| match &doc.creators {
                    Some(creators) => ListItem::new(Spans::from(vec![
                        Span::raw(creators.get(0).unwrap().firstName.as_ref().unwrap()),
                        Span::raw(" "),
                        Span::raw(creators.get(0).unwrap().lastName.as_ref().unwrap()),
                    ])),
                    None => ListItem::new(Span::raw("Unknown author(s)")),
                },
                // }
            )
            .collect(),
        UIBlockType::Year => app
            .filtered_documents
            .items
            .iter()
            .map(|doc| ListItem::new(Span::raw(&doc.item_data.pubdate[..4]).to_owned()))
            .collect(),
        _ => {
            unreachable!()
        }
    };
    let list = List::new(entries)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(match block.activated {
                    true => Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::LightGreen),
                    false => Style::default(),
                })
                .title(block.ty.to_string()),
        )
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    match block.ty {
        UIBlockType::Collections => {
            f.render_stateful_widget(list, rect, &mut app.collections.state)
        }
        _ => f.render_stateful_widget(list, rect, &mut app.filtered_documents.state),
    }
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

    let input = Paragraph::new(app.search_input.as_ref())
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, main_layout[0]);

    // Build contraints for the layout
    let mut constraints = Vec::new();
    for b in &app.ui_blocks {
        constraints.push(Constraint::Percentage(b.ratio as _));
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints.as_ref())
        .split(main_layout[1]);

    // TODO: Is there a better way?
    for i in 0..app.ui_blocks.len() {
        draw_ui_block(f, chunks[i], app, i);
    }
}
