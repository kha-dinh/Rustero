use sqlx::query_as;

use crate::app::App;
// use sqlx::sql

// TODO: load attachment

#[derive(Debug, Clone)]
pub struct Document {
    pub item_data: ItemData,
    pub creators: Option<Vec<Creator>>,
    pub attachments: Option<Vec<Attachment>>,
}
impl FromIterator<ItemData> for Vec<Document> {
    fn from_iter<T: IntoIterator<Item = ItemData>>(iter: T) -> Self {
        iter.into_iter()
            .map(|item| Document {
                item_data: item,
                creators: None,
                attachments: None,
            })
            .collect()
    }
}
#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct ItemData {
    itemId: i64,
    pub title: String,
    pub abstracttext: String,
    pub pubdate: String,
    pub key: String,
}

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct Collection {
    collectionId: i64,
    pub collectionName: String,
    pub parentCollectionId: i64,
}

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct Attachment {
    pub contentType: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub struct Creator {
    pub firstName: Option<String>,
    pub lastName: Option<String>,
}

#[allow(non_snake_case)]
pub async fn get_attachments_for_docs(app: &mut App) -> anyhow::Result<()> {
    let pool = app.sqlite_pool.as_ref().unwrap();
    for doc in &mut app.documents.items {
        let records = query_as!(
            Attachment,
            r#"
SELECT contentType as "contentType?", path as  "path?" 
FROM itemAttachments
WHERE parentItemID = ?
"#,
            doc.item_data.itemId
        )
        .fetch_all(pool)
        .await?;
        if !records.is_empty() {
            doc.attachments = Some(records);
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn get_collections(app: &mut App) -> anyhow::Result<()> {
    let pool = app.sqlite_pool.as_ref().unwrap();
    let records = query_as!(
            Collection,
            r#"
SELECT collectionID as "collectionId!", collectionName as "collectionName!", parentCollectionId as "parentCollectionId!"
FROM collections
ORDER BY collectionName
"#,
        )
        .fetch_all(pool)
        .await?;
    app.collections.items = records;
    Ok(())
}
#[allow(non_snake_case)]
pub async fn get_creators_for_docs(app: &mut App) -> anyhow::Result<()> {
    let pool = app.sqlite_pool.as_ref().unwrap();
    for doc in &mut app.documents.items {
        let records = query_as!(
            Creator,
            r#"
SELECT firstName as "firstName?", lastName as  "lastName?" 
FROM creators JOIN itemCreators on itemCreators.creatorID = creators.creatorID
WHERE itemID = ?
ORDER BY itemCreators.orderIndex
"#,
            doc.item_data.itemId
        )
        .fetch_all(pool)
        .await?;
        if !records.is_empty() {
            doc.creators = Some(records);
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
    use super::*;
    #[test]
    fn test_get_all_item_data() {
        let mut app = App::default();
        tokio_test::block_on(app.init_sqlite()).unwrap();
        let all_items =
            tokio_test::block_on(get_all_item_data(&mut app)).expect("Expect read all docs");
        // dbg!(&all_items);
        let all_docs: Vec<Document> = Vec::from_iter(all_items);
        tokio_test::block_on(get_creators_for_docs(&mut app)).expect("Expect read all creators");
        tokio_test::block_on(get_attachments_for_docs(&mut app))
            .expect("Expect read all attachments");
        dbg!(all_docs);
    }
}
