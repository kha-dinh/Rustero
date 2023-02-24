use std::{cell::RefCell, rc::Rc};

use crate::data_structures::*;

use sqlx::query_as;

use crate::app::App;
// use sqlx::sql

#[allow(non_snake_case)]
pub async fn get_attachments_for_docs(app: &mut App) -> anyhow::Result<()> {
    let pool = app.sqlite_pool.as_ref().unwrap();

    for doc in &app.documents {
        let itemId = doc.borrow().item_data.itemId;

        let records = query_as!(
            Attachment,
            r#"
SELECT contentType as "contentType?", path as  "path?", key as "key?"
FROM itemAttachments JOIN items on itemAttachments.itemID = items.itemID
WHERE parentItemID = ?
"#,
            itemId
        )
        .fetch_all(pool)
        .await?;
        if !records.is_empty() {
            doc.borrow_mut().attachments = Some(StatefulList::with_items(records));
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn get_collections_items(app: &mut App) -> anyhow::Result<()> {
    let pool = app.sqlite_pool.as_ref().unwrap();

    let records = query_as!(
        CollectionItem,
        r#"
SELECT collectionID as "collectionId!", itemID as "itemId!"
FROM collectionItems
"#,
    )
    .fetch_all(pool)
    .await?;
    assert!(!app.documents.is_empty());
    // TODO: Handle multiple collection, same record.
    for record in records {
        if let Some(doc) = app
            .documents
            .iter()
            .find(|doc| doc.borrow().item_data.itemId == record.itemId)
        {
            if let Some(collection) = app
                .collections
                .items
                .iter()
                .find(|col| col.borrow().collectionId == record.collectionId)
            {
                doc.borrow_mut().collections.push(collection.clone());
            }
        };
    }

    // app.collections.items = records;
    Ok(())
}
#[allow(non_snake_case)]
pub async fn get_collections(app: &mut App) -> anyhow::Result<()> {
    let pool = app.sqlite_pool.as_ref().unwrap();

    let records = query_as!(
            Collection,
            r#"
SELECT collectionID as "collectionId!", collectionName as "collectionName!", parentCollectionId as "parentCollectionId?"
FROM collections
ORDER BY collectionName
"#,
        )
        .fetch_all(pool)
        .await?;
    for record in records {
        app.collections.items.push(Rc::new(RefCell::new(record)));
    }
    Ok(())
}
#[allow(non_snake_case)]
pub async fn get_creators_for_docs(app: &mut App) -> anyhow::Result<()> {
    let pool = app.sqlite_pool.as_ref().unwrap();
    for doc in &mut app.documents {
        let itemId = doc.borrow().item_data.itemId;
        let records = query_as!(
            Creator,
            r#"
SELECT firstName as "firstName?", lastName as  "lastName?" 
FROM creators JOIN itemCreators on itemCreators.creatorID = creators.creatorID
WHERE itemID = ?
ORDER BY itemCreators.orderIndex
"#,
            itemId
        )
        .fetch_all(pool)
        .await?;
        if !records.is_empty() {
            doc.borrow_mut().creators.extend(records);
        } else {
            doc.borrow_mut().creators.extend(vec![Creator::default()]);
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn get_all_item_data(app: &mut App) -> anyhow::Result<Vec<ItemData>> {
    let pool = app.sqlite_pool.as_ref().unwrap();

    // let conn = pool.acquire().await?;
    // let docs = Vec::new();
    let records = query_as!(
    ItemData,
        r#"
SELECT d1.itemID as "itemId!", 
    title as "title!", 
    key as "key!", 
    abstract as "abstracttext!", 
    pubdate as "pubdate!" 
FROM 
	(SELECT value as title,itemID	from itemDataValues JOIN itemData on itemDataValues.valueID = itemData.valueID WHERE fieldID = 1) as d1
	    JOIN (SELECT value as abstract, itemID	from itemDataValues JOIN itemData on itemDataValues.valueID = itemData.valueID WHERE fieldID = 2) as d2 ON d1.itemID = d2.itemID
	    JOIN (SELECT value as pubdate, itemID	from itemDataValues JOIN itemData on itemDataValues.valueID = itemData.valueID WHERE fieldID = 6) as d3 ON d1.itemID = d3.itemID	
        JOIN items ON items.itemID = d1.itemID  
"#
    )
    .fetch_all(pool)
    .await?;
    Ok(records)
}

#[cfg(test)]
mod tests {
    use crate::user_config::UserConfig;

    use super::*;
    #[test]
    fn test_get_all_item_data() {
        let mut app = App::default();
        let user_config = UserConfig::new();
        tokio_test::block_on(app.init_sqlite(&user_config.behavior.zotero_db_path)).unwrap();
        let all_items =
            tokio_test::block_on(get_all_item_data(&mut app)).expect("Expect read all docs");
        // dbg!(&all_items);
        let all_docs: Vec<RcDoc> = Vec::from_iter(all_items);
        tokio_test::block_on(get_creators_for_docs(&mut app)).expect("Expect read all creators");
        tokio_test::block_on(get_attachments_for_docs(&mut app))
            .expect("Expect read all attachments");
        dbg!(all_docs);
    }
}
