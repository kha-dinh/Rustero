use sqlx::{query_as, SqlitePool};
// use sqlx::sql

// TODO: load attachment

#[derive(Debug, Clone)]
pub struct Document {
    pub item_data: ItemData,
    pub creators: Vec<Creator>,
}
impl FromIterator<ItemData> for Vec<Document> {
    fn from_iter<T: IntoIterator<Item = ItemData>>(iter: T) -> Self {
        iter.into_iter()
            .map(|item| Document {
                item_data: item.clone(),
                creators: Vec::new(),
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
pub struct Creator {
    pub firstName: Option<String>,
    pub lastName: Option<String>,
}

#[allow(non_snake_case)]
pub async fn get_collections(pool: &SqlitePool, col: &mut Vec<Collection>) -> anyhow::Result<()> {
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
    col.clone_from(&records);
    Ok(())
}

#[allow(non_snake_case)]
pub async fn get_creators(pool: &SqlitePool, docs: &mut Vec<Document>) -> anyhow::Result<()> {
    for doc in docs {
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
        doc.creators.clone_from(&records);
    }
    Ok(())
}

#[allow(non_snake_case)]
pub async fn get_all_item_data(
    pool: &SqlitePool,
    item_data: &mut Vec<ItemData>,
) -> anyhow::Result<()> {
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
    item_data.clone_from(&records);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    #[test]
    fn test() {
        dotenv::dotenv().ok();
        let mut all_items = Vec::new();

        let url = env::var("DATABASE_URL");
        let pool = tokio_test::block_on(SqlitePool::connect(&url.unwrap())).unwrap();
        tokio_test::block_on(get_all_item_data(&pool, &mut all_items))
            .expect("Expect read all docs");
        // dbg!(&all_items);
        let mut all_docs: Vec<Document> = Vec::from_iter(all_items);

        tokio_test::block_on(get_creators(&pool, &mut all_docs)).expect("Expect read all docs");
        dbg!(all_docs);
    }
}
