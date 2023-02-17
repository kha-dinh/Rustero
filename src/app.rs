use std::{
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
    rc::Rc,
};

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use sqlx::SqlitePool;
use tui::widgets::ListState;

use crate::{
    data_structures::{Collection, Document, StatefulList},
    ui::{UIBlock, UIBlockType},
};

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
    pub previous_block_idx: Cell<usize>,
    pub sorted: Cell<bool>,
    pub show_popup: Cell<bool>,
    pub sort_by_type: UIBlockType,
    pub error_message: String,
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
            error_message: String::new(),
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
            previous_block_idx: Cell::from(1),
            sort_by_type: UIBlockType::Title,
            ui_blocks: Vec::new(),
            sorted: Cell::from(false),
            show_popup: Cell::from(false),
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
        // let active_block = self.ui_blocks.get(self.active_block_idx.get()).unwrap();
        let ty = self.get_active_block().borrow().ty;
        self.filtered_documents.items.sort_unstable_by(|a, b| {
            let _a = a.borrow();
            let _b = b.borrow();
            let cmp_str_a = _a.get_cmp_str_for_block_type(ty);
            let cmp_str_b = _b.get_cmp_str_for_block_type(ty);
            let cmp = cmp_str_a.partial_cmp(cmp_str_b).unwrap();
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
            self.get_active_block().borrow_mut().activated = false;
            self.active_block_idx.set(cur_idx + 1);
            self.refresh_active_block();
        }
    }
    pub fn refresh_active_block(&mut self) {
        self.get_active_block().borrow_mut().activated = true;
    }
    // NOTE: Using pointer is actually more cumbersome than just using an index.
    // match app.active_block {
    //     Some(block) => {
    //         let next_block_id = app
    //             .ui_blocks
    //             .iter()
    //             .find(|b| {
    //                 b.as_ptr() == app.active_block.as_ref().unwrap().as_ptr()
    //             })
    //             .unwrap();
    //         app.active_block =
    //             Some(app.ui_blocks.get(next_block_id).unwrap().clone());
    //     }
    //     None => {}
    // }
    pub fn get_selected_doc(&self) -> Option<Rc<RefCell<Document>>> {
        if let Some(idx) = self.filtered_documents.state.selected() {
            Some(self.filtered_documents.items.get(idx).unwrap().clone())
        } else {
            None
        }
    }
    pub fn get_block_with_type(&self, ty: UIBlockType) -> Rc<RefCell<UIBlock>> {
        self.ui_blocks
            .iter()
            .find(|block| block.borrow().ty == ty)
            .unwrap()
            .clone()
    }
    pub fn set_active_block_with_type(&mut self, ty: UIBlockType) {
        self.get_active_block().borrow_mut().activated = false;
        let new_block_idx = self
            .ui_blocks
            .iter()
            .position(|block| block.borrow().ty == ty)
            .unwrap();
        self.active_block_idx.set(new_block_idx);
        self.ui_blocks
            .get(new_block_idx)
            .unwrap()
            .borrow_mut()
            .activated = true;
    }
    pub fn get_active_block(&self) -> Rc<RefCell<UIBlock>> {
        self.ui_blocks
            .get(self.active_block_idx.get())
            .unwrap()
            .clone()
    }
    pub fn select_prev_block(&mut self) {
        let cur_idx = self.active_block_idx.get();
        if self.active_block_idx.get() > 0 {
            self.get_active_block().borrow_mut().activated = false;
            self.active_block_idx.set(cur_idx - 1);
            self.get_active_block().borrow_mut().activated = true;
        }
    }
    pub fn update_filtered_doc(&mut self) {
        let matcher = SkimMatcherV2::default();
        if !self.search_input.is_empty() {
            // TODO: adding search character is cheaper because we can reuse the current list to match.
            // Removing search character should clear and fuzzy search from begining (except for when we
            // store some kind of history). Leaving it for later
            self.filtered_documents.items.clear();
            self.filtered_documents.items.extend(
                self.documents
                    .iter()
                    .filter(|doc| {
                        // match fuzzy find
                        // TODO: maybe we should cache headers somewhere so we dont have to build
                        // string every time.
                        let entry = doc.borrow().build_header_for_block_type(self.sort_by_type);
                        matcher
                            .fuzzy_match(&entry, self.search_input.as_str())
                            .is_some()
                    })
                    .map(|item| item.clone()),
            );
        } else {
            // Reclone only if it is less than max len.
            if self.filtered_documents.items.len() < self.documents.len() {
                self.filtered_documents.items.clear();
                self.filtered_documents
                    .items
                    .extend(self.documents.iter().map(|item| item.clone()));
            }
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
