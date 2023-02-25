use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::data_structures::{Collection, Library, RcCollection};

#[derive(Debug, PartialEq)]
pub struct CollectionNode {
    pub value: CollectionNodeValue,
    pub selected: bool,
}

impl From<RcCollection> for CollectionNode {
    fn from(collection: RcCollection) -> Self {
        Self {
            value: CollectionNodeValue::Collection(collection),
            selected: false,
        }
    }
}

#[derive(Debug,PartialEq)]
pub enum CollectionNodeValue {
    Collection(RcCollection),
    Library(Rc<RefCell<Library>>),
}

pub type RcCollectionNode = Rc<RefCell<CollectionNode>>;
#[derive(Debug)]
pub struct CollectionTreeEdge {
    src: Rc<RefCell<CollectionNode>>,
    dst: Rc<RefCell<CollectionNode>>,
}
#[derive(Debug)]
pub struct CollectionTree {
    pub edges: Vec<CollectionTreeEdge>,
    pub nodes: Vec<Rc<RefCell<CollectionNode>>>,
}

// impl PartialEq for CollectionNode{
//     fn eq(&self, other: &Self) -> bool {
//         // self.value == other.value && self.selected == other.selected
//     }
// }
impl CollectionTree {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub fn get_library_nodes(&self) -> Vec<Rc<RefCell<CollectionNode>>> {
        let mut ret: Vec<Rc<RefCell<CollectionNode>>> = Vec::new();
        self.nodes.iter().for_each(|node| {
            if let CollectionNodeValue::Library(col) = &node.borrow().value {
                ret.push(node.clone());
            }
        });
        ret
    }
    pub fn get_node_children(
        &self,
        node: Rc<RefCell<CollectionNode>>,
    ) -> Vec<Rc<RefCell<CollectionNode>>> {
        let mut ret: Vec<Rc<RefCell<CollectionNode>>> = Vec::new();
        self.edges.iter().for_each(|edge| {
            if edge.src == node {
                ret.push(edge.dst.clone());
            }
        });
        ret
    }
    pub fn get_collection(&self, id: i64) -> Option<Rc<RefCell<CollectionNode>>> {
        let collection = self.nodes.iter().find(|node| match &node.borrow().value {
            CollectionNodeValue::Collection(col) => {
                if col.borrow().collectionId == id {
                    true
                } else {
                    false
                }
            }
            CollectionNodeValue::Library(lib) => false,
        });
        if let Some(library) = collection {
            Some(library.clone())
        } else {
            None
        }
    }
    pub fn get_library(&self, id: i64) -> Option<Rc<RefCell<CollectionNode>>> {
        let library = self.nodes.iter().find(|node| match &node.borrow().value {
            CollectionNodeValue::Collection(_) => false,
            CollectionNodeValue::Library(lib) => {
                if lib.borrow().libraryId == id {
                    true
                } else {
                    false
                }
            }
        });
        if let Some(library) = library {
            Some(library.clone())
        } else {
            None
        }
    }

    pub fn build_collection_tree(&mut self, collections: &mut Vec<RcCollection>) {
        let mut unique_library_id = HashSet::new();
        for col in collections.iter() {
            unique_library_id.insert(col.borrow().libraryId);
        }
        // First layer: all the libraries
        for id in unique_library_id {
            let library_node = Rc::new(RefCell::new(CollectionNode {
                value: CollectionNodeValue::Library(Rc::new(RefCell::new(Library {
                    libraryId: id,
                    // HACK: How to get library names?
                    libraryName: "Test".to_string(),
                }))),
                selected: false,
            }));

            // collections.sort_by_key(|col| col.collectionId);
            // Note: all colections has a single parent. So to build a tree
            // 1. Find all parentCollectionId == None and push it to the first layer (library)
            // Until all collection is processed:
            // 2. Find all collection with collectionID = pushed node

            self.nodes.push(library_node);
        }

        while !collections.is_empty() {
            let mut processed: Vec<i64> = Vec::new();
            for collection in collections.iter() {
                if let Some(library) = self.get_library(collection.borrow().libraryId) {
                    if let Some(parent_id) = collection.borrow().parentCollectionId {
                        if let Some(node) = self.get_collection(parent_id) {
                            let new_collection =
                                Rc::new(RefCell::new(CollectionNode::from(collection.to_owned())));
                            let new_edge = CollectionTreeEdge {
                                src: node.clone(),
                                dst: new_collection.clone(),
                            };
                            self.nodes.push(new_collection.clone());
                            self.edges.push(new_edge);
                            processed.push(collection.borrow().collectionId);
                        } else {
                            // The node havent been inserted yet, leave for later iterations
                        };
                    } else {
                        // There is no parent, so add it directly to the library
                        let new_collection =
                            Rc::new(RefCell::new(CollectionNode::from(collection.to_owned())));
                        let new_edge = CollectionTreeEdge {
                            src: library.clone(),
                            dst: new_collection.clone(),
                        };
                        self.nodes.push(new_collection);
                        self.edges.push(new_edge);
                        processed.push(collection.borrow().collectionId);
                    }
                } else {
                    panic!("Collection without library ID!");
                };
            }
            for i in &processed {
                // collections.remove(*i);
                collections.remove(
                    collections
                        .iter()
                        .position(|col| col.borrow().collectionId == *i)
                        .unwrap(),
                );
            }
            processed.clear()
        }
        //     for collection in collections
        //         .iter()
        //         .filter(|col| col.libraryId == id && col.parentCollectionId.is_none())
        //     {
        //         let new_node = Rc::new(RefCell::new(CollectionNode {
        //             value: CollectionNodeValue::Collection(Rc::new(RefCell::new(collection.clone()))),
        //             selected: false,
        //             child: Vec::new(),
        //             parent: None,
        //         }));

        //         // Subsequence layers:

        //         library_node.borrow_mut().child.push(new_node)
        //     }

        // }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        app::App,
        db_connector::{get_all_item_data, get_attachments_for_docs, get_collections},
        user_config::UserConfig,
    };

    use super::*;
    #[test]
    fn test_build_collection_tree() {
        let mut app = App::default();
        let user_config = UserConfig::new();
        tokio_test::block_on(app.init_sqlite(&user_config.behavior.zotero_db_path)).unwrap();
        let all_items =
            tokio_test::block_on(get_all_item_data(&mut app)).expect("Expect read all docs");
        // dbg!(&all_items);
        // let all_docs: Vec<RcDoc> = Vec::from_iter(all_items);
        // tokio_test::block_on(get_creators_for_docs(&mut app)).expect("Expect read all creators");
        // tokio_test::block_on(get_attachments_for_docs(&mut app))
        // .expect("Expect read all attachments");
        tokio_test::block_on(get_collections(&mut app)).expect("Expect read all creators");
        app.collection_tree
            .build_collection_tree(&mut app.collections.items);
        // dbg!(&app.collection_tree.nodes);
        dbg!(&app.collection_tree.get_library_nodes());
        // dbg!(all_docs);
    }
}
