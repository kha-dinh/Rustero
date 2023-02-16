use std::{
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
    rc::Rc,
};

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use sqlx::SqlitePool;
use tui::widgets::ListState;

use crate::{
    db_connector::{Collection, Document},
    ui::{UIBlock, UIBlockType},
};

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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
pub struct App {
    /// Current value of the input box
    pub search_input: String,
    pub sqlite_pool: Option<SqlitePool>,
    /// History of recorded messages
    pub documents: Vec<Rc<RefCell<Document>>>,
    pub filtered_documents: StatefulList<Rc<RefCell<Document>>>,
    pub collections: StatefulList<Collection>,
    pub zotero_dir: PathBuf,
    pub active_block_idx: Cell<usize>,
    pub sorted: Cell<bool>,
    pub sort_direction: Cell<SortDirection>,
    pub ui_blocks: Vec<Rc<RefCell<UIBlock>>>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SortDirection {
    Up,
    Down,
}
impl Default for App {
    fn default() -> App {
        App {
            sort_direction: Cell::from(SortDirection::Up),
            search_input: String::new(),
            sqlite_pool: None,
            documents: Vec::new(),
            collections: StatefulList {
                state: ListState::default(),
                items: Vec::new(),
            },
            filtered_documents: StatefulList {
                state: ListState::default(),
                items: Vec::new(),
            },
            zotero_dir: PathBuf::new(),
            active_block_idx: Cell::from(1),
            sorted: Cell::from(false),
            ui_blocks: Vec::new(),
        }
    }
}

impl App {
    pub fn update_on_tick(&self) {}
    pub fn toggle_sorted(&mut self) {
        // self.sorted.set(!self.sorted.get());
        self.sorted.set(true);
        if self.sort_direction.get() == SortDirection::Up {
            self.sort_direction.set(SortDirection::Down)
        } else {
            self.sort_direction.set(SortDirection::Up)
        }
    }
    pub fn sort_documents(&mut self) {
        let active_block = self.ui_blocks.get(self.active_block_idx.get()).unwrap();
        self.filtered_documents.items.sort_unstable_by(|a, b| {
            let cmp = match &active_block.borrow().ty {
                UIBlockType::Title => a
                    .borrow()
                    .item_data
                    .title
                    .partial_cmp(&b.borrow().item_data.title)
                    .unwrap(),
                UIBlockType::Year => a.borrow().item_data.pubdate[..4]
                    .partial_cmp(&b.borrow().item_data.pubdate[..4])
                    .unwrap(),
                // WTF is this
                UIBlockType::Creator => a
                    .borrow()
                    .creators
                    .get(0)
                    .unwrap()
                    .firstName
                    .as_ref()
                    .unwrap()
                    .to_owned()
                    .partial_cmp(
                        &b.borrow()
                            .creators
                            .get(0)
                            .unwrap()
                            .firstName
                            .as_ref()
                            .unwrap()
                            .to_owned(),
                    )
                    .unwrap(),
                _ => a
                    .borrow()
                    .item_data
                    .itemId
                    .partial_cmp(&b.borrow().item_data.itemId)
                    .unwrap(),
            };
            match self.sort_direction.get() {
                SortDirection::Down => cmp.reverse(),
                SortDirection::Up => cmp,
            }
        })
    }
    pub fn unsort_documents(&mut self) {
        self.filtered_documents.items.sort_by(|a, b| {
            a.borrow()
                .item_data
                .itemId
                .partial_cmp(&b.borrow().item_data.itemId)
                .unwrap()
        })
    }
    pub fn select_next_block(&mut self) {
        let cur_idx = self.active_block_idx.get();
        if self.active_block_idx.get() < self.ui_blocks.len() - 1 {
            self.ui_blocks
                .get_mut(self.active_block_idx.get())
                .unwrap()
                .borrow_mut()
                .activated = false;
            let new_idx = cur_idx + 1;
            self.active_block_idx.set(new_idx);
            self.ui_blocks
                .get_mut(new_idx)
                .unwrap()
                .borrow_mut()
                .activated = true;
        }
    }
    pub fn select_prev_block(&mut self) {
        let cur_idx = self.active_block_idx.get();
        if self.active_block_idx.get() > 0 {
            self.ui_blocks
                .get_mut(self.active_block_idx.get())
                .unwrap()
                .borrow_mut()
                .activated = false;
            let new_idx = cur_idx - 1;
            self.active_block_idx.set(new_idx);
            self.ui_blocks
                .get_mut(new_idx)
                .unwrap()
                .borrow_mut()
                .activated = true;
        }
    }
    // TODO: adding search character is cheaper because we can reuse the current list to match.
    // Removing search character should clear and fuzzy search from begining (except for when we
    // store some kind of history). Leaving it for later
    pub fn update_filtered_doc(&mut self) {
        let matcher = SkimMatcherV2::default();
        if !self.search_input.is_empty() {
            // let collected
            self.filtered_documents.items.clear();
            self.filtered_documents.items.extend(
                self.documents
                    .iter()
                    .filter(|doc| {
                        // match fuzzy find
                        matcher
                            .fuzzy_match(&doc.borrow().item_data.title, self.search_input.as_str())
                            .is_some()
                    })
                    .map(|item| item.clone()),
            );
        } else {
            self.filtered_documents.items.clear();
            self.filtered_documents
                .items
                .extend(self.documents.iter().map(|item| item.clone()));
        }
        self.filtered_documents.state.select(Some(0));
    }
    pub async fn init_sqlite(&mut self, db_path: &Path) -> anyhow::Result<()> {
        // dotenv::dotenv().ok();
        // let url = env::var("DATABASE_URL");
        self.sqlite_pool =
            Some(SqlitePool::connect(&format!("sqlite:{}", db_path.to_str().unwrap())).await?);
        Ok(())
    }
}
