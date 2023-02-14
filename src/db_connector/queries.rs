use sqlx::sqlite::SqliteConnection;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{query, query_as, Connection, SqlitePool};
// use sqlx::sql
use std::env;
use std::future::Future;
use std::task::Poll;

#[derive(Debug)]
#[warn(non_snake_case)]
pub struct Document {
    itemId: i64,
    title: String,
    abstracttext: String,
    pubdate: String,
    key: String,
}


pub async fn get_all_docs(pool: &SqlitePool) -> anyhow::Result<Vec<Document>> {
    // let conn = pool.acquire().await?;
    let docs = Vec::new();
    let records = query_as!(
    Document,
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

    dbg!(records);
    Ok(docs)
}
async fn do_test() -> anyhow::Result<()> {
    let url = env::var("DATABASE_URL")?;
    let pool = SqlitePool::connect(&url).await?;
    get_all_docs(&pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        dotenv::dotenv().ok();
        tokio_test::block_on(do_test()).unwrap();
    }
}
