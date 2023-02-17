use std::{rc::Rc, cell::RefCell};

use crate::ui::UIBlockType;

#[derive(Debug, Clone)]
pub struct Document {
    pub item_data: ItemData,
    pub creators: Vec<Creator>,
    pub attachments: Option<Vec<Attachment>>,
}

impl Document {
    pub fn get_str_for_block_type(&self, ty: UIBlockType) -> &str {
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
impl FromIterator<ItemData> for Vec<Rc<RefCell<Document>>> {
    fn from_iter<T: IntoIterator<Item = ItemData>>(iter: T) -> Self {
        iter.into_iter()
            .map(|item| {
                Rc::new(RefCell::new(Document {
                    item_data: item,
                    creators: Vec::new(),
                    attachments: None,
                }))
            })
            .collect()
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
