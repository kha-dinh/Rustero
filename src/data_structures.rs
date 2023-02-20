use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use tui::widgets::{ListState, TableState, Row};

use crate::ui::UIBlockType;

#[derive(Debug)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

#[derive(Debug)]
pub struct StatefulTable<T> {
    pub state: TableState,
    pub items: Vec<T>,
}
trait Selectable {

}

impl<T> StatefulTable<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
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

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
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

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
pub type RcDoc = Rc<RefCell<Document>>;
#[derive(Debug)]
pub struct Document {
    pub item_data: ItemData,
    pub creators: Vec<Creator>,
    pub attachments: Option<StatefulList<Attachment>>,
    pub toggled: Cell<bool>,
}
impl FromIterator<ItemData> for Vec<RcDoc> {
    fn from_iter<T: IntoIterator<Item = ItemData>>(iter: T) -> Self {
        iter.into_iter()
            .map(|item| {
                Rc::new(RefCell::new(Document {
                    toggled: Cell::from(false),
                    item_data: item,
                    creators: Vec::new(),
                    attachments: None,
                }))
            })
            .collect()
    }
}

impl Document {
    pub fn toggle(&mut self) {
        self.toggled.set(!self.toggled.get());
    }
    pub fn get_cmp_str_for_block_type(&self, ty: UIBlockType) -> &str {
        match ty {
            UIBlockType::Title => self.get_title(),
            UIBlockType::Creator => {
                let first_author = self.creators.get(0).unwrap();
                let mut has_first_name = false;
                let mut ret = "";
                match &first_author.firstName {
                    Some(first_name) => {
                        if !first_name.trim().is_empty() {
                            has_first_name = true;
                            ret = first_name.as_str();
                        }
                    }
                    None => {}
                }
                match &first_author.lastName {
                    Some(last_name) => {
                        if !has_first_name {
                            ret = last_name.as_str()
                        }
                    }
                    None => {}
                }
                ret
                // String::from(vec![self.creators.get(0).unwrap().firstName]);
            }
            UIBlockType::Year => self.get_year(),
            UIBlockType::Collections => todo!(),
            _ => {
                unreachable!()
            }
        }
    }
    pub fn build_header_for_block_type(&self, ty: UIBlockType) -> String {
        match ty {
            UIBlockType::Title => self.get_title().to_owned(),
            UIBlockType::Creator => {
                let first_author = self.creators.get(0).unwrap();
                let mut ret = String::new();
                let mut has_first_name = false;
                match &first_author.firstName {
                    Some(first_name) => {
                        if !first_name.trim().is_empty() {
                            has_first_name = true;
                            ret.push_str(first_name);
                        }
                    }
                    None => {}
                }
                match &first_author.lastName {
                    Some(last_name) => {
                        if has_first_name {
                            ret.push_str(" ");
                        }
                        ret.push_str(&last_name);
                    }
                    None => {}
                }
                ret
                // String::from(vec![self.creators.get(0).unwrap().firstName]);
            }
            UIBlockType::Year => self.get_year().to_owned(),
            UIBlockType::Collections => todo!(),
            _ => {
                unreachable!()
            }
        }
    }
    pub fn get_title(&self) -> &str {
        self.item_data.title.as_str()
    }
    // pub fn try_get_first_name(&self) -> Option<&str> {
    //     // self.creators.get(0).unwrap().firstName.unwrap().as_str()
    // }
    pub fn get_year(&self) -> &str {
        &self.item_data.pubdate[..4]
    }
}
#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct ItemData {
    pub itemId: i64,
    pub title: String,
    pub abstracttext: String,
    pub pubdate: String,
    pub key: String,
}

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct Collection {
    pub collectionId: i64,
    pub collectionName: String,
    pub parentCollectionId: Option<i64>,
}

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct Attachment {
    pub contentType: Option<String>,
    pub path: Option<String>,
    pub key: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct Creator {
    pub firstName: Option<String>,
    pub lastName: Option<String>,
}
impl Default for Creator {
    fn default() -> Self {
        Self {
            firstName: Some("Unknown author(s)".to_string()),
            lastName: None,
        }
    }
}
