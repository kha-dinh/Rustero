use core::fmt;
use std::{cell::RefCell, rc::Rc};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::app::App;

impl fmt::Display for UIBlockType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UIBlockType::Menu => write!(f, "Menu"),
            UIBlockType::Title => write!(f, "Title"),
            UIBlockType::Creator => write!(f, "Creator"),
            UIBlockType::Year => write!(f, "Year"),
            UIBlockType::Collections => write!(f, "Collections"),
            UIBlockType::Input => write!(f, "Input"),
        }
    }
}
pub type RcUIBlock = Rc<RefCell<UIBlock>>;
pub struct UIBlock {
    pub ratio: usize,
    pub ty: UIBlockType,
    pub activated: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIBlockType {
    Menu,
    Input,
    Title,
    Creator,
    Year,
    Collections,
}
impl UIBlockType {
    pub fn is_searchable(&self) -> bool {
        !matches!(self, Self::Menu | Self::Input | Self::Collections)
    }
}

fn draw_ui_block<'a, B: Backend>(f: &mut Frame<B>, rect: Rect, app: &mut App, block: RcUIBlock) {
    // let block = app.ui_blocks.get(idx).unwrap();
    let entries: Vec<ListItem> = match block.borrow().ty {
        UIBlockType::Input => unreachable!(),
        UIBlockType::Collections => app
            .collections
            .items
            .iter()
            .map(|col| ListItem::new(Span::raw(&col.collectionName)))
            .collect(),
        _ => app
            .filtered_documents
            .items
            .iter()
            .map(|doc| {
                ListItem::new(Span::raw(
                    doc.borrow().build_header_for_block_type(block.borrow().ty),
                ))
            })
            .collect(),
    };
    let list = List::new(entries)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(match block.borrow().activated {
                    true => Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::LightGreen),
                    false => Style::default(),
                })
                .title(block.borrow().ty.to_string()),
        )
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    match block.borrow().ty {
        UIBlockType::Input | UIBlockType::Menu => unreachable!(),
        UIBlockType::Collections => {
            f.render_stateful_widget(list, rect, &mut app.collections.state)
        }
        _ => f.render_stateful_widget(list, rect, &mut app.filtered_documents.state),
    }
}

fn build_constraints(blocks: &Vec<RcUIBlock>) -> Vec<Constraint> {
    blocks
        .iter()
        .to_owned()
        .map(|block| Constraint::Percentage(block.borrow().ratio as _))
        .collect()
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

    let input = Paragraph::new(app.search_input.as_ref()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(
                match app
                    .get_block_with_type(UIBlockType::Input)
                    .borrow()
                    .activated
                {
                    true => Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::LightGreen),
                    false => Style::default(),
                },
            )
            .title("Input"),
    );

    f.render_widget(input, main_layout[0]);

    // Configurable UI blocks
    let flex_ui_blocks: Vec<Rc<RefCell<UIBlock>>> = app
        .ui_blocks
        .iter()
        .filter(|block| match block.borrow().ty {
            UIBlockType::Menu | UIBlockType::Input => false,
            _ => true,
        })
        .map(|block| block.clone())
        .collect();
    // Build contraints for the layout
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(build_constraints(&flex_ui_blocks))
        .split(main_layout[1]);

    for (i, b) in flex_ui_blocks.iter().enumerate() {
        draw_ui_block(f, chunks[i], app, b.to_owned());
    }
}
