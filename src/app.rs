use std::env;

use sqlx::SqlitePool;
use tui::widgets::ListState;

use crate::db_connector::{Collection, Document};

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
    pub documents: StatefulList<Document>,
    pub collections: StatefulList<Collection>,
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
            collections: StatefulList {
                state: ListState::default(),
                items: Vec::new(),
            },
        }
    }
}
impl App {
    pub fn update_on_tick(&self) {}
    pub async fn init_sqlite(&mut self) -> anyhow::Result<()> {
        dotenv::dotenv().ok();
        let url = env::var("DATABASE_URL");
        self.sqlite_pool = Some(SqlitePool::connect(url.as_ref().unwrap()).await?);
        Ok(())
    }
}
