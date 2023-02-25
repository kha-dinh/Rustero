use core::fmt;
use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::{app::App, collection_tree::CollectionNodeValue};

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
            .map(|col| ListItem::new(Span::raw(col.borrow().collectionName.to_owned())))
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
fn draw_document_items<B: Backend>(f: &mut Frame<B>, rect: Rect, app: &mut App) {
    let mut rows = Vec::new();
    let mut header = Vec::new();
    header.push(Cell::from(UIBlockType::Title.to_string()));
    header.push(Cell::from(UIBlockType::Year.to_string()));
    header.push(Cell::from(UIBlockType::Creator.to_string()));
    for (idx, doc) in app.filtered_documents.items.iter().enumerate() {
        let mut cells = Vec::new();
        let doc = doc.borrow();
        let mut row_height: u16 = 1;
        // cells.push(Cell::from(first_cell_content));
        cells.push(Cell::from(doc.get_title().to_owned()));
        cells.push(Cell::from(
            doc.get_cmp_str_for_block_type(UIBlockType::Creator)
                .to_owned(),
        ));
        cells.push(Cell::from(doc.get_year().to_owned()));
        let new_row = Row::new(cells).height(row_height);
        rows.push(new_row);
        app.row_num_to_doc.insert(rows.len() - 1, idx);

        if doc.toggled.get() {
            if let Some(attachments) = &doc.attachments {
                for att in &attachments.items {
                    if let Some(path) = &att.path {
                        if (row_height as usize) < attachments.items.len() {
                            rows.push(Row::new(vec![Cell::from(
                                format!("   ├──{}", path).replace("storage:", ""),
                            )]));
                        } else {
                            rows.push(Row::new(vec![Cell::from(
                                format!("   └──{}", path).replace("storage:", ""),
                            )]));
                        }
                        row_height += 1;
                        // Update the mapping
                        app.row_num_to_doc.insert(rows.len() - 1, idx);
                    }
                }
            }
        }
    }
    // rows.push(Row::new(vec!["test", "test", "test"]));
    // rows.push(Row::new(vec!["test", "test", "test"]));
    // rows.push(Row::new(vec!["test", "test", "test"]));
    let tbl = Table::new(rows)
        // You can set the style of the entire Table.
        .style(Style::default().fg(Color::White))
        // It has an optional header, which is simply a Row always visible at the top.
        .header(
            Row::new(header).style(Style::default().fg(Color::LightGreen)), // specify some margin at the bottom.
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                // .border_style(match block.borrow().activated {
                //     true => Style::default()
                //         .add_modifier(Modifier::BOLD)
                //         .fg(Color::LightGreen),
                //     false => Style::default(),
                // })
                .title("Documents"),
        )
        // Columns widths are constrained in the same way as Layout...
        .widths(&[
            Constraint::Percentage(70),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
        ])
        // ...and they can be separated by a fixed spacing.
        .column_spacing(1)
        // If you wish to highlight a row in any specific way when it is selected...
        // .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        // ...and potentially show a symbol in front of the selection.
        .highlight_symbol(match app.get_selected_doc() {
            Some(doc) => match doc.borrow().toggled.get() {
                true => "◯❯",
                false => "◉❯",
            },
            None => "❯",
        });
    // f.render_widget(tbl, rect);
    f.render_stateful_widget(tbl, rect, &mut app.tbl_state);
}
// fn draw_collection_block<B: Backend>(f: &mut Frame<B>, rect: Rect, app: &mut App) {
// }
fn draw_collection_block<B: Backend>(f: &mut Frame<B>, rect: Rect, app: &mut App) {
    let block = app.get_block_with_type(UIBlockType::Collections);
    let mut entries: Vec<ListItem> = Vec::new();
    // dbg!(&app.collection_tree);
    // TODO: rework this mess into a recursive function...
    app.collection_tree
        .get_library_nodes()
        .iter()
        .for_each(|node| {
            // dbg!(&node);
            if let CollectionNodeValue::Library(lib) = &node.borrow().value {
                entries.push(ListItem::new(Span::raw(
                    lib.borrow().libraryName.to_owned(),
                )));
                app.collection_tree
                    .get_node_children(node.clone())
                    .iter()
                    .for_each(|node| {
                        if let CollectionNodeValue::Collection(col) = &node.borrow().value {
                            if col.borrow().parentCollectionId.is_none() {
                                entries.push(ListItem::new(Span::raw(format!(
                                    " > {}",
                                    col.borrow().collectionName.to_owned()
                                ))));

                                // Recursively visit children
                                app.collection_tree
                                    .get_node_children(node.clone())
                                    .iter()
                                    .for_each(|sub_node| {
                                        if let CollectionNodeValue::Collection(subcol) =
                                            &sub_node.borrow().value
                                        {
                                            if subcol.borrow().parentCollectionId.unwrap()
                                                == col.borrow().collectionId
                                            {
                                                entries.push(ListItem::new(Span::raw(format!(
                                                    "   > {}",
                                                    subcol.borrow().collectionName.to_owned()
                                                ))));
                                            }
                                        }
                                    });
                            }
                        }
                    });
            } else {
                panic!("Wtf?");
            }
        });

    // dbg!(&entries);
    // app.collection_tree.nodes.iter().filter(|node| { if let CollectionNodeValue::Library() node.borrow().value});

    // app.collections
    //     .items
    //     .iter()
    //     .map(|col| ListItem::new(Span::raw(col.borrow().collectionName.to_owned())))
    //     .collect();

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

    f.render_stateful_widget(list, rect, &mut app.collections.state)
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
    let vert_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(main_layout[1]);
    draw_collection_block(f, vert_split[0], app);
    draw_document_items(f, vert_split[1], app);

    // TODO: Rewrite to use table
    if false {
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
}
