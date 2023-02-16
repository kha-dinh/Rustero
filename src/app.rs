use std::{
    env,
    path::{Path, PathBuf},
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
    pub documents: Vec<Document>,
    pub filtered_documents: StatefulList<Document>,
    pub collections: StatefulList<Collection>,
    pub zotero_dir: PathBuf,
    // TODO: putting a reference to uiblock here needs a lot of refactoring
    pub active_block_idx: usize,
    pub ui_blocks: Vec<UIBlock>,
}

impl Default for App {
    fn default() -> App {
        App {
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
            active_block_idx: 0,
            ui_blocks: Vec::new(),
        }
    }
}
impl App {
    pub fn update_on_tick(&self) {}
    pub fn update_filtered_doc(&mut self) {
        let matcher = SkimMatcherV2::default();
        if !self.search_input.is_empty() {
            self.filtered_documents.items = self
                .documents
                .clone()
                .into_iter()
                .filter(|doc| {
                    // Match fuzzy find
                    matcher
                        .fuzzy_match(&doc.item_data.title, self.search_input.as_str())
                        .is_some()
                })
                .collect();
        } else {
            self.filtered_documents.items = self.documents.clone();
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
